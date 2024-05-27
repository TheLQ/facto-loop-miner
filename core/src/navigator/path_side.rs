use crate::navigator::mori::RAIL_STEP_SIZE_I32;
use crate::state::machine_v1::CENTRAL_BASE_TILES;
use crate::surfacev::vpoint::{VPoint, SHIFT_POINT_ONE};
use std::rc::Rc;

const CENTRAL_BASE_TILES_BY_RAIL_STEP: i32 = CENTRAL_BASE_TILES
    + ((RAIL_STEP_SIZE_I32 * 2) - (CENTRAL_BASE_TILES % (RAIL_STEP_SIZE_I32 * 2)));

pub struct BaseSource {
    positive: Rc<BaseSourceEighth>,
    negative: Rc<BaseSourceEighth>,
}

/// Because a struct field of IntoIterator<VPoint> creates Rust type hell
pub struct BaseSourceEighth {
    sign: i32,
    next: i32,
}

impl BaseSourceEighth {
    pub fn next(&mut self) -> VPoint {
        let result = self.get_for_pos(self.next);
        self.next = self.next + 1;
        result
    }

    pub fn peek_add(&self, pos_add: usize) -> VPoint {
        self.get_for_pos(self.next + pos_add as i32)
    }

    // pub fn peek_add_vec(&self, pos_add: usize) -> Vec<VPoint> {
    //     let result = Vec::with_capacity(pos_add);
    //
    //     result
    // }

    pub fn get_for_pos(&self, pos: i32) -> VPoint {
        VPoint::new(
            CENTRAL_BASE_TILES_BY_RAIL_STEP,
            self.sign * pos * RAIL_STEP_SIZE_I32 * 2,
        ) + SHIFT_POINT_ONE
    }
}
