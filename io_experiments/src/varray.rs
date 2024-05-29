use crate::read_entire_file_varray_mmap_lib;
use libc::mmap;
use memmap2::MmapMut;
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};

enum BackingMemory {
    RegularOldeVec {
        vec: Vec<usize>,
    },
    Mmap {
        mmap: MmapMut,
        vec: ManuallyDrop<Vec<usize>>,
        path: PathBuf,
    },
}

impl Default for BackingMemory {
    fn default() -> Self {
        BackingMemory::RegularOldeVec { vec: Vec::new() }
    }
}

pub const EMPTY_XY_INDEX: usize = usize::MAX;

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
            },
        }
    }

    pub fn from_mmap(
        path: &Path,
        backing_memory_map: MmapMut,
        xy_to_entity: ManuallyDrop<Vec<usize>>,
    ) -> Self {
        VArray {
            inner: BackingMemory::Mmap {
                mmap: backing_memory_map,
                vec: xy_to_entity,
                path: path.to_path_buf(),
            },
        }
    }

    pub fn as_slice(&self) -> &[usize] {
        match &self.inner {
            BackingMemory::RegularOldeVec { vec } => vec.as_slice(),
            BackingMemory::Mmap { vec, .. } => vec.as_slice(),
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [usize] {
        match &mut self.inner {
            BackingMemory::RegularOldeVec { vec } => vec.as_mut_slice(),
            BackingMemory::Mmap { vec, .. } => vec.as_mut_slice(),
        }
    }

    #[allow(clippy::len_without_is_empty)] // this is never empty
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }
}

impl Clone for VArray {
    fn clone(&self) -> Self {
        match &self.inner {
            BackingMemory::RegularOldeVec { vec } => VArray {
                inner: BackingMemory::RegularOldeVec {
                    vec: Vec::clone(vec),
                },
            },
            BackingMemory::Mmap { mmap, vec, path } => {
                VArray {
                    inner: BackingMemory::RegularOldeVec {
                        vec: Vec::clone(vec),
                    },
                }
                // read_entire_file_varray_mmap_lib(path).unwrap()
            }
        }
    }
}
