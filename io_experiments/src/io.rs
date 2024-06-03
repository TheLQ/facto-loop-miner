use std::fs::File;
use std::io::{Read, Write};
use std::mem::{transmute, ManuallyDrop};
use std::os::fd::AsRawFd;
use std::path::Path;
use std::{io, mem, ptr, slice};

use crate::err::{VIoError, VIoResult};
use crate::varray::VArray;
use libc::munmap;
use memmap2::{Mmap, MmapOptions};
use tracing::{debug, info};

pub const USIZE_BYTES: usize = (usize::BITS / u8::BITS) as usize;

pub fn read_entire_file(path: &Path, preallocate_vec: bool) -> VIoResult<Vec<u8>> {
    let mut file = File::open(path).map_err(VIoError::io_error(path))?;
    let xy_array_len_u8 = get_file_size(&file, path)? as usize;
    let mut xy_array_u8_raw: Vec<u8> = if preallocate_vec {
        Vec::with_capacity(xy_array_len_u8)
    } else {
        Vec::new()
    };
    file.read_to_end(&mut xy_array_u8_raw)
        .map_err(VIoError::io_error(path))?;
    assert_eq!(xy_array_u8_raw.len(), xy_array_len_u8);
    Ok(xy_array_u8_raw)
}

#[cfg(feature = "lol")]
pub fn read_entire_file_usize_aligned_vec_broken(path: &Path) -> VIoResult<Vec<usize>> {
    let mut file = File::open(path).map_err(VIoError::io_error(path))?;
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
        Vec::from_raw_parts(xy_array_aligned.as_mut_ptr() as *mut u8, 0, xy_array_len_u8)
    };
    assert_eq!(
        xy_vec_aligned_u8.capacity(),
        xy_array_len_u8,
        "unexpected original written array length"
    );
    file.read_to_end(&mut xy_vec_aligned_u8)
        .map_err(VIoError::io_error(path))?;
    assert_eq!(
        xy_vec_aligned_u8.len(),
        xy_array_len_u8,
        "unexpected after written array length"
    );

    // We should return a normally allocated vec clone
    let mut output: Vec<usize> = vec![0; xy_array_aligned.len()];
    if output.len() != xy_array_aligned.len() {
        return Err(VIoError::IoUring_CqeCopyFailed {
            source_size: xy_array_aligned.len(),
            target_size: output.len(),
            backtrace: Backtrace::capture(),
        });
    }
    output.copy_from_slice(xy_array_aligned);
    println!("wrote array of {}", output.len());
    println!("array sum {}", output.iter().sum::<usize>());
    Ok(output)
}

/// DOC
pub fn read_entire_file_usize_aligned_vec(path: &Path) -> VIoResult<Vec<usize>> {
    let mut file = File::open(path).map_err(VIoError::io_error(path))?;
    let xy_array_len_u8 = get_file_size(&file, path)? as usize;
    let xy_array_len_u64 = xy_array_len_u8 / USIZE_BYTES;

    // Allocate result buffer. We will fill the internal capacity
    let mut xy_vec_u64: Vec<usize> = vec![0; xy_array_len_u64];

    // Build u8 Vec viewing the same memory with proper aligned access
    // Docs state the outer slices should be empty in real world environments
    let (xy_vec_prefix, xy_vec_aligned, xy_vec_suffix) = unsafe { xy_vec_u64.align_to_mut::<u8>() };
    assert_eq!(xy_vec_prefix.len(), 0, "prefix big");
    assert_eq!(xy_vec_suffix.len(), 0, "suffix big");
    assert_eq!(xy_vec_aligned.len(), xy_array_len_u8, "aligned size");
    let mut xy_vec_aligned_u8: Vec<u8> =
        unsafe { Vec::from_raw_parts(xy_vec_aligned.as_mut_ptr(), 0, xy_array_len_u8) };
    assert_eq!(xy_vec_aligned_u8.capacity(), xy_array_len_u8, "veccapacity");

    file.read_to_end(&mut xy_vec_aligned_u8)
        .map_err(VIoError::io_error(path))?;
    assert_eq!(xy_vec_aligned_u8.len(), xy_array_len_u8, "vec length");

    // Do not double free memory owned by xy_vec_u64
    mem::forget(xy_vec_aligned_u8);

    println!("wrote array of {}", xy_vec_u64.len());
    println!("array sum {}", xy_vec_u64.iter().sum::<usize>());

    Ok(xy_vec_u64)
}

/// DOC
pub fn read_entire_file_usize_mmap_custom(
    path: &Path,
    populate: bool,
    sequential: bool,
    willneed: bool,
) -> VIoResult<ManuallyDrop<Vec<usize>>> {
    let file = File::open(path).map_err(VIoError::io_error(path))?;
    let file_size = get_file_size(&file, path)? as usize;
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) as usize };
    let alignment_padding = page_size - (file_size % page_size);

    let xy_array_len_u8 = file_size;
    let xy_array_len_u64 = xy_array_len_u8 / USIZE_BYTES;
    let xy_array_len_aligned_u8 = file_size + alignment_padding;
    let xy_array_len_aligned_u64 = xy_array_len_aligned_u8 / USIZE_BYTES;

    info!("hello mmap");
    let vec: Vec<usize> = unsafe {
        info!("starting mmap");
        let mmap_ptr = libc::mmap(
            ptr::null_mut(),
            xy_array_len_aligned_u8,
            // ACL required to use it
            libc::PROT_READ | libc::PROT_WRITE,
            // TODO: libc::MAP_HUGETLB | libc::MAP_HUGE_2MB
            // Required mode, Prepopulate with file content
            libc::MAP_PRIVATE | enable_if(libc::MAP_POPULATE, populate),
            // SAFETY file can be closed immediately
            file.as_raw_fd(),
            0,
        );
        if mmap_ptr == libc::MAP_FAILED {
            panic!("failed to mmap {}", xy_array_len_u8);
        }
        info!("mmapped");

        if (sequential || willneed)
            && libc::madvise(
                mmap_ptr,
                xy_array_len_aligned_u8,
                enable_if(libc::MADV_SEQUENTIAL, sequential)
                    | enable_if(libc::MADV_WILLNEED, willneed),
            ) != libc::EXIT_SUCCESS
        {
            panic!("madvise failed {}", io::Error::last_os_error());
        }

        let xy_array_u8 = slice::from_raw_parts_mut(mmap_ptr, xy_array_len_u8);

        // Build usize Vec viewing the same memory with proper aligned access
        // Docs state the outer slices should be empty in real world environments
        let (xy_array_prefix, xy_array_aligned, xy_array_suffix) =
            xy_array_u8.align_to_mut::<usize>();
        assert_eq!(xy_array_prefix.len(), 0, "prefix big");
        assert_eq!(xy_array_suffix.len(), 0, "suffix big");
        assert_eq!(xy_array_aligned.len(), xy_array_len_u64, "aligned size");
        let xy_vec_aligned_usize: Vec<usize> = Vec::from_raw_parts(
            xy_array_aligned.as_mut_ptr(),
            xy_array_len_u64,
            xy_array_len_aligned_u64,
        );

        xy_vec_aligned_usize
    };

    // println!("wrote array of {}", vec.len());
    // println!("array sum {}", vec.iter().sum::<usize>());
    Ok(ManuallyDrop::new(vec))
}

fn enable_if(value: libc::c_int, enable: bool) -> libc::c_int {
    if enable {
        value
    } else {
        0
    }
}

/// Must Drop with munmap() not the normal free()
pub fn drop_mmap_vec(mut mmap_vec: ManuallyDrop<Vec<usize>>) {
    unsafe {
        let res = munmap(
            mmap_vec.as_mut_slice().as_mut_ptr() as *mut libc::c_void,
            mmap_vec.capacity() * USIZE_BYTES,
        );
        if res != 0 {
            panic!("munmap failed {}", io::Error::from_raw_os_error(-res));
        }
    }
}

#[cfg(feature = "lol")]
pub fn read_entire_file_usize_memmap_u8(path: &Path) -> VIoResult<Vec<usize>> {
    let file = File::open(path).map_err(VIoError::io_error(path))?;

    let vec = unsafe {
        let xy_array_len_u8 = get_file_size(&file, path)? as usize;
        let xy_array_len_u64 = xy_array_len_u8 / USIZE_BYTES;
        println!("mmap");
        // PROT_READ | PROT_WRITE = Basic rw memory usage
        // MAP_PRIVATE = Required mode
        // MAP_POPULATE = Prepoplate file
        let ptr = libc::mmap(
            ptr::null_mut(),
            xy_array_len_u8 * mem::size_of::<u8>(),
            libc::PROT_READ | libc::PROT_WRITE,
            // libc::MAP_HUGETLB | libc::MAP_HUGE_2MB
            libc::MAP_PRIVATE | libc::MAP_POPULATE,
            // SAFETY this can be closed immediately
            file.as_raw_fd(),
            0,
        );
        if ptr == libc::MAP_FAILED {
            panic!("failed to mmap {}", xy_array_len_u8);
        }

        if libc::madvise(
            ptr,
            xy_array_len_u8,
            libc::MADV_SEQUENTIAL | libc::MADV_WILLNEED,
        ) != libc::EXIT_SUCCESS
        {
            panic!("failed to madvise");
        }

        Vec::from_raw_parts(ptr as *mut usize, xy_array_len_u64, xy_array_len_u64)
    };

    // let vec
    println!("wrote array of {}", vec.len());
    println!("array sum {}", vec.iter().sum::<usize>());
    Ok(vec)
}

#[cfg(feature = "lol")]
pub unsafe fn read_entire_file_usize_aligned_vec_golfed(path: &Path) -> VIoResult<Vec<usize>> {
    let mut file = File::open(path).map_err(VIoError::io_error(path))?;
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
        .map_err(VIoError::io_error(path))?;
    mem::forget(xy_vec_aligned_u8);

    Ok(xy_vec_u64)
}

#[allow(clippy::unsound_collection_transmute)]
pub fn read_entire_file_usize_transmute_broken(path: &Path) -> VIoResult<Vec<usize>> {
    let mut file = File::open(path).map_err(VIoError::io_error(path))?;
    let xy_array_len_u8 = get_file_size(&file, path)? as usize;
    let xy_array_len_u64 = xy_array_len_u8 / 8;

    let big_xy_bytes: Vec<usize> = vec![0; xy_array_len_u64];

    let mut small_xy_bytes: Vec<u8> = unsafe { transmute(big_xy_bytes) };
    unsafe { small_xy_bytes.set_len(xy_array_len_u8) };

    // let mut small_xy_bytes: Vec<u8> = vec![0; xy_array_len_u8];

    file.read_to_end(&mut small_xy_bytes)
        .map_err(VIoError::io_error(path))?;

    let mut big_xy_bytes: Vec<usize> = unsafe { transmute(small_xy_bytes) };
    unsafe { big_xy_bytes.set_len(xy_array_len_u64) };
    Ok(big_xy_bytes)
}

pub fn read_entire_file_varray_mmap_lib(path: &Path) -> VIoResult<VArray> {
    let file = File::open(path).map_err(VIoError::io_error(path))?;
    let xy_array_len_u8 = get_file_size(&file, path)? as usize;
    let xy_array_len_u64 = xy_array_len_u8 / USIZE_BYTES;

    debug!("mmapping...");
    let mut mmap = unsafe {
        MmapOptions::new()
            // .populate()
            .map_copy(&file)
            .map_err(VIoError::io_error(path))?
    };
    debug!("mapped {}", path.display());

    // View mmap as a Vec
    let xy_array_u8 = unsafe { slice::from_raw_parts_mut(mmap.as_mut_ptr(), xy_array_len_u8) };

    debug!("from raw parts {}", path.display());

    // Build usize Vec viewing the same memory with proper aligned access
    // Docs state the outer slices should be empty in real world environments
    let (xy_array_prefix, xy_array_aligned, xy_array_suffix) =
        unsafe { xy_array_u8.align_to_mut::<usize>() };
    assert_eq!(xy_array_prefix.len(), 0, "prefix big");
    assert_eq!(xy_array_suffix.len(), 0, "suffix big");
    assert_eq!(xy_array_aligned.len(), xy_array_len_u64, "aligned size");

    let xy_array_u64 = unsafe {
        ManuallyDrop::new(Vec::from_raw_parts(
            xy_array_aligned.as_mut_ptr(),
            xy_array_len_u64,
            xy_array_len_u64,
        ))
    };

    debug!("reviewed {}", path.display());

    Ok(VArray::from_mmap(path, file, mmap, xy_array_u64))
}

pub fn read_entire_file_mmap_copy(path: &Path) -> VIoResult<Vec<usize>> {
    let file = File::open(path).map_err(VIoError::io_error(path))?;
    let xy_array_len_u8 = get_file_size(&file, path)? as usize;
    let xy_array_len_u64 = xy_array_len_u8 / USIZE_BYTES;

    let result = unsafe {
        let mmap = Mmap::map(&file).map_err(VIoError::io_error(path))?;

        // Build usize Vec viewing the same memory with proper aligned access
        // Docs state the outer slices should be empty in real world environments
        let (xy_vec_prefix, xy_vec_aligned, xy_vec_suffix) = mmap.align_to::<usize>();
        assert_eq!(xy_vec_prefix.len(), 0, "prefix big");
        assert_eq!(xy_vec_suffix.len(), 0, "suffix big");
        assert_eq!(xy_vec_aligned.len(), xy_array_len_u64, "aligned size");

        let mut result = vec![0usize; xy_array_len_u64];
        result.copy_from_slice(xy_vec_aligned);
        result
    };

    Ok(result)
}

pub fn get_file_size(file: &File, path: &Path) -> VIoResult<u64> {
    Ok(file.metadata().map_err(VIoError::io_error(path))?.len())
}

pub fn get_file_size_u8_and_u64(file: &File, path: &Path) -> VIoResult<(usize, usize)> {
    let size = file.metadata().map_err(VIoError::io_error(path))?.len();
    Ok((size as usize, size as usize / USIZE_BYTES))
}

pub fn map_u8_to_usize_iter(
    input_bytes: impl IntoIterator<Item = u8>,
) -> impl Iterator<Item = usize> {
    input_bytes
        .into_iter()
        .array_chunks()
        .map(usize::from_ne_bytes)
}

pub fn map_u8_to_usize_iter_ref<'a>(
    input_bytes: impl IntoIterator<Item = &'a u8> + 'a,
) -> impl Iterator<Item = usize> + 'a {
    input_bytes
        .into_iter()
        .array_chunks()
        .map(|e| e.map(|b| *b))
        .map(usize::from_ne_bytes)
}

pub fn map_usize_to_u8_iter(
    input_bytes: impl IntoIterator<Item = usize>,
) -> impl Iterator<Item = u8> {
    input_bytes.into_iter().flat_map(usize::to_ne_bytes)
}

pub fn map_usize_to_u8_slice(input: &[usize], output: &mut [u8]) {
    assert_eq!(input.len() * USIZE_BYTES, output.len(), "outsize too small",);
    for (i, output_chunk) in output.array_chunks_mut::<USIZE_BYTES>().enumerate() {
        output_chunk.clone_from_slice(&input[i].to_ne_bytes());
    }
}

pub fn map_u8_to_usize_slice(input: &[u8], output: &mut [usize]) {
    assert_eq!(input.len() / USIZE_BYTES, output.len(), "outsize too small");
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

pub fn get_usize_vec_length_from_file_size(path: &Path, file: &File) -> VIoResult<usize> {
    let file_size = file.metadata().map_err(VIoError::io_error(path))?.len();
    let array_size = file_size / 8;
    Ok(array_size as usize)
}

pub fn write_entire_file(path: &Path, data: &[u8]) -> VIoResult<()> {
    let mut file = File::create(path).map_err(VIoError::io_error(path))?;
    file.write_all(data).map_err(VIoError::io_error(path))?;
    Ok(())
}

pub fn get_mebibytes_of_slice_usize(input: &[usize]) -> String {
    let input_bytes = (input.len() * USIZE_BYTES) as f32 / 1024.0 /*kB*/ / 1024.0 /*mB*/;
    format!("{} MebiBytes", input_bytes)
}
