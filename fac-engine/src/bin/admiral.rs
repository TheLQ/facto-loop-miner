use facto_loop_miner_common::log_init;
use facto_loop_miner_fac_engine::{
    admiral::{
        err::{AdmiralResult, pretty_panic_admiral},
        executor::{LuaCompiler, client::AdmiralClient},
        lua_command::{LuaCommand, fac_destroy::FacDestroy},
    },
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::chest::{FacEntChest, FacEntChestType},
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

    match 1 {
        1 => make_basic(&mut client)?,
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

fn execute_destroy(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    let command = FacDestroy::new_filtered(5, vec![
        // "straight-rail",
        // "curved-rail",
        "roboport",
        "substation",
        "big-electric-pole",
        "small-lamp",
        // "rail-signal",
        // "steel-chest",
    ]);
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}
