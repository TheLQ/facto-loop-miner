use crate::admiral::err::{pretty_panic_admiral, AdmiralResult};
use crate::admiral::executor::client::AdmiralClient;
use crate::admiral::executor::LuaCompiler;
use crate::admiral::generators::rail90::{
    dual_rail_east, dual_rail_north, dual_rail_south, dual_rail_west, rail_degrees_east,
    rail_degrees_north, rail_degrees_south, rail_degrees_west,
};
use crate::admiral::lua_command::chart_pulse::ChartPulse;
use crate::admiral::lua_command::fac_destroy::FacDestroy;
use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
use crate::admiral::lua_command::lua_batch::LuaBatchCommand;
use crate::admiral::lua_command::raw_lua::RawLuaCommand;
use crate::admiral::lua_command::scanner::{facscan_mega_export_entities_compressed, BaseScanner};
use crate::admiral::lua_command::LuaCommand;
use crate::navigator::mori::{Rail, RailDirection, RailMode};
use crate::state::machine_v1::REMOVE_RESOURCE_BASE_TILES;
use crate::surfacev::bit_grid::BitGrid;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use bitvec::prelude::*;
use bitvec::vec::BitVec;
use opencv::core::Point2f;
use regex::Replacer;
use simd_json::{to_owned_value, to_string, OwnedValue, StaticNode};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

pub fn admiral_entrypoint(mut admiral: AdmiralClient) {
    info!("admiral entrypoint");

    match 3 {
        1 => admiral_entrypoint_testing(&mut admiral),
        2 => admiral_entrypoint_prod(&mut admiral),
        3 => admiral_entrypoint_turn_area_extractor(&mut admiral),
        _ => panic!("asdf"),
    }
    .map_err(pretty_panic_admiral)
    .unwrap();

    // validate we have space for us

    // for command in facscan_hyper_scan() {
    //     let res = admiral._execute_statement(command).unwrap();
    //     info!("return: {}", res.body);
    // }
    // for command in facscan_mega_export_entities_compressed() {
    //     let res = admiral._execute_statement(command).unwrap();
    //     info!("return: {}", res.body);
    // }
    // let res = admiral
    //     ._execute_statement(FacLog::new("done".to_string()))
    //     .unwrap();
    // info!("return: {}", res.body);
}

fn admiral_entrypoint_prod(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    // if 1 + 1 == 2 {
    //     return Ok(());
    // }

    let surface = VSurface::load(Path::new("work/out0/step20-nav"))?;
    let radius = surface.get_radius();

    scan_area(admiral, radius)?;
    destroy_placed_entities(admiral, radius)?;

    insert_rail_from_surface(admiral, &surface)?;

    chart_pulse(admiral, radius)?;

    Ok(())
}

fn scan_area(admiral: &mut AdmiralClient, radius: u32) -> AdmiralResult<()> {
    // Need to have generated space for our testing
    let command = BaseScanner::new_radius(radius);
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}

fn destroy_placed_entities(admiral: &mut AdmiralClient, radius: u32) -> AdmiralResult<()> {
    let command = FacDestroy::new_filtered(
        radius,
        vec!["straight-rail", "curved-rail", "steel-chest", "small-lamp"],
    );
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}

fn insert_rail_from_surface(admiral: &mut AdmiralClient, surface: &VSurface) -> AdmiralResult<()> {
    let mut entities = Vec::new();

    for rail in surface.get_rail() {
        // if !rail
        //     .endpoint
        //     .is_within_center_radius(REMOVE_RESOURCE_BASE_TILES as u32 + 20)
        // {
        //     continue;
        // }
        if rail.mode != RailMode::Straight {
            continue;
        }
        info!("writing {:?}", rail);

        rail.to_factorio_entities(&mut entities);
    }

    let entities_length = entities.len();
    admiral.execute_checked_commands_in_wrapper_function(entities)?;
    info!("Inserted {} rail", entities_length);

    // let command = FacSurfaceCreateEntity::new_rail_straight(
    //     rail.endpoint.to_f32_with_offset(1.0),
    //     rail.direction.clone(),
    // );
    // admiral.execute_checked_command(command.into_boxed())?;
    Ok(())
}

fn chart_pulse(admiral: &mut AdmiralClient, radius: u32) -> AdmiralResult<()> {
    let command = ChartPulse::new_radius(radius);
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}

fn admiral_entrypoint_testing(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    const WORK_RADIUS: u32 = REMOVE_RESOURCE_BASE_TILES as u32 + 20;

    scan_area(admiral, WORK_RADIUS)?;
    destroy_placed_entities(admiral, WORK_RADIUS)?;

    // {
    //     let command = rail_degrees_south(VPoint::new(0, 0).to_f32_with_offset(0.0));
    //     let command = LuaBatchCommand::new(Vec::from(command));
    //     admiral.execute_checked_command(command.into_boxed())?;
    //
    //     let command = rail_degrees_west(VPoint::new(32, 0).to_f32_with_offset(0.0));
    //     let command = LuaBatchCommand::new(Vec::from(command));
    //     admiral.execute_checked_command(command.into_boxed())?;
    //
    //     let command = rail_degrees_north(VPoint::new(64, 0).to_f32_with_offset(0.0));
    //     let command = LuaBatchCommand::new(Vec::from(command));
    //     admiral.execute_checked_command(command.into_boxed())?;
    //
    //     let command = rail_degrees_east(VPoint::new(96, 0).to_f32_with_offset(0.0));
    //     let command = LuaBatchCommand::new(Vec::from(command));
    //     admiral.execute_checked_command(command.into_boxed())?;
    // }

    {
        let command = RawLuaCommand::new("rendering.clear()".to_string());
        admiral.execute_checked_command(command.into_boxed())?;

        let mut rails = Vec::new();

        let rail = Rail::new_straight(VPoint::new(64, 64), RailDirection::Down);
        rails.push(rail.clone());

        let rail = rail.move_right();
        rails.push(rail.clone());

        let rail = rail.move_forward_step();
        rails.push(rail.clone());

        for rail in rails {
            info!("-----");
            let mut entities = Vec::new();
            rail.to_factorio_entities(&mut entities);
            for entity in entities {
                admiral.execute_checked_command(entity)?;
            }

            let command = RawLuaCommand::new(format!(
                "rendering.draw_rectangle{{ \
            surface = game.surfaces[1], \
            left_top = {{ {}, {} }}, \
            right_bottom =  {{ {}, {} }}, \
            color = {{ 1, 1, 1 }} }}",
                rail.endpoint.x() - 1,
                rail.endpoint.y() - 1,
                rail.endpoint.x() + 1,
                rail.endpoint.y() + 1
            ));
            admiral.execute_checked_command(command.into_boxed())?;

            info!("-----");
        }
    }

    Ok(())
}

pub fn admiral_entrypoint_turn_area_extractor(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    const WORK_RADIUS: u32 = 200;

    scan_area(admiral, WORK_RADIUS)?;
    destroy_placed_entities(admiral, WORK_RADIUS)?;

    let mut commands = Vec::new();

    let chunk_x_offset = 0;
    let chunk_y_offset = -64;
    dual_rail_north(VPoint::new(chunk_x_offset, chunk_y_offset), &mut commands);

    let chunk_x_offset = 32;
    dual_rail_south(VPoint::new(chunk_x_offset, chunk_y_offset), &mut commands);

    let chunk_x_offset = 64;
    dual_rail_east(VPoint::new(chunk_x_offset, chunk_y_offset), &mut commands);

    let chunk_x_offset = 96;
    dual_rail_west(VPoint::new(chunk_x_offset, chunk_y_offset), &mut commands);

    for chunk_x_offset in [0, 32, 64, 96] {
        for lamp_x_offset in 0..16 {
            for lamp_y_offset in 0..16 {
                let lamp_x = chunk_x_offset + lamp_x_offset;
                let lamp_y = -64 + lamp_y_offset;

                let command = RawLuaCommand::new(format!("\
                if game.surfaces[1].can_place_entity{{ name=\"steel-chest\",position={{ {lamp_x},{lamp_y} }} }} then \
                game.surfaces[1].create_entity{{ name=\"steel-chest\",position={{ {lamp_x},{lamp_y} }} }}\
                end\
                "));
                commands.push(command.into_boxed());
            }
        }
    }

    admiral.execute_checked_command(LuaBatchCommand::new(commands).into_boxed())?;

    // fetch position of steel crates

    let command = RawLuaCommand::new(
        r#"
    local entities = game.surfaces[1].find_entities_filtered{
        area = {{0, -64}, {128, -48}}, 
        name = "steel-chest"
    }
    local output = {}
    for _, entity in ipairs(entities) do
        table.insert(output, entity.position.x)
        table.insert(output, entity.position.y)
    end
    rcon.print(game.table_to_json(output))
    "#
        .trim()
        .replace('\n', ""),
    );

    let response = admiral._execute_statement(command).unwrap();

    let mut body = response.body.into_bytes();
    let main_array = if let Ok(OwnedValue::Array(raw)) = to_owned_value(body.as_mut()) {
        raw
    } else {
        panic!("no wrapper array?")
    };

    let mut chest_positions = Vec::new();
    for [x_value, y_value] in main_array.into_iter().array_chunks() {
        let x = if let OwnedValue::Static(StaticNode::F64(raw)) = x_value {
            raw as f32
        } else {
            panic!("not x");
        };
        let y = if let OwnedValue::Static(StaticNode::F64(raw)) = y_value {
            raw as f32
        } else {
            panic!("not y");
        };
        chest_positions.push(VPoint::from_f32_with_offset(Point2f { x, y }, 0.5)?);
    }
    info!("Loaded {} chests", chest_positions.len());

    // bucketize chest points
    let mut turn_grids = [
        BitGrid::new(),
        BitGrid::new(),
        BitGrid::new(),
        BitGrid::new(),
    ];

    for chest in chest_positions {
        let (turn, relative_chest) = match chest.x() {
            0..16 => (0, chest.move_xy(0, -chunk_y_offset)),
            32..48 => (1, chest.move_xy(-32, -chunk_y_offset)),
            64..80 => (2, chest.move_xy(-64, -chunk_y_offset)),
            96..112 => (3, chest.move_xy(-96, -chunk_y_offset)),
            _ => panic!("asdf {:?}", chest),
        };
        turn_grids[turn].set(
            relative_chest.x() as usize,
            relative_chest.y() as usize,
            true,
        );
    }

    // magic!
    for turn_grid in turn_grids {
        println!();
        println!("Numbay");
        for info in turn_grid.to_hex_strings() {
            println!("{}", info);
        }
    }

    Ok(())
}
