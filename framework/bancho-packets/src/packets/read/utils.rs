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
