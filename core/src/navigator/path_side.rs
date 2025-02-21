use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_dual::DUAL_RAIL_STEP_I32;

pub struct BaseSource {
    positive: BaseSourceEighth,
    negative: BaseSourceEighth,
}

impl BaseSource {
    pub fn new(origin: VPointDirectionQ) -> Self {
        origin.point().assert_even_position();
        Self {
            positive: BaseSourceEighth::new(origin, 1),
            negative: BaseSourceEighth::new(origin, -1),
        }
    }

    pub fn positive(&mut self) -> &mut BaseSourceEighth {
        &mut self.positive
    }

    pub fn negative(&mut self) -> &mut BaseSourceEighth {
        &mut self.negative
    }
}

/// Because a struct field of IntoIterator<VPoint> creates Rust type hell
#[derive(Debug)]
pub struct BaseSourceEighth {
    origin: VPointDirectionQ,
    sign: i32,
    next: i32,
}

impl BaseSourceEighth {
    pub fn new(origin: VPointDirectionQ, sign: i32) -> Self {
        // Must start at 1 due to conflict at 0!
        Self {
            origin,
            sign,
            next: 1,
        }
    }

    fn get_for_index(&self, index: i32) -> VPointDirectionQ {
        const LOOP_STEP: i32 = DUAL_RAIL_STEP_I32;
        let pos = self
            .origin
            .point()
            .move_direction_sideways_int(self.origin.direction(), self.sign * LOOP_STEP * index);
        tracing::trace!("working with {} from {}", pos, self.origin);
        pos.assert_step_rail();
        VPointDirectionQ(pos, *self.origin.direction())
    }
}

impl Iterator for BaseSourceEighth {
    type Item = VPointDirectionQ;
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.get_for_index(self.next);
        self.next += 1;
        Some(result)
    }
}
