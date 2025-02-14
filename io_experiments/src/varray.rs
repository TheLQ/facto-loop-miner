use crate::read_entire_file_varray_mmap_lib;
use memmap2::MmapMut;
use std::fs::File;
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};

enum BackingMemory {
    RegularOldeVec {
        vec: Vec<usize>,
        is_dirty: bool,
    },
    Mmap {
        mmap: MmapMut,
        vec: ManuallyDrop<Vec<usize>>,
        backing_file: File,
        backing_path: PathBuf,
        is_dirty: bool,
    },
}

impl Default for BackingMemory {
    fn default() -> Self {
        BackingMemory::RegularOldeVec {
            vec: Vec::new(),
            is_dirty: false,
        }
    }
}

pub const EMPTY_XY_INDEX: usize = usize::MAX;
pub const CACHED_MMAP_PATH_PREFIX: &str = "recached_mmap";

// Light wrapper around Vec. Memory map backed Vec's must live as long as the Mmap
#[derive(Default)]
pub struct VArray {
    inner: BackingMemory,
}

impl VArray {
    pub fn new_length(size: usize) -> Self {
        VArray {
            inner: BackingMemory::RegularOldeVec {
                vec: vec![EMPTY_XY_INDEX; size],
                is_dirty: false,
            },
        }
    }

    pub fn from_mmap(
        path: &Path,
        file: File,
        backing_memory_map: MmapMut,
        xy_to_entity: ManuallyDrop<Vec<usize>>,
    ) -> Self {
        VArray {
            inner: BackingMemory::Mmap {
                mmap: backing_memory_map,
                vec: xy_to_entity,
                backing_file: file,
                backing_path: path.to_path_buf(),
                is_dirty: false,
            },
        }
    }

    pub fn as_slice(&self) -> &[usize] {
        match &self.inner {
            BackingMemory::RegularOldeVec { vec, .. } => vec.as_slice(),
            BackingMemory::Mmap { vec, .. } => vec.as_slice(),
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [usize] {
        match &mut self.inner {
            BackingMemory::RegularOldeVec { vec, .. } => vec.as_mut_slice(),
            BackingMemory::Mmap { vec, is_dirty, .. } => {
                *is_dirty = true;
                vec.as_mut_slice()
            }
        }
    }

    #[allow(clippy::len_without_is_empty)] // this is never empty
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    pub fn cache_buffers_for_cloning(&mut self) {
        // match &self.inner {
        //     BackingMemory::RegularOldeVec { .. } => {}
        //     BackingMemory::Mmap { path, .. } => {
        //         let temp_file = tempfile::tempfile().unwrap();
        //         let new_path = if path.starts_with(CACHED_MMAP_PATH_PREFIX) {
        //             // reconstruct path with
        //             let mut path_parts = path.components();
        //             let mut new_path = PathBuf::new();
        //             new_path.push(path_parts.next().unwrap());
        //
        //             let count_raw = path_parts.next().unwrap().as_os_str();
        //             let count = usize::from_str(&count_raw.to_string_lossy()).unwrap() + 1;
        //             new_path.push(count.to_string());
        //             for part in path_parts {
        //                 new_path.push(part);
        //             }
        //             new_path
        //         } else {
        //             let mut new_path = PathBuf::new();
        //             new_path.push(CACHED_MMAP_PATH_PREFIX);
        //             new_path.push("0");
        //             new_path.join(path)
        //         };
        //
        //         let new_array =
        //             read_entire_file_varray_mmap_lib_for_file(&new_path, temp_file).unwrap();
        //         self.inner = new_array.inner;
        //     }
        // }
    }
}

impl Clone for VArray {
    fn clone(&self) -> Self {
        match &self.inner {
            BackingMemory::RegularOldeVec { vec, is_dirty } => {
                unimplemented!("does anything actually do this?")
                // if *is_dirty {
                //     panic!("Already dirty regular vec from mmap");
                // }
                // VArray {
                //     inner: BackingMemory::RegularOldeVec {
                //         vec: Vec::clone(vec),
                //         // stay false, safe to clone
                //         is_dirty: false,
                //     },
                // }
            }
            BackingMemory::Mmap {
                mmap,
                vec,
                backing_file,
                backing_path,
                is_dirty,
            } => {
                // if *is_dirty {
                //     panic!("Cannot clone dirty mmap");
                // }
                // read_entire_file_varray_mmap_lib(backing_path).unwrap()
                VArray {
                    inner: BackingMemory::RegularOldeVec {
                        vec: Vec::clone(vec),
                        // stay false, safe to clone
                        is_dirty: false,
                    },
                }
            }
        }
    }
}
