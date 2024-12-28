use std::rc::Rc;

use exhaustive::Exhaustive;
use facto_loop_miner_common::log_init_trace;
use facto_loop_miner_fac_engine::blueprint::bpitem::BlueprintItem;
use facto_loop_miner_fac_engine::blueprint::output::FacItemOutput;
use facto_loop_miner_fac_engine::common::names::FacEntityName;
use facto_loop_miner_fac_engine::common::vpoint::VPOINT_ZERO;
use facto_loop_miner_fac_engine::tests::assembler::make_assembler_thru;
use facto_loop_miner_fac_engine::tests::belt_tests::{
    make_belt_bettel, make_belt_bettel_train_unload, make_belt_combiner,
};
use facto_loop_miner_fac_engine::tests::train_loop::make_rail_loop;
use facto_loop_miner_fac_engine::tests::train_rails::{
    make_rail_dual_powered, make_rail_dual_turning, make_rail_shift_45, make_rail_spiral_90,
};
use facto_loop_miner_fac_engine::tests::train_station::make_rail_station;
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

    let output = FacItemOutput::new_admiral(client).into_rc();
    execute_destroy(output.clone())?;

    match 11 {
        1 => make_basic(output)?,
        2 => make_assembler_thru(output)?,
        3 => make_belt_bettel(output)?,
        4 => make_rail_spiral_90(output)?,
        5 => make_rail_shift_45(output)?,
        6 => make_rail_dual_turning(output)?,
        7 => make_rail_dual_powered(output)?,
        8 => make_rail_station(output)?,
        9 => make_rail_loop(output)?,
        10 => make_belt_bettel_train_unload(output)?,
        11 => make_belt_combiner(output),
        _ => panic!("uihhh"),
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

    Ok(())
}
