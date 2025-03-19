use crate::gamedata::lua::{read_lua_tiles, LuaEntity, LuaThing};
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::surface::pixel::Pixel;
use crate::surfacev::err::VResult;
use crate::surfacev::vsurface::VSurface;
use crate::util::duration::BasicWatch;
use facto_loop_miner_fac_engine::blueprint::bpfac::position::FacBpPosition;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

pub struct Step00 {}

/// Import loop-miner-scanner data into usable `VSurface`
impl Step00 {
    pub(crate) fn new_boxed() -> Box<dyn Step> {
        Box::new(Step00 {})
    }
}

impl Step for Step00 {
    fn name(&self) -> &'static str {
        "step00-import"
    }

    fn transformer(&self, params: StepParams) -> XMachineResult<()> {
        let lua_dir = Path::new("work/lm-artful");
        let lua_tiles = read_lua_tiles(lua_dir);

        let convert_watch = BasicWatch::start();
        let radius = find_radius(&lua_tiles) as u32;
        let mut surface = VSurface::new(radius);
        translate_entities_to_image(&lua_tiles, &mut surface, &params)?;
        info!("Converted in {}", convert_watch);

        // let center = surface.get_pixel(VPoint::new(0, 0));
        // if center != Pixel::SteelChest {
        //     panic!("unexpeted centerpoint {:?}", center);
        // }

        surface.save(&params.step_out_dir)?;

        Ok(())
    }
}

fn find_radius(data: &[LuaEntity]) -> f32 {
    let mut bottom_left: FacBpPosition = FacBpPosition { x: 0.0, y: 0.0 };
    let mut top_right = FacBpPosition { x: 0.0, y: 0.0 };
    find_radius_max(data, &mut bottom_left, &mut top_right);

    let mut max_radius = 0.0f32;
    max_radius = max_radius.max(bottom_left.x.abs());
    max_radius = max_radius.max(bottom_left.y.abs());
    max_radius = max_radius.max(top_right.x.abs());
    max_radius = max_radius.max(top_right.y.abs());

    // spacing
    max_radius += 10.0;

    info!(
        "Radius {} from top_right {:?} bottom_left {:?}",
        max_radius, top_right, bottom_left
    );

    max_radius
}
fn find_radius_max<T: LuaThing>(
    things: &[T],
    bottom_left: &mut FacBpPosition,
    top_right: &mut FacBpPosition,
) {
    for thing in things {
        let pos = thing.position();
        *bottom_left = FacBpPosition {
            x: bottom_left.x.min(pos.x),
            y: bottom_left.y.min(pos.y),
        };
        *top_right = FacBpPosition {
            x: top_right.x.max(pos.x),
            y: top_right.y.max(pos.y),
        };
    }
}

fn translate_entities_to_image<E>(
    entities: &[E],
    surface: &mut VSurface,
    params: &StepParams,
) -> VResult<()>
where
    E: LuaThing,
{
    let mut mega_init_entities: HashMap<Pixel, Vec<VPoint>> = HashMap::new();

    for entity in entities {
        let name = *entity.name();
        mega_init_entities
            .entry(name)
            .or_default()
            .push(entity.position().to_vpoint_with_offset(0.5, 0.5));
        params
            .metrics
            .borrow_mut()
            .increment_slow(entity.name().as_ref());
    }

    for (pixel, points) in mega_init_entities {
        surface.set_pixels(pixel, points)?;
    }
    Ok(())
}
