use crate::simd::{
    apply_any_u8_iter_to_m256_buffer, apply_positions_iter_to_m256_buffer, compare_m256_bool,
    m256_zero_vec, SseUnit, SSE_BITS,
};
use crate::surface::surface::Surface;
use std::mem::transmute;

pub struct SurfaceDiff {
    source: Vec<SseUnit>,
}

impl SurfaceDiff {
    pub fn from_surface(surface: &Surface) -> Self {
        let len = surface.buffer.len();
        let mut source = m256_zero_vec((len - (len % SSE_BITS)) / SSE_BITS);
        let raw_buffer: &[u8] = unsafe { transmute(surface.buffer.as_slice()) };
        apply_any_u8_iter_to_m256_buffer((*raw_buffer).into_iter(), &mut source);

        SurfaceDiff { source }
    }

    pub fn diff_positions(&mut self, positions: impl Iterator<Item = usize>) -> bool {
        let mut working = m256_zero_vec(self.source.len());

        apply_positions_iter_to_m256_buffer(positions, &mut working);

        let res = compare_m256_bool(self.source.iter(), working.iter());

        res
    }
}
