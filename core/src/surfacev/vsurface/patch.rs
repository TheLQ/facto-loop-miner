use crate::surfacev::mine::MineLocation;
use crate::surfacev::ventity_map::{VEntityMap, VPixel};
use crate::surfacev::vpatch::VPatch;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use tracing::{debug, info};

pub struct PlugMut<'s> {
    pub(super) pixels: &'s mut VEntityMap<VPixel>,
    pub(super) patches: &'s mut Vec<VPatch>,
}

impl<'s> PlugMut<'s> {
    pub fn remove_patches_within_radius(&mut self, radius: u32) {
        let mut removed_points: Vec<VPoint> = Vec::new();
        let mut patches_to_remove = Vec::new();
        for (patch_index, patch) in self.patches.iter().enumerate() {
            if !patch.area.point_center().is_within_center_radius(radius) {
                // trace!("asdf {:?}\tfor {:?}", patch.area.start, patch.resource);
                continue;
            }
            // trace!("hello {:?}", patch);
            removed_points.extend_from_slice(&patch.pixel_indexes);
            patches_to_remove.push(patch_index);
        }
        info!(
            "removing {} patches with {} entities within {} radius",
            patches_to_remove.len(),
            removed_points.len(),
            radius
        );
        self.pixels.change(removed_points).remove();

        patches_to_remove.reverse();
        for patch_index in patches_to_remove {
            self.patches.remove(patch_index);
        }
    }

    pub fn remove_patches_in_column(&mut self, radius: u32) {
        let mut removed_points: Vec<VPoint> = Vec::new();
        let mut patches_to_remove = Vec::new();
        let radius = radius as i32;
        for (patch_index, patch) in self.patches.iter().enumerate() {
            if (-radius..radius).contains(&patch.area.point_center().x()) {
                removed_points.extend_from_slice(&patch.pixel_indexes);
                patches_to_remove.push(patch_index);
            }
        }
        info!(
            "removing {} patches with {} entities within {} radius",
            patches_to_remove.len(),
            removed_points.len(),
            radius
        );
        self.pixels.change(removed_points).remove();

        patches_to_remove.reverse();
        for patch_index in patches_to_remove {
            self.patches.remove(patch_index);
        }
    }

    pub fn add_patches(&mut self, patches: impl IntoIterator<Item = VPatch>) {
        self.patches.extend(patches)
    }
}

#[derive(Clone, Copy)]
pub struct Plug<'s> {
    pub(super) pixels: &'s VEntityMap<VPixel>,
    pub(super) patches: &'s Vec<VPatch>,
}

impl<'s> Plug<'s> {
    pub fn get_patches(&self) -> &[VPatch] {
        self.patches
    }

    pub fn mine_patches(&self, mine: &MineLocation) -> impl Iterator<Item = &VPatch> {
        mine.patch_indexes()
            .iter()
            .map(|patch_index| &self.patches[*patch_index])
    }

    pub fn mine_patches_len(mine: &MineLocation) -> usize {
        mine.patch_indexes().len()
    }

    /// Anti-entropy
    pub fn validate(&self) {
        self.pixels.validate();
        self.validate_patches();
    }

    fn validate_patches(&self) {
        if self.patches.is_empty() {
            panic!("no patches to validate")
        }
        let mut checks = 0;
        let mut points_history: Vec<&VPoint> = Vec::new();
        for patch in self.patches.as_slice() {
            for point in &patch.pixel_indexes {
                if points_history.contains(&point) {
                    panic!("dupe {patch:?}");
                }
                points_history.push(point);

                let pixel = self.pixels.get_entity_by_point(point).unwrap();
                assert_eq!(pixel.pixel, patch.resource);
                checks += 1;
            }
        }
        debug!("validate {checks} checks");
    }

    pub fn get_patch_index(&self, patch: &VPatch) -> usize {
        self.patches
            .iter()
            .position(|surface_patch| patch == surface_patch)
            .unwrap()
    }
}

//

pub trait AsVsMut<'s> {
    fn patches_mut(&mut self) -> PlugMut<'s>;
}

pub trait AsVs<'s> {
    fn patches(&'s self) -> Plug<'s>;
}
