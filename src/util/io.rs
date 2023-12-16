use crate::surfacev::err::{VError, VResult};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub fn read_entire_file(path: &Path) -> VResult<Vec<u8>> {
    let mut file = File::open(path).map_err(VError::io_error(path))?;
    let mut big_xy_bytes: Vec<u8> = Vec::new();
    file.read_to_end(&mut big_xy_bytes)
        .map_err(VError::io_error(path))?;
    Ok(big_xy_bytes)
}

pub fn write_entire_file(path: &Path, data: &[u8]) -> VResult<()> {
    let mut file = File::create(path).map_err(VError::io_error(path))?;
    file.write_all(data).map_err(VError::io_error(path))?;
    Ok(())
}
