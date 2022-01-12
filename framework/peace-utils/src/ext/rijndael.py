import sys
import copy

print('{0}\nPYTHON - {1}\n{0}'.format('-' * 60, sys.version))


shifts = [[[0, 0], [1, 3], [2, 2], [3, 1]],
          [[0, 0], [1, 5], [2, 4], [3, 3]],
          [[0, 0], [1, 7], [3, 5], [4, 4]]]

# [key_size][block_size]
num_rounds = {
    16: {16: 10, 24: 12, 32: 14},
    24: {16: 12, 24: 12, 32: 14},
    32: {16: 14, 24: 14, 32: 14}
}

A = [[1, 1, 1, 1, 1, 0, 0, 0],
     [0, 1, 1, 1, 1, 1, 0, 0],
     [0, 0, 1, 1, 1, 1, 1, 0],
     [0, 0, 0, 1, 1, 1, 1, 1],
     [1, 0, 0, 0, 1, 1, 1, 1],
     [1, 1, 0, 0, 0, 1, 1, 1],
     [1, 1, 1, 0, 0, 0, 1, 1],
     [1, 1, 1, 1, 0, 0, 0, 1]]

# produce log and a_log tables, needed for multiplying in the
# field GF(2^m) (generator = 3)
a_log = [1]
for i in range(255):
    j = (a_log[-1] << 1) ^ a_log[-1]
    if j & 0x100 != 0:
        j ^= 0x11B
    a_log.append(j)

log = [0] * 256
for i in range(1, 255):
    log[a_log[i]] = i

# multiply two elements of GF(2^m)


def mul(a, b):
    if a == 0 or b == 0:
        return 0
    return a_log[(log[a & 0xFF] + log[b & 0xFF]) % 255]


# substitution box based on F^{-1}(x)
box = [[0] * 8 for i in range(256)]
box[1][7] = 1
for i in range(2, 256):
    j = a_log[255 - log[i]]
    for t in range(8):
        box[i][t] = (j >> (7 - t)) & 0x01

B = [0, 1, 1, 0, 0, 0, 1, 1]

# affine transform:  box[i] <- B + A*box[i]
cox = [[0] * 8 for i in range(256)]
for i in range(256):
    for t in range(8):
        cox[i][t] = B[t]
        for j in range(8):
            cox[i][t] ^= A[t][j] * box[i][j]

# S-boxes and inverse S-boxes
S = [0] * 256
Si = [0] * 256
for i in range(256):
    S[i] = cox[i][0] << 7
    for t in range(1, 8):
        S[i] ^= cox[i][t] << (7-t)
    Si[S[i] & 0xFF] = i

# T-boxes
G = [
    [2, 1, 1, 3],
    [3, 2, 1, 1],
    [1, 3, 2, 1],
    [1, 1, 3, 2]
]

AA = [[0] * 8 for i in range(4)]

for i in range(4):
    for j in range(4):
        AA[i][j] = G[i][j]
        AA[i][i+4] = 1

for i in range(4):
    pivot = AA[i][i]
    for j in range(8):
        if AA[i][j] != 0:
            AA[i][j] = a_log[(255 + log[AA[i][j] & 0xFF] -
                              log[pivot & 0xFF]) % 255]
    for t in range(4):
        if i != t:
            for j in range(i+1, 8):
                AA[t][j] ^= mul(AA[i][j], AA[t][i])
            AA[t][i] = 0

iG = [[0] * 4 for i in range(4)]

for i in range(4):
    for j in range(4):
        iG[i][j] = AA[i][j + 4]


def mul4(a, bs):
    if a == 0:
        return 0
    rr = 0
    for b in bs:
        rr <<= 8
        if b != 0:
            rr = rr | mul(a, b)
    return rr


T1 = []
T2 = []
T3 = []
T4 = []
T5 = []
T6 = []
T7 = []
T8 = []
U1 = []
U2 = []
U3 = []
U4 = []

for t in range(256):
    s = S[t]
    T1.append(mul4(s, G[0]))
    T2.append(mul4(s, G[1]))
    T3.append(mul4(s, G[2]))
    T4.append(mul4(s, G[3]))

    s = Si[t]
    T5.append(mul4(s, iG[0]))
    T6.append(mul4(s, iG[1]))
    T7.append(mul4(s, iG[2]))
    T8.append(mul4(s, iG[3]))

    U1.append(mul4(t, iG[0]))
    U2.append(mul4(t, iG[1]))
    U3.append(mul4(t, iG[2]))
    U4.append(mul4(t, iG[3]))

# round constants
r_con = [1]
r = 1
for t in range(1, 30):
    r = mul(2, r)
    r_con.append(r)


class PaddingBase:
    def __init__(self, block_size):
        self.block_size = block_size

    def encode(self, source: bytes) -> bytes:  # pragma: nocover
        raise NotImplementedError

    def decode(self, source: bytes) -> bytes:  # pragma: nocover
        raise NotImplementedError


class ZeroPadding(PaddingBase):
    """
    Specified for hashes and MACs as Padding Method 1 in ISO/IEC 10118-1 and ISO/IEC 9797-1.
    """

    def encode(self, source):
        pad_size = self.block_size - \
            ((len(source) + self.block_size - 1) % self.block_size + 1)
        return source + b'\0' * pad_size

    def decode(self, source):
        assert len(source) % self.block_size == 0
        offset = len(source)
        if offset == 0:
            return b''
        end = offset - self.block_size + 1

        while offset > end:
            offset -= 1
            if source[offset]:
                return source[:offset + 1]

        return source[:end]


class Pkcs7Padding(PaddingBase):
    """
    Technique for padding a string as defined in RFC 2315, section 10.3,
    note #2
    """

    def encode(self, source):
        amount_to_pad = self.block_size - (len(source) % self.block_size)
        amount_to_pad = self.block_size if amount_to_pad == 0 else amount_to_pad
        pad = chr(amount_to_pad).encode()
        return source + pad * amount_to_pad

    def decode(self, source):
        return source[:-source[-1]]


class Rijndael:
    def __init__(self, key, block_size: int = 16):

        if block_size not in (16, 24, 32):
            raise ValueError('Invalid block size: %s' % str(block_size))

        if len(key) not in (16, 24, 32):
            raise ValueError('Invalid key size: %s' % str(len(key)))

        self.block_size = block_size
        self.key = key

        rounds = num_rounds[len(key)][block_size]
        b_c = block_size // 4
        # encryption round keys
        k_e = [[0] * b_c for _ in range(rounds + 1)]
        # decryption round keys
        k_d = [[0] * b_c for _ in range(rounds + 1)]
        round_key_count = (rounds + 1) * b_c
        k_c = len(key) // 4

        # copy user material bytes into temporary ints
        tk = []
        for i in range(0, k_c):
            tk.append((ord(key[i * 4:i * 4 + 1]) << 24) | (ord(key[i * 4 + 1:i * 4 + 1 + 1]) << 16) |
                      (ord(key[i * 4 + 2: i * 4 + 2 + 1]) << 8) | ord(key[i * 4 + 3:i * 4 + 3 + 1]))

        # copy values into round key arrays
        t = 0
        j = 0
        while j < k_c and t < round_key_count:
            k_e[t // b_c][t % b_c] = tk[j]
            k_d[rounds - (t // b_c)][t % b_c] = tk[j]
            j += 1
            t += 1
        r_con_pointer = 0
        while t < round_key_count:
            # extrapolate using phi (the round key evolution function)
            tt = tk[k_c - 1]
            tk[0] ^= (S[(tt >> 16) & 0xFF] & 0xFF) << 24 ^ \
                     (S[(tt >> 8) & 0xFF] & 0xFF) << 16 ^ \
                     (S[tt & 0xFF] & 0xFF) << 8 ^ \
                     (S[(tt >> 24) & 0xFF] & 0xFF) ^ \
                     (r_con[r_con_pointer] & 0xFF) << 24
            r_con_pointer += 1
            if k_c != 8:
                for i in range(1, k_c):
                    tk[i] ^= tk[i - 1]
            else:
                for i in range(1, k_c // 2):
                    tk[i] ^= tk[i - 1]
                tt = tk[k_c // 2 - 1]
                tk[k_c // 2] ^= (S[tt & 0xFF] & 0xFF) ^ \
                                (S[(tt >> 8) & 0xFF] & 0xFF) << 8 ^ \
                                (S[(tt >> 16) & 0xFF] & 0xFF) << 16 ^ \
                                (S[(tt >> 24) & 0xFF] & 0xFF) << 24
                for i in range(k_c // 2 + 1, k_c):
                    tk[i] ^= tk[i - 1]
            # copy values into round key arrays
            j = 0
            while j < k_c and t < round_key_count:
                k_e[t // b_c][t % b_c] = tk[j]
                k_d[rounds - (t // b_c)][t % b_c] = tk[j]
                j += 1
                t += 1
        # inverse MixColumn where needed
        for r in range(1, rounds):
            for j in range(b_c):
                tt = k_d[r][j]
                k_d[r][j] = (
                    U1[(tt >> 24) & 0xFF] ^
                    U2[(tt >> 16) & 0xFF] ^
                    U3[(tt >> 8) & 0xFF] ^
                    U4[tt & 0xFF]
                )
        self.Ke = k_e
        self.Kd = k_d

    def encrypt(self, source):

        if len(source) != self.block_size:
            raise ValueError(
                'Wrong block length, expected %s got %s' % (
                    str(self.block_size),
                    str(len(source))
                )
            )

        k_e = self.Ke

        b_c = self.block_size // 4
        rounds = len(k_e) - 1
        if b_c == 4:
            s_c = 0
        elif b_c == 6:
            s_c = 1
        else:
            s_c = 2
        s1 = shifts[s_c][1][0]
        s2 = shifts[s_c][2][0]
        s3 = shifts[s_c][3][0]
        a = [0] * b_c
        # temporary work array
        t = []
        # source to ints + key
        for i in range(b_c):
            t.append((ord(source[i * 4: i * 4 + 1]) << 24 |
                      ord(source[i * 4 + 1: i * 4 + 1 + 1]) << 16 |
                      ord(source[i * 4 + 2: i * 4 + 2 + 1]) << 8 |
                      ord(source[i * 4 + 3: i * 4 + 3 + 1])) ^ k_e[0][i])
        # apply round transforms
        for r in range(1, rounds):
            for i in range(b_c):
                a[i] = (T1[(t[i] >> 24) & 0xFF] ^
                        T2[(t[(i + s1) % b_c] >> 16) & 0xFF] ^
                        T3[(t[(i + s2) % b_c] >> 8) & 0xFF] ^
                        T4[t[(i + s3) % b_c] & 0xFF]) ^ k_e[r][i]
            t = copy.copy(a)
        # last round is special
        result = []
        for i in range(b_c):
            tt = k_e[rounds][i]
            result.append((S[(t[i] >> 24) & 0xFF] ^ (tt >> 24)) & 0xFF)
            result.append(
                (S[(t[(i + s1) % b_c] >> 16) & 0xFF] ^ (tt >> 16)) & 0xFF)
            result.append(
                (S[(t[(i + s2) % b_c] >> 8) & 0xFF] ^ (tt >> 8)) & 0xFF)
            result.append((S[t[(i + s3) % b_c] & 0xFF] ^ tt) & 0xFF)
        out = bytes()
        for xx in result:
            out += bytes([xx])
        return out

    def decrypt(self, cipher):
        if len(cipher) != self.block_size:
            raise ValueError(
                'wrong block length, expected %s got %s' % (
                    str(self.block_size),
                    str(len(cipher))
                )
            )

        k_d = self.Kd
        b_c = self.block_size // 4
        rounds = len(k_d) - 1
        if b_c == 4:
            s_c = 0
        elif b_c == 6:
            s_c = 1
        else:
            s_c = 2
        s1 = shifts[s_c][1][1]
        s2 = shifts[s_c][2][1]
        s3 = shifts[s_c][3][1]
        a = [0] * b_c
        # temporary work array
        t = [0] * b_c
        # cipher to ints + key
        for i in range(b_c):
            t[i] = (ord(cipher[i * 4: i * 4 + 1]) << 24 |
                    ord(cipher[i * 4 + 1: i * 4 + 1 + 1]) << 16 |
                    ord(cipher[i * 4 + 2: i * 4 + 2 + 1]) << 8 |
                    ord(cipher[i * 4 + 3: i * 4 + 3 + 1])) ^ k_d[0][i]
        # apply round transforms
        for r in range(1, rounds):
            for i in range(b_c):
                a[i] = (T5[(t[i] >> 24) & 0xFF] ^
                        T6[(t[(i + s1) % b_c] >> 16) & 0xFF] ^
                        T7[(t[(i + s2) % b_c] >> 8) & 0xFF] ^
                        T8[t[(i + s3) % b_c] & 0xFF]) ^ k_d[r][i]
            t = copy.copy(a)
        # last round is special
        result = []
        for i in range(b_c):
            tt = k_d[rounds][i]
            result.append((Si[(t[i] >> 24) & 0xFF] ^ (tt >> 24)) & 0xFF)
            result.append(
                (Si[(t[(i + s1) % b_c] >> 16) & 0xFF] ^ (tt >> 16)) & 0xFF)
            result.append(
                (Si[(t[(i + s2) % b_c] >> 8) & 0xFF] ^ (tt >> 8)) & 0xFF)
            result.append((Si[t[(i + s3) % b_c] & 0xFF] ^ tt) & 0xFF)
        out = bytes()
        for xx in result:
            out += bytes([xx])
        return out


class RijndaelCbc(Rijndael):

    def __init__(self, key: bytes, iv: bytes, padding: PaddingBase, block_size: int = 16):
        super().__init__(key=key, block_size=block_size)
        self.iv = iv
        self.padding = padding

    def encrypt(self, source: bytes):
        ppt = self.padding.encode(source)
        offset = 0

        ct = bytes()
        v = self.iv
        while offset < len(ppt):
            block = ppt[offset:offset + self.block_size]
            block = self.x_or_block(block, v)
            block = super().encrypt(block)
            ct += block
            offset += self.block_size
            v = block
        return ct

    def decrypt(self, cipher):
        assert len(cipher) % self.block_size == 0
        ppt = bytes()
        offset = 0
        v = self.iv
        while offset < len(cipher):
            block = cipher[offset:offset + self.block_size]
            decrypted = super().decrypt(block)
            ppt += self.x_or_block(decrypted, v)
            offset += self.block_size
            v = block
        pt = self.padding.decode(ppt)
        return pt

    def x_or_block(self, b1, b2):
        i = 0
        r = bytes()
        while i < self.block_size:
            r += bytes([ord(b1[i:i+1]) ^ ord(b2[i:i+1])])
            i += 1
        return r


def rijndael_cbc_decrypt(key, iv, data):
    cbc = RijndaelCbc(
        key,
        iv,
        padding=ZeroPadding(32),
        block_size=32
    )
    return cbc.decrypt(data).decode().split(':')
