use crate::navigator::planners::PathingTunables;
use crate::state::tuneables::PathCommonTunables;
use crate::surfacev::mine::{MineDestination, MinePath};
use crate::surfacev::vsurface::{VSurfaceRailAsVs, VSurfaceRailMut};
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::{VPointDirectionQ, VSegment};
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::SECTION_POINTS_I32;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::error;

pub struct BaseSource {
    positive: BaseSourceEighth,
    negative: BaseSourceEighth,
}

impl BaseSource {
    pub fn from_central_base(tunables: &PathingTunables) -> Self {
        let mut offset_x_from_base = tunables.base_chunks().as_tiles_i32();
        offset_x_from_base -= offset_x_from_base % SECTION_POINTS_I32;
        offset_x_from_base += SECTION_POINTS_I32;
        BaseSource::new(
            VPointDirectionQ(
                VPoint::new(offset_x_from_base, 0),
                FacDirectionQuarter::East,
            ),
            tunables.path_common().clone(),
        )
    }

    fn new(origin: VPointDirectionQ, tunables: PathCommonTunables) -> Self {
        origin.point().assert_even_position();
        Self {
            positive: BaseSourceEighth::new(origin, 1, tunables.clone()),
            negative: BaseSourceEighth::new(origin, -1, tunables),
        }
    }

    pub fn into_positive(self) -> BaseSourceEighth {
        self.positive
    }

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

// const SMALLEST_RAIL_SQUARE: i32 = 6;
// const TOTAL_INTRA_RAILS: i32 = 4;

/// Dual wide rail is 6x26 = [SMALLEST_RAIL_SQUARE] * [SECTION_POINTS_I32]
/// Rail navigates on a 26x26 grid = [SECTION_POINTS_I32]
/// For less wasteful navigation
/// Advance by 45 degrees with xy-square offset 6 = [SMALLEST_RAIL_SQUARE]
/// Non-perfect pattern, so can only advance 4 times = [TOTAL_INTRA_RAILS] (4*6=24)
#[derive(Debug, Eq, PartialEq)]
pub struct BaseSourceEighth {
    origin: VPointDirectionQ,
    sign: i32,
    next: i32,
    tunables: PathCommonTunables,
}

impl BaseSourceEighth {
    pub fn new(origin: VPointDirectionQ, sign: i32, tunables: PathCommonTunables) -> Self {
        origin.point().assert_step_rail();
        // Must start at 1 due to conflict at 0!
        Self {
            origin,
            sign,
            next: 1,
            tunables,
        }
    }

    pub fn intra_level_at_index(&self, level: u8) -> IntraLevel {
        IntraLevel {
            level,
            step_forward: self.tunables.base_source_intra_forward,
            step_sideways: self.tunables.base_source_intra_sideways,
            direction: self.origin.direction(),
        }
    }

    pub fn all_intra_levels(&self) -> impl Iterator<Item = IntraLevel> {
        (0..self.tunables.base_source_intra_rails).map(|i| self.intra_level_at_index(i))
    }

    fn get_for_index(&self, index: i32) -> BaseSourceEntry {
        // tracing::trace!("get for index {index}");
        let section_move = (index / self.tunables.base_source_intra_rails as i32)
            * self.tunables.base_source_section_step;
        let section_pos = self
            .origin
            .point()
            .move_direction_sideways_int(self.origin.direction(), section_move);
        section_pos.assert_step_rail();

        let applied_intra = self.intra_level_at_index(
            u8::try_from(index % self.tunables.base_source_intra_rails as i32).unwrap(),
        );
        let intra_pos = applied_intra.apply(section_pos);

        BaseSourceEntry {
            origin: VPointDirectionQ(intra_pos, self.origin.direction()),
            applied_intra,
        }
    }

    pub fn peek_single(&self) -> BaseSourceEntry {
        self.get_for_index(self.next)
    }

    pub fn peek_after(&self, index: usize) -> BaseSourceEntry {
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

    pub fn origin(&self) -> VPointDirectionQ {
        self.origin
    }

    pub fn fixed_limiting_start(&self) -> VPoint {
        let VPointDirectionQ(origin, direction) = self.origin;
        // Must give spacing from Edge, because hope_link.area() can extend past it.
        origin
            .move_direction_int(direction, -SECTION_POINTS_I32)
            .move_direction_sideways_int(direction, -SECTION_POINTS_I32)
    }

    fn _undo_one(&mut self) -> BaseSourceEntry {
        self.next -= 1;
        // this value was last given, and will be repeated
        let current = self.get_for_index(self.next);
        assert!(self.next >= 1);
        current
    }

    pub fn undo_mine_path(
        &mut self,
        surface: &mut VSurfaceRailMut,
        cause: impl std::fmt::Display,
    ) -> Option<(MinePath, Vec<VPoint>, BaseSourceEntry)> {
        let (path, points) =
            surface.remove_mine_path_pop(format!("undoing {} - {cause}", self.next))?;
        let undo = self._undo_one();
        assert_eq!(path.segment.start, undo.origin);

        Some((path, points, undo))
    }

    pub fn undo_mine_path_until_index(
        &mut self,
        surface: &mut VSurfaceRailMut,
        remove_until: usize,
    ) -> Vec<MinePath> {
        let mut res = Vec::new();
        let mut i = 0;
        while { surface.rails().get_paths().len() } > remove_until {
            let (path, _, _) = self
                .undo_mine_path(surface, format!("[rollback] pop {i}"))
                .unwrap();
            i += 1;
            res.push(path);
        }

        res
    }

    pub fn into_rc_refcell(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }

    pub fn advance_sorting(&mut self, mut input: Vec<MinePath>) -> Vec<MinePath> {
        let mut sorted = Vec::with_capacity(input.len());
        for i in 0..input.len() {
            let next = self.next().unwrap();
            match input.iter().position(|v| v.segment.start == next.origin) {
                Some(actual_i) => sorted.push(input.remove(actual_i)),
                None => {
                    panic!("not found i {i} origin {}", next.origin);
                    // ignore
                }
            }
        }

        if input.is_empty() {
            sorted
        } else {
            for input in input {
                error!("unsorted {input:?}")
            }
            panic!("base not found in path")
        }
    }

    pub fn get_i(&self) -> i32 {
        self.next
    }
}

impl Iterator for BaseSourceEighth {
    type Item = BaseSourceEntry;
    fn next(&mut self) -> Option<Self::Item> {
        tracing::trace!("nexting {}", self.next);
        let current = self.get_for_index(self.next);
        self.next += 1;
        Some(current)
    }
}

#[derive(Debug, PartialEq)]
pub struct BaseSourceEntry {
    pub origin: VPointDirectionQ,
    pub applied_intra: IntraLevel,
}

impl BaseSourceEntry {
    pub fn segment_for_mine(&self, destination: &MineDestination) -> VSegment {
        let orig_origin = self.applied_intra.undo(self.origin.point());
        assert_eq!(
            orig_origin.test_step_rail(),
            None,
            "Origin not step rail - pos_raw {} step {}",
            self.origin,
            orig_origin
        );

        let end = destination.for_level(&self.applied_intra);
        let orig_pos = self.applied_intra.undo(end.point());
        assert_eq!(
            orig_pos.test_step_rail(),
            None,
            "Destination not step rail - pos {} orig_pause {orig_pos}",
            end.point()
        );

        VSegment {
            start: self.origin,
            end,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub struct IntraLevel {
    level: u8,
    step_forward: i32,
    step_sideways: i32,
    direction: FacDirectionQuarter,
}

impl IntraLevel {
    pub fn apply(&self, input: VPoint) -> VPoint {
        input
            .move_direction_int(self.direction, i32::from(self.level) * self.step_forward)
            .move_direction_sideways_int(self.direction, i32::from(self.level) * self.step_sideways)
    }

    pub fn undo(&self, input: VPoint) -> VPoint {
        let backwards = self.direction.rotate_flip();
        input
            .move_direction_int(backwards, i32::from(self.level) * self.step_forward)
            .move_direction_sideways_int(backwards, i32::from(self.level) * self.step_sideways)
    }
}

#[cfg(test)]
mod test {
    use crate::navigator::base_source::{BaseSourceEighth, BaseSourceEntry, IntraLevel};
    use crate::navigator::planners::PathingTunables;
    use crate::state::tuneables::{PathCommonTunables, Tunables};
    use crate::surfacev::vsurface::{VSurfaceRailAsVs, VSurfaceRailAsVsMut};
    use facto_loop_miner_common::log_init_trace;
    use facto_loop_miner_fac_engine::common::vpoint::{VPOINT_ZERO, VPoint};
    use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
    use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::SECTION_POINTS_I32;
    use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
    use tracing::info;

    #[test]
    fn test_nexts() {
        log_init_trace();

        let tunables = Tunables::new();

        info!("test_nexts");

        let mut source = BaseSourceEighth::new(
            VPointDirectionQ(VPOINT_ZERO, FacDirectionQuarter::East),
            1,
            tunables.path_common.clone(),
        );
        struct StepIntra(usize, usize);
        let expected_plan = [
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 0),
            (1, 1),
            (1, 2),
            (1, 3),
            (2, 0),
            (2, 1),
        ];
        let mut is_failed = false;
        for (sections, intras) in expected_plan {
            let next = source.next().unwrap();

            let intra = SMALLEST_RAIL_SQUARE * intras;
            let expected_entry = BaseSourceEntry {
                origin: VPointDirectionQ(
                    VPoint::new(intra, intra + (SECTION_POINTS_I32 * sections)),
                    FacDirectionQuarter::East,
                ),
                applied_intra: IntraLevel {
                    direction: FacDirectionQuarter::East,
                    level: intras.try_into().unwrap(),
                },
            };

            let test_result = next == expected_entry;
            is_failed = is_failed || !test_result;

            // println!(
            //     "{test_result} expected {} next {} offset {}",
            //     expected_entry.origin.point(),
            //     next.origin.point(),
            //     // next.origin.point() - &next.applied_intra_offset,
            //     next.applied_intra_offset
            // );
            info!(
                "={test_result}\n{:<8}: {}\n{:<8}: {}\n{:<8}: {}\n{:<8}: {:?}\n{:<8}: {:?}",
                "expected",
                expected_entry.origin.point(),
                "next",
                next.origin.point(),
                "",
                next.applied_intra.undo(*next.origin.point()),
                "offset-actual",
                next.applied_intra,
                "offset-expect",
                expected_entry.applied_intra,
            );
        }
        assert!(!is_failed);
    }

    // #[test]
    // fn test_nexts_negative() {
    //     let mut source =
    //         BaseSourceEighth::new(VPointDirectionQ(VPOINT_ZERO, FacDirectionQuarter::East), -1);
    //     let mut test_next = |step_count, intra_count| {
    //         assert_eq!(
    //             source.next().unwrap(),
    //             BaseSourceEntry {
    //                 origin: VPointDirectionQ(
    //                     VPoint::new(
    //                         0,
    //                         (-SECTION_POINTS_I32 * step_count) - (INTRA_OFFSET * intra_count)
    //                     ),
    //                     FacDirectionQuarter::East
    //                 ),
    //                 applied_intra_offset: VPoint::new(0, -INTRA_OFFSET * intra_count)
    //             }
    //         );
    //     };
    //
    //     test_next(0, 1);
    //     test_next(0, 2);
    //     test_next(0, 3);
    //     test_next(1, 0);
    //     test_next(1, 1);
    //     test_next(1, 2);
    //     test_next(1, 3);
    //     test_next(2, 0);
    //     test_next(2, 1);
    // }
}
