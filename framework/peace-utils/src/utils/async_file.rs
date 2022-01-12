use tokio::io;
use md5::{Digest, Md5};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

#[inline(always)]
pub fn calc_file_md5<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let file = File::open(path)?;
    let mut hasher = Md5::new();
    let mut buffer = [0; 8192];
    let mut reader = BufReader::new(file);
    while let Ok(size) = reader.read(&mut buffer) {
        if size == 0 {
            break;
        };
        hasher.update(&buffer[..size]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
