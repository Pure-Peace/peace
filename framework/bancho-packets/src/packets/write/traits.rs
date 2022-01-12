pub trait OsuWrite {
    fn osu_write(self) -> Vec<u8>;
}

macro_rules! impl_number {
    ($($t:ty),+) => {
        $(impl OsuWrite for $t {
            #[inline(always)]
            fn osu_write(self) -> Vec<u8> {
                self.to_le_bytes().into()
            }
        })+
    }
}

impl_number!(u8, u16, i16, i32, u32, i64, f32);

/// Number vec
impl<W> OsuWrite for &Vec<W>
where
    W: OsuWrite + Copy,
{
    #[inline(always)]
    fn osu_write(self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(self.len() + 2);
        ret.extend((self.len() as u16).to_le_bytes());
        ret.extend(self.iter().map(|i| i.osu_write()).collect());
        ret
    }
}

impl OsuWrite for Vec<u8> {
    #[inline(always)]
    fn osu_write(mut self) -> Self {
        let mut ret = Vec::with_capacity(self.len() + 2);
        ret.extend((self.len() as u16).to_le_bytes());
        ret.append(&mut self);
        ret
    }
}

macro_rules! impl_string {
    ($($t:ty),+) => {
        $(impl OsuWrite for $t {
            #[inline(always)]
            fn osu_write(self) -> Vec<u8> {
                let byte_length = self.len();
                let mut data: Vec<u8> = Vec::with_capacity(byte_length + 3);
                if byte_length > 0 {
                    data.push(11);
                    data.extend(super::utils::write_uleb128(byte_length as u32));
                    data.extend(self.as_bytes());
                } else {
                    data.push(0);
                }
                data
            }
        })+
    }
}

impl_string!(&String, &str);

impl OsuWrite for bool {
    #[inline(always)]
    fn osu_write(self) -> Vec<u8> {
        if self {
            vec![1]
        } else {
            vec![0]
        }
    }
}
