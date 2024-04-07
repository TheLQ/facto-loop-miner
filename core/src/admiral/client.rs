use crate::admiral::err::AdmiralResult;
use crate::admiral::executor::client::AdmiralClient;
use crate::admiral::executor::file::AdmiralFile;
use crate::admiral::executor::LuaCompiler;
use crate::admiral::generators::assembler_farm::{AssemblerChest, AssemblerFarmGenerator};
use crate::admiral::generators::beacon_farm::BeaconFarmGenerator;
use crate::admiral::generators::terapower::Terapower;
use crate::admiral::lua_command::fac_destroy::FacDestroy;
use crate::admiral::lua_command::lua_batch::LuaBatchCommand;
use opencv::core::{Point, Point2f};
use tracing::error;

pub fn admiral() {
    if let Err(e) = inner_admiral() {
        error!("Admiral failed! {}\n{}", e, e.my_backtrace());
    }
}

pub fn inner_admiral() -> AdmiralResult<()> {
    // let mut admiral = AdmiralClient::new()?;
    // admiral.auth()?;
    // admiral.log("init admiral")?;

    // let mut admiral = AdmiralFile::new()?;

    // admiral.execute_block(BasicLuaBatch {
    //     commands: vec![Box::new(FacDestroy {})],
    // })?;

    // let res = admiral.execute_block(RailStationGenerator {
    //     wagon_size: 8,
    //     start: Point2f { x: 200.0, y: 200.0 },
    // })?;

    // admiral.execute_block(Terapower {
    //     start: Point { x: 0, y: 0 },
    //     height: 600,
    //     width: 600,
    // })?;
    //
    // admiral.execute_block(AssemblerFarmGenerator {
    //     inner: BeaconFarmGenerator {
    //         cell_size: 3,
    //         width: 5,
    //         height: 5,
    //         start: Point2f { x: 200.5, y: 200.5 },
    //         module: "speed-module-3".to_string(),
    //     },
    //     chests: vec![
    //         AssemblerChest::Output { is_purple: false },
    //         AssemblerChest::Output { is_purple: true },
    //         AssemblerChest::Input {
    //             name: "plastic-bar".to_string(),
    //             count: 500,
    //         },
    //         AssemblerChest::Input {
    //             name: "steel-chest".to_string(),
    //             count: 500,
    //         },
    //         AssemblerChest::Buffer {
    //             name: "plastic-bar".to_string(),
    //             count: 500,
    //         },
    //         AssemblerChest::Buffer {
    //             name: "steel-chest".to_string(),
    //             count: 500,
    //         },
    //     ],
    // })?;

    // let origin = Point2f {
    //     x: 1000.0,
    //     y: 1000.0,
    // };
    //
    // let assembler_width = 9;
    // admiral.execute_block(AssemblerRoboFarmGenerator {
    //     start: origin,
    //     row_count: 2,
    //     robo_height: 1,
    //     assembler_width,
    //     assembler_height: 4,
    //     chests: vec![
    //         AssemblerChest::Output { is_purple: false },
    //         AssemblerChest::Output { is_purple: true },
    //         AssemblerChest::Input {
    //             name: "plastic-bar".to_string(),
    //             count: 500,
    //         },
    //         AssemblerChest::Input {
    //             name: "steel-chest".to_string(),
    //             count: 500,
    //         },
    //         AssemblerChest::Buffer {
    //             name: "plastic-bar".to_string(),
    //             count: 500,
    //         },
    //         AssemblerChest::Buffer {
    //             name: "steel-chest".to_string(),
    //             count: 500,
    //         },
    //     ],
    // })?;
    //
    // admiral.execute_block(RailPanGenerator {
    //     width: 15,
    //     height: 25,
    //     start: PointU32 {
    //         x: origin.x as u32,
    //         y: origin.y as u32 - 5,
    //     },
    // })?;

    // admiral.execute_block(RailStationPathfoundGenerator {
    //     start: Point2f {
    //         x: origin.x + (assembler_width * 9) as f32,
    //         y: origin.y - 10.0,
    //     },
    //     station: Point2f {
    //         x: origin.x - 40.0,
    //         y: origin.y - 10.0,
    //     },
    //     pan: Point2f {
    //         x: origin.x + 50.0,
    //         y: origin.y + 160.0,
    //     },
    // })?;

    // admiral.end_file()?;
    Ok(())
}

fn _generate_mega_block(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    // for x in 0..50 {
    //     for y in 0..50 {
    //         let text = admiral.execute_block(FacSurfaceCreateEntitySafe {
    //             inner: FacSurfaceCreateEntity {
    //                 surface_var: "game.surfaces[1]".to_string(),
    //                 position: Point2f::new(1f32 + (x as f32 * 2.0), 1f32 + (y as f32 * 2.0)),
    //                 name: "straight-rail".to_string(),
    //                 params: HashMap::new(),
    //             },
    //         })?;
    //     }
    // }
    //
    // admiral.execute_lua_empty(RailLineGenerator {
    //     length: 200,
    //     rail_loops: 20,
    //     start: Point2f { x: 1f32, y: 1f32 },
    //     separator_every_num: 8,
    // })?;
    //
    // admiral.execute_block(RailBeaconFarmGenerator {
    //     inner: BeaconFarmGenerator {
    //         cell_size: 3,
    //         width: 20,
    //         height: 15,
    //         start: Point2f { x: 200.5, y: 200.5 },
    //     },
    // })?;

    Ok(())
}
