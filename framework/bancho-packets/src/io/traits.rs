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
        fn osu_write(self, buf: &mut Vec<u8>);
    }

    macro_rules! impl_number {
        ($($t:ty),+) => {
            $(impl OsuWrite for $t {
                #[inline(always)]
                fn osu_write(self, buf: &mut Vec<u8>) {
                    buf.extend(self.to_le_bytes())
                }
            })+
        }
    }

    macro_rules! impl_number_list {
        () => {
            #[inline(always)]
            fn osu_write(self, buf: &mut Vec<u8>) {
                buf.extend((self.len() as u16).to_le_bytes());
                for int in self {
                    buf.extend(int.to_le_bytes())
                }
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
                fn osu_write(self, buf: &mut Vec<u8>) {
                    let byte_length = self.len();
                    if byte_length > 0 {
                        buf.push(11);
                        buf.extend(write_uleb128(byte_length as u32));
                        buf.extend(self.as_bytes());
                    } else {
                        buf.push(0);
                    }
                }
            })+
        }
    }

    impl OsuWrite for u8 {
        #[inline(always)]
        fn osu_write(self, buf: &mut Vec<u8>) {
            buf.push(self);
        }
    }

    impl OsuWrite for Vec<u8> {
        #[inline(always)]
        fn osu_write(mut self, buf: &mut Vec<u8>) {
            buf.append(&mut self);
        }
    }

    impl OsuWrite for &Vec<u8> {
        #[inline(always)]
        fn osu_write(self, buf: &mut Vec<u8>) {
            buf.extend(self);
        }
    }

    impl OsuWrite for bool {
        #[inline(always)]
        fn osu_write(self, buf: &mut Vec<u8>) {
            buf.push(if self { 1 } else { 0 });
        }
    }

    impl_number!(u16, i16, i32, u32, i64, u64, f32, f64);
    impl_number_vec!(u16, i16, i32, u32, i64, u64, f32, f64);
    impl_number_vec_ref!(u16, i16, i32, u32, i64, u64, f32, f64);
    impl_string!(&String, &str, String);
}
