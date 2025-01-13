use std::rc::Rc;

use exhaustive::Exhaustive;
use facto_loop_miner_common::log_init_trace;
use facto_loop_miner_fac_engine::blueprint::bpfac::blueprint::FacBpBlueprintWrapper;
use facto_loop_miner_fac_engine::blueprint::bpitem::BlueprintItem;
use facto_loop_miner_fac_engine::blueprint::converter::encode_blueprint_to_string;
use facto_loop_miner_fac_engine::blueprint::output::FacItemOutput;
use facto_loop_miner_fac_engine::common::names::FacEntityName;
use facto_loop_miner_fac_engine::common::names_tile::FacTileConcreteType;
use facto_loop_miner_fac_engine::common::vpoint::VPOINT_ZERO;
use facto_loop_miner_fac_engine::tests::assembler_tests::{make_assembler_thru, make_industry};
use facto_loop_miner_fac_engine::tests::belt_tests::{
    make_belt_bettel, make_belt_bettel_train_unload, make_belt_combiner, make_belt_grid,
};
use facto_loop_miner_fac_engine::tests::ore_tests::{make_mine, make_mine_and_rail};
use facto_loop_miner_fac_engine::tests::other_tests::make_solar_bath_test;
use facto_loop_miner_fac_engine::tests::train_loop::make_rail_loop;
use facto_loop_miner_fac_engine::tests::train_rails::{
    make_rail_dual_powered, make_rail_dual_turning, make_rail_shift_45, make_rail_spiral_90,
};
use facto_loop_miner_fac_engine::tests::train_station_tests::make_rail_station;
use facto_loop_miner_fac_engine::{
    admiral::{
        err::{AdmiralResult, pretty_panic_admiral},
        executor::client::AdmiralClient,
        lua_command::{LuaCommand, fac_destroy::FacDestroy},
    },
    common::entity::FacEntity,
    game_entities::chest::{FacEntChest, FacEntChestType},
};

fn main() {
    log_init_trace();
    // log_init_debug();

    if let Err(e) = inner_main() {
        let msg = pretty_panic_admiral(e);
        panic!("⛔⛔⛔ DEAD: {}", msg)
    }
}

fn inner_main() -> AdmiralResult<()> {
    let mut client = AdmiralClient::new()?;
    client.auth()?;

    let is_lua = true;

    let output = if is_lua {
        let output = FacItemOutput::new_admiral(client).into_rc();
        execute_destroy(output.clone())?;
        output
    } else {
        FacItemOutput::new_blueprint().into_rc()
    };

    let command_output = output.clone();
    match 8 {
        1 => make_basic(command_output)?,
        2 => make_assembler_thru(command_output),
        3 => make_belt_bettel(command_output)?,
        4 => make_rail_spiral_90(command_output)?,
        5 => make_rail_shift_45(command_output)?,
        6 => make_rail_dual_turning(command_output)?,
        7 => make_rail_dual_powered(command_output)?,
        8 => make_rail_station(command_output)?,
        9 => make_rail_loop(command_output)?,
        10 => make_belt_bettel_train_unload(command_output)?,
        11 => make_belt_combiner(command_output),
        12 => make_belt_grid(command_output),
        13 => make_industry(command_output),
        14 => make_mine(command_output),
        15 => make_solar_bath_test(command_output),
        16 => make_mine_and_rail(command_output),
        _ => panic!("uihhh"),
    }
    output.flush();

    if !is_lua {
        let bpcontents = output.consume_rc().into_blueprint_contents();
        let bp: FacBpBlueprintWrapper = bpcontents.into();
        let res = encode_blueprint_to_string(&bp).unwrap();
        println!("bp {res}");
    }

    Ok(())
}

fn make_basic(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    execute_destroy(output.clone())?;

    output.write(BlueprintItem::new(
        FacEntChest::new(FacEntChestType::Active).into_boxed(),
        VPOINT_ZERO,
    ));

    Ok(())
}

fn execute_destroy(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    let command = FacDestroy::new_filtered(
        150,
        FacEntityName::iter_exhaustive(None)
            .map(|v| v.to_fac_name())
            .collect(),
    );
    // Do not use, this deletes mine resource tiles
    // let command = FacDestroy::new_everything(50);
    output.admiral_execute_command(command.into_boxed())?;

    let command = FacDestroy::new_filtered(
        150,
        FacTileConcreteType::iter_exhaustive(None)
            .map(|v| v.to_fac_name())
            .collect(),
    )
    .into_tiles();
    output.admiral_execute_command(command.into_boxed())?;

    Ok(())
}
