mod reader;
mod writer;

pub mod traits;

pub use reader::*;
pub use writer::*;

#[inline(always)]
/// Unsigned to uleb128
pub fn write_uleb128(mut unsigned: u32) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity(2);
    while unsigned >= 0x80 {
        data.push(((unsigned & 0x7f) | 0x80) as u8);
        unsigned >>= 7;
    }
    data.push(unsigned as u8);
    data
}

#[inline(always)]
pub fn read_uleb128(slice: &[u8]) -> Option<(u32, usize)> {
    let (mut val, mut shift, mut index) = (0, 0, 0);
    loop {
        let byte = slice.get(index)?;
        index += 1;
        if (byte & 0x80) == 0 {
            val |= (*byte as u32) << shift;
            return Some((val, index));
        }
        val |= ((byte & 0x7f) as u32) << shift;
        shift += 7;
    }
}
