use memmap2::MmapMut;
use std::fs::File;
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};

enum BackingMemory {
    RegularOldeVec {
        data: Box<[usize]>,
        is_dirty: bool,
    },
    Mmap {
        mmap: MmapMut,
        /// Owned by mmap
        data: ManuallyDrop<Box<[usize]>>,
        backing_path: PathBuf,
        is_dirty: bool,
    },
}

pub const EMPTY_XY_INDEX: usize = usize::MAX;

/// Gigantic memory-map or Slice backed storage
#[derive(Default)]
pub struct VArray {
    inner: BackingMemory,
}

impl VArray {
    pub fn new_length(size: usize) -> Self {
        VArray {
            inner: BackingMemory::RegularOldeVec {
                data: vec![EMPTY_XY_INDEX; size].into_boxed_slice(),
                is_dirty: false,
            },
        }
    }

    pub fn from_mmap(
        path: &Path,
        file: File,
        backing_memory_map: MmapMut,
        xy_to_entity: ManuallyDrop<Box<[usize]>>,
    ) -> Self {
        VArray {
            inner: BackingMemory::Mmap {
                mmap: backing_memory_map,
                data: xy_to_entity,
                backing_path: path.to_path_buf(),
                is_dirty: false,
            },
        }
    }

    pub fn as_slice(&self) -> &[usize] {
        match &self.inner {
            BackingMemory::RegularOldeVec { data, .. } => data,
            BackingMemory::Mmap { data, .. } => data,
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [usize] {
        match &mut self.inner {
            BackingMemory::RegularOldeVec { data, .. } => data,
            BackingMemory::Mmap { data, is_dirty, .. } => {
                *is_dirty = true;
                data
            }
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
            BackingMemory::RegularOldeVec {
                data: vec,
                is_dirty,
            } => {
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
                data,
                backing_path,
                is_dirty,
            } => {
                // if *is_dirty {
                //     panic!("Cannot clone dirty mmap");
                // }
                // read_entire_file_varray_mmap_lib(backing_path).unwrap()
                VArray {
                    inner: BackingMemory::RegularOldeVec {
                        data: ManuallyDrop::into_inner(data.clone()),
                        is_dirty: false,
                    },
                }
            }
        }
    }
}

// purely for serde deserialize
impl Default for BackingMemory {
    fn default() -> Self {
        BackingMemory::RegularOldeVec {
            data: Vec::new().into_boxed_slice(),
            is_dirty: false,
        }
    }
}
