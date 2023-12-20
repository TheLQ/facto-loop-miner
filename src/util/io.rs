use crate::surfacev::err::{VError, VResult};
use crate::LOCALE;
use bytemuck::cast_vec;
use itertools::Itertools;
use memmap2::Mmap;
use num_format::ToFormattedString;
use std::fs::File;
use std::io::{Read, Write};
use std::mem;
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

pub fn read_entire_file_usize_aligned_vec_broken(path: &Path) -> VResult<Vec<usize>> {
    let mut file = File::open(path).map_err(VError::io_error(path))?;
    let xy_array_len_u8 = get_file_size(&file, path)? as usize;
    let xy_array_len_u64 = xy_array_len_u8 / USIZE_BYTES;

    // Create a large Vec, grab an aligned piece of it
    // Docs claim the asserted behavior is expected
    let mut xy_array_u64_raw: Vec<usize> = vec![0usize; xy_array_len_u64];
    let (xy_array_prefix, xy_array_aligned, xy_array_suffix) =
        unsafe { xy_array_u64_raw.align_to_mut::<usize>() };
    assert_eq!(xy_array_prefix.len(), 0, "prefix big");
    assert_eq!(xy_array_suffix.len(), 0, "suffix big");
    assert_eq!(
        xy_array_aligned.len(),
        xy_array_len_u64,
        "aligned not big enough"
    );

    // Build Vec for read_to_end using aligned slice
    let mut xy_vec_aligned_u8 = unsafe {
        Vec::from_raw_parts(
            xy_array_aligned.as_mut_ptr() as *mut u8,
            0,
            xy_array_len_u8 * mem::size_of::<u8>(),
        )
    };
    assert_eq!(
        xy_vec_aligned_u8.capacity(),
        xy_array_len_u8,
        "unexpected original written array length"
    );
    file.read_to_end(&mut xy_vec_aligned_u8)
        .map_err(VError::io_error(path))?;
    assert_eq!(
        xy_vec_aligned_u8.len(),
        xy_array_len_u8,
        "unexpected after written array length"
    );

    // We should return a normally allocated vec clone
    let mut output: Vec<usize> = vec![0; xy_array_aligned.len()];
    output.copy_from_slice(xy_array_aligned);
    println!("wrote array of {}", output.len());
    println!("array sum {}", output.iter().sum::<usize>());
    Ok(output)
}
pub fn read_entire_file_usize_aligned_vec(path: &Path) -> VResult<Vec<usize>> {
    let mut file = File::open(path).map_err(VError::io_error(path))?;
    let xy_array_len_u8 = get_file_size(&file, path)? as usize;
    let xy_array_len_u64 = xy_array_len_u8 / USIZE_BYTES;

    // Allocate result buffer. We will fill the internal capacity
    let mut xy_vec_u64: Vec<usize> = vec![0; xy_array_len_u64];

    // Build u8 Vec viewing the same memory with proper aligned access
    // Docs claim the asserted behavior is expected usually
    let (xy_vec_prefix, xy_vec_aligned, xy_vec_suffix) = unsafe { xy_vec_u64.align_to_mut::<u8>() };
    assert_eq!(xy_vec_prefix.len(), 0, "prefix big");
    assert_eq!(xy_vec_suffix.len(), 0, "suffix big");
    assert_eq!(xy_vec_aligned.len(), xy_array_len_u8, "aligned size");
    let mut xy_vec_aligned_u8: Vec<u8> = unsafe {
        Vec::from_raw_parts(
            xy_vec_aligned.as_mut_ptr(),
            0,
            xy_array_len_u8 * mem::size_of::<u8>(),
        )
    };
    assert_eq!(xy_vec_aligned_u8.capacity(), xy_array_len_u8, "veccapacity");

    file.read_to_end(&mut xy_vec_aligned_u8)
        .map_err(VError::io_error(path))?;
    assert_eq!(xy_vec_aligned_u8.len(), xy_array_len_u8, "vec length");

    // Do not double free. Data is owned by xy_vec_u64
    mem::forget(xy_vec_aligned_u8);

    println!("wrote array of {}", xy_vec_u64.len());
    println!("array sum {}", xy_vec_u64.iter().sum::<usize>());

    Ok(xy_vec_u64)
}

#[cfg(lol)]
pub unsafe fn read_entire_file_usize_aligned_vec_golfed(path: &Path) -> VResult<Vec<usize>> {
    let mut file = File::open(path).map_err(VError::io_error(path))?;
    let xy_array_len_u8 = get_file_size(&file, path)? as usize;

    let mut xy_vec_u64: Vec<usize> = vec![0; xy_array_len_u8 / USIZE_BYTES];
    let (_, xy_vec_aligned, _) = xy_vec_u64.align_to_mut::<u8>();
    assert_eq!(xy_vec_aligned.len(), xy_array_len_u8, "aligned size");
    let mut xy_vec_aligned_u8 = Vec::from_raw_parts(
        xy_vec_aligned.as_mut_ptr(),
        0,
        xy_array_len_u8 * mem::size_of::<u8>(),
    );
    file.read_to_end(&mut xy_vec_aligned_u8)
        .map_err(VError::io_error(path))?;
    mem::forget(xy_vec_aligned_u8);

    Ok(xy_vec_u64)
}

#[allow(clippy::unsound_collection_transmute)]
pub fn read_entire_file_usize_transmute_broken(path: &Path) -> VResult<Vec<usize>> {
    let mut file = File::open(path).map_err(VError::io_error(path))?;
    let xy_array_len_u8 = get_file_size(&file, path)? as usize;
    let xy_array_len_u64 = xy_array_len_u8 / 8;

    let big_xy_bytes: Vec<usize> = vec![0; xy_array_len_u64];

    let mut small_xy_bytes: Vec<u8> = unsafe { transmute(big_xy_bytes) };
    unsafe { small_xy_bytes.set_len(xy_array_len_u8) };

    // let mut small_xy_bytes: Vec<u8> = vec![0; xy_array_len_u8];

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
    map_u8_to_usize_slice(&mmap, result.as_mut_slice());
    Ok(result)
}

pub fn get_file_size(file: &File, path: &Path) -> VResult<u64> {
    Ok(file.metadata().map_err(VError::io_error(path))?.len())
}

pub fn get_file_size_u8_and_u64(file: &File, path: &Path) -> VResult<(usize, usize)> {
    let size = file.metadata().map_err(VError::io_error(path))?.len();
    Ok((size as usize, size as usize / USIZE_BYTES))
}

pub fn map_u8_to_usize_iter(
    input_bytes: impl IntoIterator<Item = u8>,
) -> impl IntoIterator<Item = usize> {
    input_bytes
        .into_iter()
        .array_chunks()
        .map(usize::from_ne_bytes)
}

pub fn map_u8_to_usize_iter_ref<'a>(
    input_bytes: impl IntoIterator<Item = &'a u8> + 'a,
) -> impl IntoIterator<Item = usize> + 'a {
    input_bytes
        .into_iter()
        .array_chunks()
        .map(|e| e.map(|b| *b))
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

// pub fn map_u8_to_usize_slice_transmute_vec(input: &[u8]) -> Vec<usize> {
//     let expected_output_size = input.len() / USIZE_BYTES;
//     let mut output = vec![0; expected_output_size];
//
//     // let mutated_values: &[usize] = unsafe { input.align_to() };
//     output.clone_from_slice(mutated_values);
//
//     output
// }

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
