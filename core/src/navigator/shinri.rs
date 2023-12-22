use crate::navigator::mori::Rail;
use crate::surfacev::vsurface::VSurface;
use crossbeam::queue::SegQueue;

/// Pathfinder executor v1, Josuiji Shinri
///
/// Pathfinders have several possible destinations. They also start from multiple places.
/// Normal sequential search is slow and not always optimal.
///
/// This manages the execution
pub struct ShinriExecutor {
    surface: VSurface,
}

impl ShinriExecutor {
    pub fn new(surface: VSurface) -> Self {
        ShinriExecutor { surface }
    }

    pub fn start(&self) {
        let queue: SegQueue<ShinriTask> = SegQueue::new();
    }
}

pub struct ShinriTask {
    surface: VSurface,
    start_patch_index: usize,
    destination: Rail,
}
