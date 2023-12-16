use crate::simd::{
    any_bit_equal_m256_bool, apply_positions_iter_to_m256_buffer, m256_zero, SseUnit,
};
use crate::surfacev::vsurface::VSurface;

pub struct SurfaceDiff {
    surface_copy: Vec<u8>,
    source: Vec<SseUnit>,
    working: Vec<SseUnit>,
}

impl SurfaceDiff {
    pub fn TODO_new() -> Self {
        SurfaceDiff {
            working: Vec::new(),
            source: Vec::new(),
            surface_copy: Vec::new(),
        }
    }

    pub fn from_surface(surface: &VSurface) -> Self {
        todo!()
        // if 1 + 1 == 2 {
        //     return Self::TODO_new();
        // }
        // let len = todo!(); // surface.buffer.len();
        // let mut source = m256_zero_vec((len - (len % SSE_BITS)) / SSE_BITS);
        // let surface_buffer: VSurface = todo!(); // surface.buffer.clone();
        // let raw_buffer: &[u8] = unsafe { transmute(surface_buffer.as_slice()) };
        // // TODO: was raw_buffer.clone()
        // let raw_buffer2 = raw_buffer;
        // apply_any_u8_iter_to_m256_buffer((*raw_buffer).iter(), &mut source);
        //
        // let working = m256_zero_vec(source.len());
        //
        // let res = SurfaceDiff {
        //     source,
        //     working,
        //     surface_copy: Vec::from(raw_buffer2),
        // };
        //
        // res
    }

    pub fn is_positions_free(&mut self, positions: Vec<usize>) -> bool {
        apply_positions_iter_to_m256_buffer(&positions, &mut self.working, true);

        // let mut talked = false;
        // for pos in &positions {
        //     if !talked && self.surface_copy[pos] != 0 {
        //         // tracing::debug!("expect {:?} at {}", self.surface_copy[pos], pos);
        //         // tracing::debug!(
        //         //     "{}{}\n{}{}",
        //         //     "source  ",
        //         //     format_m256(*self.source.get(bucket_div(pos, SSE_BITS)).unwrap()),
        //         //     "working ",
        //         //     format_m256(*self.working.get(bucket_div(pos, SSE_BITS)).unwrap()),
        //         // );
        //     }
        // }

        let found = any_bit_equal_m256_bool(&self.source, &self.working);

        // self.reset_buffer();
        // tracing::debug!("found {}", found);
        apply_positions_iter_to_m256_buffer(&positions, &mut self.working, false);

        !found
    }

    fn reset_buffer(&mut self) {
        for entry in self.working.iter_mut() {
            *entry = m256_zero();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::simd::SSE_BITS;
    use crate::simd_diff::SurfaceDiff;
    use crate::surface::pixel::Pixel;
    use crate::surface::surface::Surface;

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

        tracing::debug!("-----");

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
