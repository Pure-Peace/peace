use std::convert::TryInto;

pub trait ReadInteger<T> {
    fn from_le_bytes(data: &[u8]) -> Option<T>;
    fn from_be_bytes(data: &[u8]) -> Option<T>;
    fn as_usize(self) -> usize;
}

macro_rules! impl_read_integer {
    ($($t:ty),+) => {
        $(impl ReadInteger<$t> for $t {
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
