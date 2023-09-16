use crate::simd::{
    any_bit_equal_m256_bool, apply_any_u8_iter_to_m256_buffer, apply_positions_iter_to_m256_buffer,
    m256_zero, m256_zero_vec, SseUnit, SSE_BITS,
};
use crate::surface::surface::Surface;
use std::mem::transmute;

pub struct SurfaceDiff {
    source: Vec<SseUnit>,
    working: Vec<SseUnit>,
}

impl SurfaceDiff {
    pub fn from_surface(surface: &Surface) -> Self {
        let len = surface.buffer.len();
        let mut source = m256_zero_vec((len - (len % SSE_BITS)) / SSE_BITS);
        let raw_buffer: &[u8] = unsafe { transmute(surface.buffer.as_slice()) };
        apply_any_u8_iter_to_m256_buffer((*raw_buffer).into_iter(), &mut source);

        let mut working = m256_zero_vec(source.len());

        let res = SurfaceDiff { source, working };

        res
    }

    pub fn is_positions_free(&mut self, positions: Vec<usize>) -> bool {
        apply_positions_iter_to_m256_buffer(&positions, &mut self.working, true);

        let found = any_bit_equal_m256_bool(&self.source, &self.working);

        self.reset_source();
        // apply_positions_iter_to_m256_buffer(&positions, &mut self.working, false);

        !found
    }

    fn reset_source(&mut self) {
        for source in self.source.iter_mut() {
            *source = m256_zero();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::opencv::{load_cv_from_buffer, load_raw_image_from_slice};
    use crate::simd::SSE_BITS;
    use crate::simd_diff::SurfaceDiff;
    use crate::surface::pixel::Pixel;
    use crate::surface::surface::Surface;
    use num_format::Locale::he;
    use opencv::core::Mat;
    use std::mem::transmute;

    #[test]
    fn test() {
        const WIDTH: usize = SSE_BITS;
        const HEIGHT: usize = 4;

        let mut pixel_positions_test = Vec::new();
        for i in 0..HEIGHT {
            // pixel_positions_test.push((i * WIDTH) + ((i + 1) * 5));
            pixel_positions_test.push((i * WIDTH) + 5);
        }

        let mut input_raw = Vec::new();
        for _ in 0..(WIDTH * HEIGHT) {
            input_raw.push(Pixel::Empty);
        }
        for (pos, pixel_pos) in pixel_positions_test.iter().enumerate() {
            let slice = input_raw.as_mut_slice();
            slice[*pixel_pos] = Pixel::Rail;
            // should be ignored
            slice[pixel_pos + 1] = Pixel::Rail;
            slice[pixel_pos - 5] = Pixel::Rail;
        }

        let mut surface = Surface::new(WIDTH as u32, HEIGHT as u32 - 1);
        surface.buffer = input_raw;

        assert_eq!(
            SurfaceDiff::from_surface(&surface).is_positions_free(pixel_positions_test.clone()),
            false
        );

        println!("-----");

        for pos in &pixel_positions_test {
            let slice = surface.buffer.as_mut_slice();
            slice[*pos] = Pixel::Empty;
        }

        assert_eq!(
            SurfaceDiff::from_surface(&surface).is_positions_free(pixel_positions_test),
            true
        );
    }
}
