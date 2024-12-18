use facto_loop_miner_common::log_init;
use facto_loop_miner_fac_engine::{
    admiral::{
        err::{AdmiralResult, pretty_panic_admiral},
        executor::{LuaCompiler, client::AdmiralClient},
        lua_command::{LuaCommand, fac_destroy::FacDestroy},
    },
    common::{entity::FacEntity, vpoint::VPoint},
    game_blocks::{assembler_thru::FacBlkAssemblerThru, block::FacBlock},
    game_entities::{
        assembler::FacEntAssembler,
        belt::FacEntBeltType,
        chest::{FacEntChest, FacEntChestType},
        inserter::FacEntInserterType,
        tier::FacTier,
    },
};
use tracing::Level;

fn main() {
    log_init(Some(Level::DEBUG));

    if let Err(e) = inner_main() {
        let msg = pretty_panic_admiral(e);
        panic!("DEAD: {}", msg)
    }
}

fn inner_main() -> AdmiralResult<()> {
    let mut client = AdmiralClient::new()?;
    client.auth()?;

    match 2 {
        1 => make_basic(&mut client)?,
        2 => make_assembler_thru(&mut client)?,
        _ => panic!("uihhh"),
    }

    Ok(())
}

fn make_basic(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    execute_destroy(admiral)?;

    let entity = FacEntChest::new(FacEntChestType::Active);
    admiral.execute_checked_command(entity.to_fac(0, &VPoint::zero()).to_lua().into_boxed())?;

    Ok(())
}

fn make_assembler_thru(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    execute_destroy(admiral)?;

    let farm = FacBlkAssemblerThru {
        assembler: FacEntAssembler::new(FacTier::Tier1, "copper-cable".into(), Default::default()),
        belt_type: FacEntBeltType::Fast,
        inserter_type: FacEntInserterType::Fast,
        width: 2,
        height: 2,
    };
    for entity in farm.generate(VPoint::new(5, 5)) {
        admiral.execute_checked_command(entity.to_blueprint().to_lua().into_boxed())?;
    }

    Ok(())
}

fn execute_destroy(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    // let command = FacDestroy::new_filtered(5, vec![
    //     // "straight-rail",
    //     // "curved-rail",
    //     "roboport",
    //     "substation",
    //     "big-electric-pole",
    //     "small-lamp",
    //     // "rail-signal",
    //     // "steel-chest",
    // ]);
    let command = FacDestroy::new_everything(50);
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}
