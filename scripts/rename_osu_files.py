import os
import hashlib
import time


def file_hash(file_path: str, hash_method) -> str:
    if not os.path.isfile(file_path):
        print(f'{file_path} not a file, skip.')
        return ''
    h = hash_method()
    with open(file_path, 'rb') as f:
        while b := f.read(8192):
            h.update(b)
    return h.hexdigest()


def str_hash(content: str, hash_method, encoding: str = 'UTF-8') -> str:
    return hash_method(content.encode(encoding)).hexdigest()


def file_md5(file_path: str) -> str:
    return file_hash(file_path, hashlib.md5)


def file_sha256(file_path: str) -> str:
    return file_hash(file_path, hashlib.sha256)


def file_sha512(file_path: str) -> str:
    return file_hash(file_path, hashlib.sha512)


def file_sha384(file_path: str) -> str:
    return file_hash(file_path, hashlib.sha384)


def file_sha1(file_path: str) -> str:
    return file_hash(file_path, hashlib.sha1)


def file_sha224(file_path: str) -> str:
    return file_hash(file_path, hashlib.sha224)


def str_md5(content: str, encoding: str = 'UTF-8') -> str:
    return str_hash(content, hashlib.md5, encoding)


def str_sha256(content: str, encoding: str = 'UTF-8') -> str:
    return str_hash(content, hashlib.sha256, encoding)


def str_sha512(content: str, encoding: str = 'UTF-8') -> str:
    return str_hash(content, hashlib.sha512, encoding)


def str_sha384(content: str, encoding: str = 'UTF-8') -> str:
    return str_hash(content, hashlib.sha384, encoding)


def str_sha1(content: str, encoding: str = 'UTF-8') -> str:
    return str_hash(content, hashlib.sha1, encoding)


def str_sha224(content: str, encoding: str = 'UTF-8') -> str:
    return str_hash(content, hashlib.sha224, encoding)


def recalculate_osu_files_md5():
    dir_path = input('input your .osu files dir path here: ')
    print('listing dir...')
    start = time.time()
    file_list = os.listdir()
    file_count = len(file_list)

    print(
        f'done, time spent: {time.time() - start}s; file count: {file_count}; starting...')
    done = 0
    removed = 0
    start = time.time()
    for file_name in file_list:
        full_path = dir_path + file_name
        last_name = os.path.splitext(file_name)[-1]
        md5 = file_md5(full_path)
        if md5:
            try:
                os.rename(full_path, dir_path + md5 + last_name)
                done += 1
            except:
                os.remove(full_path)
                removed += 1
        print(
            f'\rdone {done}/{file_count}, removed: {removed}; time spent: {time.time() - start}s', end='')


if __name__ == '__main__':
    recalculate_osu_files_md5()
