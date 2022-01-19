pub mod reading {
    use std::convert::TryInto;

    pub trait NumberAsBytes<T> {
        fn from_le_bytes(data: &[u8]) -> Option<T>;
        fn from_be_bytes(data: &[u8]) -> Option<T>;
        fn as_usize(self) -> usize;
    }

    macro_rules! impl_read_integer {
        ($($t:ty),+) => {
            $(impl NumberAsBytes<$t> for $t {
                #[inline(always)]
                fn from_le_bytes(data: &[u8]) -> Option<$t> {
                    Some(<$t>::from_le_bytes(match data.try_into() {
                        Ok(d) => d,
                        Err(_) => {
                            return None;
                        }
                    }))
                }
                #[inline(always)]
                fn from_be_bytes(data: &[u8]) -> Option<$t> {
                    Some(<$t>::from_be_bytes(match data.try_into() {
                        Ok(d) => d,
                        Err(_) => {
                            return None;
                        }
                    }))
                }
                #[inline(always)]
                fn as_usize(self) -> usize {
                    self as usize
                }
            })+
        }
    }

    impl_read_integer!(i8, u8, i16, u16, i32, u32, i64, u64);
}

pub mod writing {
    use crate::io::utils::write_uleb128;

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

    macro_rules! impl_number_list {
        () => {
            #[inline(always)]
            fn osu_write(self) -> Vec<u8> {
                let mut ret = Vec::with_capacity(self.len() + 2);
                ret.extend((self.len() as u16).to_le_bytes());
                for int in self {
                    ret.append(&mut int.osu_write());
                }
                ret
            }
        };
    }

    macro_rules! impl_number_vec {
        ($($t:ty),+) => {$(impl OsuWrite for Vec<$t> {impl_number_list!();})+}
    }

    macro_rules! impl_number_vec_ref {
        ($($t:ty),+) => {$(impl OsuWrite for &Vec<$t> {impl_number_list!();})+}
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
                        data.extend(write_uleb128(byte_length as u32));
                        data.extend(self.as_bytes());
                    } else {
                        data.push(0);
                    }
                    data
                }
            })+
        }
    }

    impl OsuWrite for u8 {
        #[inline(always)]
        fn osu_write(self) -> Vec<u8> {
            vec![self]
        }
    }

    impl OsuWrite for Vec<u8> {
        #[inline(always)]
        fn osu_write(self) -> Vec<u8> {
            self
        }
    }

    impl OsuWrite for &Vec<u8> {
        #[inline(always)]
        fn osu_write(self) -> Vec<u8> {
            self.clone()
        }
    }

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

    impl_number!(u16, i16, i32, u32, i64, u64, f32, f64);
    impl_number_vec!(u16, i16, i32, u32, i64, u64, f32, f64);
    impl_number_vec_ref!(u16, i16, i32, u32, i64, u64, f32, f64);
    impl_string!(&String, &str, String);
}
