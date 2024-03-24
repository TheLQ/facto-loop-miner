use memmap2::MmapMut;
use std::mem::ManuallyDrop;

enum BackingMemory {
    RegularOldeVec(Vec<usize>),
    Mmap(MmapMut, ManuallyDrop<Vec<usize>>),
}

impl Default for BackingMemory {
    fn default() -> Self {
        BackingMemory::RegularOldeVec(Vec::new())
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
            inner: BackingMemory::RegularOldeVec(vec![EMPTY_XY_INDEX; size]),
        }
    }

    pub fn from_mmap(backing_memory_map: MmapMut, xy_to_entity: ManuallyDrop<Vec<usize>>) -> Self {
        VArray {
            inner: BackingMemory::Mmap(backing_memory_map, xy_to_entity),
        }
    }

    pub fn as_slice(&self) -> &[usize] {
        match &self.inner {
            BackingMemory::RegularOldeVec(vec) => vec.as_slice(),
            BackingMemory::Mmap(_, vec) => vec.as_slice(),
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [usize] {
        match self.inner {
            BackingMemory::RegularOldeVec(ref mut vec) => vec.as_mut_slice(),
            BackingMemory::Mmap(_, ref mut vec) => vec.as_mut_slice(),
        }
    }

    pub fn len(&self) -> usize {
        self.as_slice().len()
    }
}
