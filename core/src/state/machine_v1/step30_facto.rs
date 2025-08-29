use crate::navigator::planners::{debug_draw_mine_index_labels, debug_draw_mine_links};
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::surfacev::mine::{MineLocation, MinePath};
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_common::err_bt::PrettyUnwrapMyBacktrace;
use facto_loop_miner_fac_engine::admiral::err::AdmiralResult;
use facto_loop_miner_fac_engine::admiral::executor::client::AdmiralClient;
use facto_loop_miner_fac_engine::admiral::executor::{ExecuteResponse, LuaCompiler};
use facto_loop_miner_fac_engine::admiral::lua_command::LuaCommand;
use facto_loop_miner_fac_engine::admiral::lua_command::fac_destroy::FacDestroy;
use facto_loop_miner_fac_engine::admiral::lua_command::fac_render_destroy::FacRenderDestroy;
use facto_loop_miner_fac_engine::blueprint::output::FacItemOutput;
use facto_loop_miner_fac_engine::common::entity::FacEntity;
use facto_loop_miner_fac_engine::common::names::{FacEntityName, FacEntityNameBuilder};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPOINT_ZERO, VPoint};
use facto_loop_miner_fac_engine::game_blocks::block::{FacBlock2, FacBlockFancy};
use facto_loop_miner_fac_engine::game_blocks::mine_island::FacBlkMineIsland;
use facto_loop_miner_fac_engine::game_blocks::mine_ore::FacBlkMineOre;
use facto_loop_miner_fac_engine::game_entities::belt::FacEntBeltType;
use facto_loop_miner_fac_engine::game_entities::chest::{FacEntChest, FacEntChestType};
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use facto_loop_miner_fac_engine::game_entities::electric_large::FacEntElectricLargeType;
use facto_loop_miner_fac_engine::game_entities::electric_mini::FacEntElectricMiniType;
use facto_loop_miner_fac_engine::game_entities::infinity_power::FacEntInfinityPower;
use facto_loop_miner_fac_engine::game_entities::inserter::FacEntInserterType;
use facto_loop_miner_fac_engine::game_entities::rail_signal::FacEntRailSignalType;
use itertools::Itertools;
use std::rc::Rc;
use strum::VariantArray;
use tracing::info;

pub(crate) struct Step30;

impl Step30 {
    pub fn new_boxed() -> Box<dyn Step> {
        Box::new(Step30 {})
    }
}

impl Step for Step30 {
    fn name(&self) -> &'static str {
        "step30-facto"
    }

    fn transformer(&self, params: StepParams) -> XMachineResult<()> {
        let mut surface = VSurface::load_from_last_step(&params)?;

        let output = connect_admiral().pretty_unwrap();

        let needle_path = surface.get_mine_paths()[13].clone();
        plotter(&mut surface, output.clone(), &needle_path).unwrap();

        output.flush();

        Ok(())
    }
}

fn connect_admiral() -> AdmiralResult<Rc<FacItemOutput>> {
    let mut client = AdmiralClient::new()?;
    client.auth()?;
    Ok(FacItemOutput::new_admiral_dedupe(client).into_rc())
}

fn plotter_initial(surface: &mut VSurface) {
    let mines = surface
        .get_mine_paths()
        .iter()
        .map(|v| v.mine_base.clone())
        .collect_vec();
    debug_draw_mine_index_labels(surface, mines);

    surface.paint_pixel_colored_zoomed().save_to_oculante();
}

fn plotter(
    surface: &VSurface,
    output: Rc<FacItemOutput>,
    needle_path: &MinePath,
) -> AdmiralResult<()> {
    // destroy_mine_area(&needle_path.mine_base, 20, &output)?;
    destroy_everything(surface, &output)?;

    let actual_area = VArea::from_arbitrary_points(
        needle_path
            .mine_base
            .surface_patches(surface)
            .flat_map(|v| &v.pixel_indexes),
    );
    info!(
        "DIFF start {}",
        actual_area.point_top_left() - needle_path.mine_base.area_min().point_top_left()
    );
    info!(
        "DIFF end   {}",
        actual_area.point_bottom_right() - needle_path.mine_base.area_min().point_bottom_right()
    );

    // output.writei(
    //     FacEntChest::new(FacEntChestType::Wood),
    //     needle_path.mine_base.area_min().point_center(),
    // );
    let patch = needle_path
        .mine_base
        .surface_patches(surface)
        .next()
        .unwrap();
    output.writei(
        FacEntInfinityPower::new(),
        patch.area.point_top_left() + VPoint::new(0, 20),
    );

    // FacBlkMineOre {
    //     ore_points: patch.pixel_indexes.clone(),
    //     exit_clockwise: true,
    //     exit_direction: FacDirectionQuarter::South,
    //     belt: FacEntBeltType::Basic,
    //     drill_modules: [None, None, None],
    //     output,
    // }
    // .generate();
    // for direct in FacDirectionQuarter::VARIANTS {
    //     FacBlkMineOre {
    //         ore_points: patch.pixel_indexes.clone(),
    //         exit_clockwise: true,
    //         exit_direction: *direct,
    //         belt: FacEntBeltType::Basic,
    //         drill_modules: [None, None, None],
    //         output: output.clone(),
    //     }
    //     .generate();
    // }

    FacBlkMineIsland {
        rail_entrance_link: needle_path.sodas.last().unwrap().clone(),
        wagons: 3,
        front_engines: 3,
        drill_modules: [None, None, None],
        belt: FacEntBeltType::Basic,
        inserter: FacEntInserterType::Basic,
        mines: needle_path
            .mine_base
            .surface_patches(surface)
            .map(|v| v.pixel_indexes.clone())
            .collect(),
        output: output.clone(),
    }
    .generate();

    Ok(())
}

fn destroy_everything(
    surface: &VSurface,
    output: &FacItemOutput,
) -> AdmiralResult<ExecuteResponse> {
    destroy_area(
        VArea::from_arbitrary_points_pair(VPOINT_ZERO, surface.point_bottom_right()),
        output,
    )
}

fn destroy_mine_area(
    mine: &MineLocation,
    margin: i32,
    output: &FacItemOutput,
) -> AdmiralResult<ExecuteResponse> {
    destroy_area(mine.area_min().expand_margin(margin), output)
}

fn destroy_area(area: VArea, output: &FacItemOutput) -> AdmiralResult<ExecuteResponse> {
    output.admiral_execute_command(
        FacDestroy::new_filtered_entities_area(
            area.clone(),
            FacEntityNameBuilder::new_all().into_vec(),
        )
        .into_boxed(),
    )?;

    output.admiral_execute_command(FacRenderDestroy::destroy_area(area.clone()).into_boxed())
}
