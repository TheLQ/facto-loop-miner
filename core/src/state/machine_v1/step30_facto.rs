use crate::navigator::planners::{debug_draw_mine_index_labels, debug_draw_mine_links};
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::admiral::err::AdmiralResult;
use facto_loop_miner_fac_engine::admiral::executor::client::AdmiralClient;
use facto_loop_miner_fac_engine::admiral::executor::{ExecuteResponse, LuaCompiler};
use facto_loop_miner_fac_engine::admiral::lua_command::LuaCommand;
use facto_loop_miner_fac_engine::admiral::lua_command::fac_destroy::FacDestroy;
use facto_loop_miner_fac_engine::blueprint::output::FacItemOutput;
use facto_loop_miner_fac_engine::common::entity::FacEntity;
use facto_loop_miner_fac_engine::common::names::FacEntityName;
use facto_loop_miner_fac_engine::game_blocks::block::FacBlock2;
use facto_loop_miner_fac_engine::game_blocks::mine_ore::FacBlkMineOre;
use facto_loop_miner_fac_engine::game_entities::belt::FacEntBeltType;
use facto_loop_miner_fac_engine::game_entities::chest::{FacEntChest, FacEntChestType};
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use facto_loop_miner_fac_engine::game_entities::rail_signal::FacEntRailSignalType;
use itertools::Itertools;
use std::rc::Rc;

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

        let mut client = AdmiralClient::new().unwrap();
        client.auth().unwrap();
        let output = FacItemOutput::new_admiral_dedupe(client).into_rc();

        plotter(&mut surface, output);

        Ok(())
    }
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

fn plotter(surface: &VSurface, output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    let needle_path = surface.get_mine_paths()[13].clone();

    destroy_mine_area(&needle_path.mine_base, &output)?;

    // output.writei(
    //     FacEntChest::new(FacEntChestType::Wood),
    //     needle_path.mine_base.area_min().point_center(),
    // );
    FacBlkMineOre {
        width: 2,
        height: 2,
        build_direction: FacDirectionQuarter::North,
        belt: FacEntBeltType::Basic,
        drill_modules: [None, None, None],
        output,
    }
    .generate(needle_path.mine_base.area_min().point_center());

    Ok(())
}

fn destroy_mine_area(
    mine: &MineLocation,
    output: &FacItemOutput,
) -> AdmiralResult<ExecuteResponse> {
    let command = FacDestroy::new_filtered_entities_area(
        mine.area_min(),
        [
            FacEntityName::RailStraight,
            FacEntityName::RailCurved,
            FacEntityName::RailSignal(FacEntRailSignalType::Basic),
        ]
        .to_vec(),
    );
    output.admiral_execute_command(command.into_boxed())
}
