use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::{VPointDirectionQ, VSegment};
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::SECTION_POINTS_I32;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use itertools::Itertools;
use std::cell::RefCell;
use std::rc::Rc;

pub struct BaseSource {
    positive: BaseSourceEighth,
    negative: BaseSourceEighth,
}

impl BaseSource {
    pub fn from_central_base(surface: &VSurface) -> Self {
        let mut offset_x_from_base = surface.tunables().base.base_chunks.as_tiles_i32();
        offset_x_from_base -= offset_x_from_base % SECTION_POINTS_I32;
        BaseSource::new(VPointDirectionQ(
            VPoint::new(offset_x_from_base, 0),
            FacDirectionQuarter::East,
        ))
    }

    fn new(origin: VPointDirectionQ) -> Self {
        origin.point().assert_even_position();
        Self {
            positive: BaseSourceEighth::new(origin, 1),
            negative: BaseSourceEighth::new(origin, -1),
        }
    }

    // pub fn positive(&mut self) -> &mut BaseSourceEighth {
    //     &mut self.positive
    // }
    //
    // pub fn negative(&mut self) -> &mut BaseSourceEighth {
    //     &mut self.negative
    // }

    pub fn into_refcells(self) -> BaseSourceRefs {
        BaseSourceRefs {
            positive: self.positive.into_rc_refcell(),
            negative: self.negative.into_rc_refcell(),
        }
    }
}

pub struct BaseSourceRefs {
    positive: Rc<RefCell<BaseSourceEighth>>,
    negative: Rc<RefCell<BaseSourceEighth>>,
}

impl BaseSourceRefs {
    pub fn positive_rc(&self) -> Rc<RefCell<BaseSourceEighth>> {
        self.positive.clone()
    }

    pub fn negative_rc(&self) -> Rc<RefCell<BaseSourceEighth>> {
        self.negative.clone()
    }
}

const INTRA_OFFSET: i32 = 6;

/// From a source point,
#[derive(Debug, Eq, PartialEq)]
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

    pub fn regenerate(&self) -> Self {
        Self {
            origin: self.origin,
            sign: self.sign,
            next: 1,
        }
    }

    fn get_for_index(&self, index: i32) -> BaseSourceEntry {
        const TOTAL_INTRA_RAILS: i32 = 4;

        // non-zero to move outside of no-touch area
        let stay_outside_offset = 1;
        let applied_infra_offset_pos =
            self.sign * ((index % TOTAL_INTRA_RAILS) + stay_outside_offset) * INTRA_OFFSET;
        let pos = self.origin.point().move_direction_sideways_int(
            self.origin.direction(),
            self.sign * SECTION_POINTS_I32 * (index / TOTAL_INTRA_RAILS) + applied_infra_offset_pos,
        );
        // todo: never true
        if applied_infra_offset_pos == 0 {
            pos.assert_step_rail();
        }

        // calculate the applied offset
        let applied_intra_offset = pos
            .move_direction_sideways_axis_int(self.origin.direction(), applied_infra_offset_pos)
            - pos;

        BaseSourceEntry {
            origin: VPointDirectionQ(pos, *self.origin.direction()),
            applied_intra_offset,
        }
    }

    pub fn peek_single(&self) -> BaseSourceEntry {
        self.get_for_index(self.next)
    }

    pub fn peek_at(&self, index: usize) -> BaseSourceEntry {
        self.get_for_index(self.next + i32::try_from(index).unwrap())
    }

    pub fn peek_multiple(&self, size: usize) -> Vec<BaseSourceEntry> {
        let res = (self.next..(self.next + size as i32))
            .map(|i| self.get_for_index(i))
            .collect_vec();
        assert_eq!(res.len(), size);
        res
    }

    pub fn peek_multiple_backwards(&self, size: usize) -> Vec<BaseSourceEntry> {
        let res = ((self.next - (size as i32) + 1)..=self.next)
            .map(|i| self.get_for_index(i))
            .collect_vec();
        assert_eq!(res.len(), size);
        res
    }

    pub fn undo_one(&mut self) {
        self.next -= 1;
        assert!(self.next > 1)
    }

    pub fn into_rc_refcell(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }
}

impl Iterator for BaseSourceEighth {
    type Item = BaseSourceEntry;
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.get_for_index(self.next);
        self.next += 1;
        Some(result)
    }
}

#[derive(Debug, PartialEq)]
pub struct BaseSourceEntry {
    pub origin: VPointDirectionQ,
    pub applied_intra_offset: VPoint,
}

impl BaseSourceEntry {
    pub fn segment_for_mine(
        &self,
        VPointDirectionQ(pos, direction): &VPointDirectionQ,
    ) -> VSegment {
        let test_origin = *self.origin.point() - self.applied_intra_offset;
        assert_eq!(
            test_origin.test_step_rail(),
            None,
            "Origin not step rail - pos_raw {} step {}",
            self.origin,
            test_origin
        );

        assert_eq!(
            pos.test_step_rail(),
            None,
            "Destination not step rail - pos_raw {}",
            pos,
        );

        let new_pos = *pos + self.applied_intra_offset;
        // trace!("adjusted {pos} to {new_pos} diff {}", new_pos - *pos);

        VSegment {
            start: self.origin,
            end: VPointDirectionQ(new_pos, *direction),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::navigator::base_source::{BaseSourceEighth, BaseSourceEntry, INTRA_OFFSET};
    use facto_loop_miner_fac_engine::common::vpoint::{VPOINT_ZERO, VPoint};
    use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
    use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::SECTION_POINTS_I32;
    use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;

    #[test]
    fn test_nexts() {
        let mut source =
            BaseSourceEighth::new(VPointDirectionQ(VPOINT_ZERO, FacDirectionQuarter::East), 1);
        let mut test_next = |step_count, intra_count| {
            assert_eq!(
                source.next().unwrap(),
                BaseSourceEntry {
                    origin: VPointDirectionQ(
                        VPoint::new(
                            0,
                            (SECTION_POINTS_I32 * step_count) + (INTRA_OFFSET * intra_count)
                        ),
                        FacDirectionQuarter::East
                    ),
                    applied_intra_offset: VPoint::new(0, INTRA_OFFSET * intra_count)
                }
            );
        };

        test_next(0, 1);
        test_next(0, 2);
        test_next(0, 3);
        test_next(1, 0);
        test_next(1, 1);
        test_next(1, 2);
        test_next(1, 3);
        test_next(2, 0);
        test_next(2, 1);
    }

    #[test]
    fn test_nexts_negative() {
        let mut source =
            BaseSourceEighth::new(VPointDirectionQ(VPOINT_ZERO, FacDirectionQuarter::East), -1);
        let mut test_next = |step_count, intra_count| {
            assert_eq!(
                source.next().unwrap(),
                BaseSourceEntry {
                    origin: VPointDirectionQ(
                        VPoint::new(
                            0,
                            (-SECTION_POINTS_I32 * step_count) - (INTRA_OFFSET * intra_count)
                        ),
                        FacDirectionQuarter::East
                    ),
                    applied_intra_offset: VPoint::new(0, -INTRA_OFFSET * intra_count)
                }
            );
        };

        test_next(0, 1);
        test_next(0, 2);
        test_next(0, 3);
        test_next(1, 0);
        test_next(1, 1);
        test_next(1, 2);
        test_next(1, 3);
        test_next(2, 0);
        test_next(2, 1);
    }
}
