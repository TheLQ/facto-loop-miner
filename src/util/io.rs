use crate::surfacev::err::{VError, VResult};
use crate::LOCALE;
use bytemuck::cast_vec;
use memmap2::Mmap;
use num_format::ToFormattedString;
use std::fs::File;
use std::io::{Read, Write};
use std::mem::transmute;
use std::path::Path;

pub const USIZE_BYTES: usize = (usize::BITS / u8::BITS) as usize;

pub fn read_entire_file(path: &Path) -> VResult<Vec<u8>> {
    let mut file = File::open(path).map_err(VError::io_error(path))?;

    // let xy_size = get_usize_vec_length_from_file_size(path, &file)?;
    let xy_size = file.metadata().map_err(VError::io_error(path))?.len();
    let mut big_xy_bytes: Vec<u8> = vec![0; xy_size as usize];

    file.read_to_end(&mut big_xy_bytes)
        .map_err(VError::io_error(path))?;
    Ok(big_xy_bytes)
}

pub fn read_entire_file_aligned_usize(path: &Path) -> VResult<Vec<u8>> {
    let mut file = File::open(path).map_err(VError::io_error(path))?;
    let xy_array_len_u8 = get_file_size(&file, path)? as usize;
    let xy_array_len_u64 = xy_array_len_u8 / 8;

    // let xy_size = get_usize_vec_length_from_file_size(path, &file)?;
    let xy_array_u64: Vec<usize> = vec![0usize; xy_array_len_u64];
    let mut small_xy_bytes: Vec<u8> = cast_vec(xy_array_u64);
    file.read_to_end(&mut small_xy_bytes)
        .map_err(VError::io_error(path))?;
    Ok(cast_vec(small_xy_bytes))
}
#[allow(clippy::unsound_collection_transmute)]
pub fn read_entire_file_transmute_u64(path: &Path) -> VResult<Vec<usize>> {
    let mut file = File::open(path).map_err(VError::io_error(path))?;
    let xy_array_len_u8 = get_file_size(&file, path)? as usize;
    let xy_array_len_u64 = xy_array_len_u8 / 8;

    // let big_xy_bytes: Vec<usize> = vec![0; xy_array_len_u64];
    //
    // let mut small_xy_bytes: Vec<u8> = unsafe { transmute(big_xy_bytes) };
    // unsafe { small_xy_bytes.set_len(xy_array_len_u8) };

    let mut small_xy_bytes: Vec<u8> = vec![0; xy_array_len_u8];

    file.read_to_end(&mut small_xy_bytes)
        .map_err(VError::io_error(path))?;

    let mut big_xy_bytes: Vec<usize> = unsafe { transmute(small_xy_bytes) };
    unsafe { big_xy_bytes.set_len(xy_array_len_u64) };
    Ok(big_xy_bytes)
}

pub fn read_entire_file_mmap(path: &Path) -> VResult<Vec<usize>> {
    let file = File::open(path).map_err(VError::io_error(path))?;
    let xy_array_len_u8 = get_file_size(&file, path)? as usize;
    let xy_array_len_u64 = xy_array_len_u8 / 8;

    let mmap = unsafe { Mmap::map(&file).map_err(VError::io_error(path))? };
    let mut result = vec![0usize; xy_array_len_u64];
    map_u8_to_usize_slice_transmute(&mmap, result.as_mut_slice());
    Ok(result)
}

pub fn get_file_size(file: &File, path: &Path) -> VResult<u64> {
    Ok(file.metadata().map_err(VError::io_error(path))?.len())
}

pub fn map_u8_to_usize_iter(
    input_bytes: impl IntoIterator<Item = u8>,
) -> impl IntoIterator<Item = usize> {
    input_bytes
        .into_iter()
        .array_chunks::<USIZE_BYTES>()
        .map(usize::from_ne_bytes)
}

pub fn map_usize_to_u8_iter(
    input_bytes: impl IntoIterator<Item = usize>,
) -> impl IntoIterator<Item = u8> {
    input_bytes.into_iter().flat_map(usize::to_ne_bytes)
}

pub fn map_usize_to_u8_slice(input: &[usize], output: &mut [u8]) {
    let expected_output_size = input.len() / USIZE_BYTES;
    if output.len() != expected_output_size {
        panic!(
            "outsize {} too small expected {} * {} = {}",
            output.len(),
            input.len(),
            USIZE_BYTES,
            expected_output_size
        );
    }

    for (i, output_chunk) in output.array_chunks_mut::<USIZE_BYTES>().enumerate() {
        output_chunk.clone_from_slice(&input[i].to_ne_bytes());
    }
}

pub fn map_u8_to_usize_slice(input: &[u8], output: &mut [usize]) {
    let expected_output_size = input.len() / USIZE_BYTES;
    if output.len() != expected_output_size {
        panic!(
            "outsize {} too small expected {} * {} = {}",
            output.len(),
            input.len(),
            USIZE_BYTES,
            expected_output_size
        );
    }

    for (i, input_chunk) in input.array_chunks().enumerate() {
        output[i] = usize::from_ne_bytes(*input_chunk);
    }
}

pub fn map_u8_to_usize_slice_transmute(input: &[u8], output: &mut [usize]) {
    let expected_output_size = input.len() / USIZE_BYTES;
    if output.len() != expected_output_size {
        panic!(
            "outsize {} too small expected {} * {} = {}",
            output.len(),
            input.len(),
            USIZE_BYTES,
            expected_output_size
        );
    }

    let mutated_values: &[usize] = todo!(); //unsafe { input.align_to() };
    output.clone_from_slice(mutated_values);
}

pub fn get_usize_vec_length_from_file_size(path: &Path, file: &File) -> VResult<usize> {
    let file_size = file.metadata().map_err(VError::io_error(path))?.len();
    let array_size = file_size / 8;
    Ok(array_size as usize)
}

pub fn write_entire_file(path: &Path, data: &[u8]) -> VResult<()> {
    let mut file = File::create(path).map_err(VError::io_error(path))?;
    file.write_all(data).map_err(VError::io_error(path))?;
    Ok(())
}

pub fn get_mebibytes_of_slice_usize(input: &[usize]) -> String {
    let input_bytes = (input.len() * USIZE_BYTES) as f32 / 1024.0 /*kB*/ / 1024.0 /*mB*/;
    format!("{} MebiBytes", input_bytes)
}
