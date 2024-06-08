use crate::gamedata::lua::{read_lua_tiles, LuaEntity, LuaPoint, LuaThing};
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::surfacev::err::VResult;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use crate::util::duration::BasicWatch;
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

        let center = surface.get_pixel(VPoint::new(0, 0));
        // if center != Pixel::SteelChest {
        //     panic!("unexpeted centerpoint {:?}", center);
        // }

        surface.save(&params.step_out_dir)?;

        Ok(())
    }
}

fn find_radius(data: &[LuaEntity]) -> f32 {
    let mut bottom_left = LuaPoint { x: 0.0, y: 0.0 };
    let mut top_right = LuaPoint { x: 0.0, y: 0.0 };
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
    bottom_left: &mut LuaPoint,
    top_right: &mut LuaPoint,
) {
    for thing in things {
        let pos = thing.position();
        *bottom_left = LuaPoint {
            x: bottom_left.x.min(pos.x),
            y: bottom_left.y.min(pos.y),
        };
        *top_right = LuaPoint {
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
    for entity in entities {
        surface.set_pixel(
            VPoint::from_f32_with_offset(entity.position().to_point2f(), 0.5)?,
            *entity.name(),
        )?;
        params
            .metrics
            .borrow_mut()
            .increment_slow(entity.name().as_ref());
    }
    Ok(())
}
