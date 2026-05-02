use crate::surfacev::mine::MineLocation;
use crate::surfacev::vsurface::{
    MineRef, VSurfaceMine, VSurfacePatch, VSurfacePixel, VSurfacePixelAsVs,
};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use itertools::Itertools;
use std::ops::ControlFlow;
use tracing::trace;

/// Scanner v2 "Banzoin Hakka🔅"
/// Scans bases by section squares in pattern
/// Iterative scanner
pub(in super::super) struct HakkaBase {
    pub(in super::super) step_size: usize,
    pub(in super::super) origin: VPoint,
    pub(in super::super) direction_advancing: FacDirectionQuarter,
    pub(in super::super) direction_scanning: FacDirectionQuarter,
}

/// Concerned only with scanning the remaining mines
pub(in super::super) struct Hakka {
    pub(in super::super) base: HakkaBase,
    pub(in super::super) advance_i: usize,
    pub(in super::super) scanning_i: usize,
    pub(in super::super) reset_scanning: usize,
    pub(in super::super) banned_mines: Vec<MineRef>,
}

impl Hakka {
    pub(in super::super) fn new(base: HakkaBase, surface: VSurfacePatch) -> Self {
        let mut new = Self {
            base,
            advance_i: 0,
            scanning_i: 0,
            reset_scanning: 0,
            banned_mines: Vec::new(),
        };
        while !surface
            .pixels()
            .is_points_out_bounds_slice(PointAt::Normal.area_at_init(&new).get_points())
        {
            new.reset_scanning += 1;
            new.scanning_i += 1;
            assert!(new.reset_scanning < 100);
        }
        new.scanning_i = 0;
        // new.scanning_i -= 1;
        // new.reset_scanning -= 1;
        assert!(new.reset_scanning > 0);
        new
    }

    pub fn reset(&mut self) {
        self.advance_i = 0;
        self.scanning_i = 0;
    }

    pub(in super::super) fn increment(
        &mut self,
        surface: VSurfacePixel,
        cause: impl std::fmt::Display,
    ) -> ControlFlow<()> {
        trace!(
            "incrementing {} and {} reset {} - {cause}",
            self.advance_i, self.scanning_i, self.reset_scanning
        );
        if self.scanning_i == self.reset_scanning {
            self.advance_i += 1;
            self.scanning_i = 0;

            let next = PointAt::Normal.area_at_init(self);
            if surface.is_points_out_bounds_slice(next.get_corner_points()) {
                trace!(
                    "incrementing out of bounds for {} and {}",
                    self.advance_i, self.scanning_i
                );
                return ControlFlow::Break(());
            }
        } else {
            self.scanning_i += 1;
        }
        ControlFlow::Continue(())
    }

    pub(in super::super) fn scan(&self, end_at: PointAt, surface: VSurfaceMine) -> HakkaResult {
        // let scan_start = self.base.area_at(self.advance_i, self.scanning_i);
        let scan_area = end_at.area_at(self);

        let mut new_mines_in_scan_area: Vec<&MineLocation> = surface
            .all_mines_iter()
            .filter(|v| scan_area.contains_point(&v.area_min().point_center()))
            .collect();
        if new_mines_in_scan_area.is_empty() {
            return HakkaResult::NoneFound;
        }
        new_mines_in_scan_area.sort_by_key(|mine| {
            let mine_pos = mine.area_min().point_center();
            // v2 bias closer to scan_direction
            let scanning =
                |point: VPoint| -> i32 { point.axis_value(self.base.direction_scanning) };
            let scanning_axis_score = scanning(mine_pos).abs_diff(scanning(self.base.origin));

            let advancing =
                |point: VPoint| -> i32 { point.axis_value(self.base.direction_advancing) };
            let advancing_axis_score =
                advancing(mine_pos).abs_diff(advancing(self.base.origin)) * 2;

            scanning_axis_score + advancing_axis_score
        });
        new_mines_in_scan_area.reverse();

        let scanned_mines = surface
            .get_mines_ref_for(new_mines_in_scan_area)
            .collect_vec();
        HakkaResult::NewPatchesInScanArea {
            scanned_mines,
            scan_area,
        }
    }
}

pub(in super::super) enum HakkaResult {
    NoneFound,
    NewPatchesInScanArea {
        scanned_mines: Vec<MineRef>,
        scan_area: VArea,
    },
}

#[derive(Clone, Copy)]
pub(in super::super) enum PointAt {
    Normal,
    Reduced,
}

impl PointAt {
    pub(in super::super) fn area_at_init(&self, scanner: &Hakka) -> VArea {
        self._area_at(scanner, true)
    }

    pub(in super::super) fn area_at(&self, scanner: &Hakka) -> VArea {
        self._area_at(scanner, false)
    }

    fn _area_at(
        &self,
        Hakka {
            base:
                HakkaBase {
                    origin,
                    direction_advancing,
                    direction_scanning,
                    step_size,
                    ..
                },
            scanning_i,
            advance_i,
            reset_scanning,
            ..
        }: &Hakka,
        is_init: bool,
    ) -> VArea {
        let scanning = if is_init {
            // during init we are still calculating
            *scanning_i
        } else {
            reset_scanning.checked_sub(*scanning_i).unwrap()
        };
        let start = origin
            .move_direction_usz(direction_advancing, step_size * advance_i)
            .move_direction_usz(direction_scanning, step_size * scanning);
        let end = match self {
            PointAt::Normal => start
                .move_direction_usz(direction_advancing, *step_size)
                .move_direction_usz(direction_scanning, *step_size),
            PointAt::Reduced => start
                .move_direction_usz(direction_advancing, step_size / 2)
                .move_direction_usz(direction_scanning, step_size / 2),
        };
        VArea::from_arbitrary_points_pair(&start, &end)
    }
}
