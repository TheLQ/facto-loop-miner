use crate::surfacev::ventity_map::VEntityMap;
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vsurface::VPixel;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use tracing::info;

pub struct SurfacePixelsPatchesMut<'s> {
    pixels: &'s mut VEntityMap<VPixel>,
    patches: &'s mut Vec<VPatch>,
}

impl<'s> SurfacePixelsPatchesMut<'s> {
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

struct SurfacePixelsPatches<'s> {
    pixels: &'s mut VEntityMap<VPixel>,
    patches: &'s mut Vec<VPatch>,
}

impl SurfacePixelsPatches<'_> {
    pub fn get_patches_slice(&self) -> &[VPatch] {
        &self.patches
    }
}
