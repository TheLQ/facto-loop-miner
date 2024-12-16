use crate::navigator::mori::RAIL_STEP_SIZE_I32;
use crate::state::machine_v1::CENTRAL_BASE_TILES;
use facto_loop_miner_fac_engine::common::vpoint::{VPoint, SHIFT_POINT_ONE};
use std::rc::Rc;
use std::sync::Mutex;

const CENTRAL_BASE_TILES_BY_RAIL_STEP: i32 = CENTRAL_BASE_TILES
    + ((RAIL_STEP_SIZE_I32 * 2) - (CENTRAL_BASE_TILES % (RAIL_STEP_SIZE_I32 * 2)));

pub struct BaseSource {
    positive: Rc<Mutex<BaseSourceEighth>>,
    negative: Rc<Mutex<BaseSourceEighth>>,
}

impl BaseSource {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            positive: Rc::new(Mutex::new(BaseSourceEighth::new(1))),
            negative: Rc::new(Mutex::new(BaseSourceEighth::new(-1))),
        }
    }

    pub fn get_positive(&self) -> Rc<Mutex<BaseSourceEighth>> {
        self.positive.clone()
    }

    pub fn get_negative(&self) -> Rc<Mutex<BaseSourceEighth>> {
        self.negative.clone()
    }
}

/// Because a struct field of IntoIterator<VPoint> creates Rust type hell
#[derive(PartialEq, Debug)]
pub struct BaseSourceEighth {
    sign: i32,
    next: i32,
}

impl BaseSourceEighth {
    pub fn new(sign: i32) -> Self {
        // Must start at 1 due to conflict at 0!
        Self { sign, next: 1 }
    }

    pub fn next(&mut self) -> VPoint {
        let result = self.get_for_pos(self.next);
        self.next += 1;
        result
    }

    pub fn peek_add(&self, pos_add: usize) -> VPoint {
        self.get_for_pos(self.next + pos_add as i32)
    }

    pub fn pos(&self) -> i32 {
        self.next
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
