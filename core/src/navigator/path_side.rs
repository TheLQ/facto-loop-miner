use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;

pub struct BaseSource {
    positive: BaseSourceEighth,
    negative: BaseSourceEighth,
}

impl BaseSource {
    pub fn new(origin: VPointDirectionQ) -> Self {
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

    // pub fn peek_add(&self, pos_add: usize) -> VPoint {
    //     self.get_for_pos(self.next + pos_add as i32)
    // }

    pub fn pos(&self) -> i32 {
        self.next
    }

    // pub fn peek_add_vec(&self, pos_add: usize) -> Vec<VPoint> {
    //     let result = Vec::with_capacity(pos_add);
    //
    //     result
    // }

    pub fn get_for_pos(&self, pos: i32) -> VPointDirectionQ {
        const LOOP_STEP: i32 = 8;
        let pos = self
            .origin
            .point()
            .move_direction_sideways_int(self.origin.direction(), self.sign * LOOP_STEP);
        VPointDirectionQ(pos, *self.origin.direction())
    }
}

impl Iterator for BaseSourceEighth {
    type Item = VPointDirectionQ;
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.get_for_pos(self.next);
        self.next += 1;
        Some(result)
    }
}
