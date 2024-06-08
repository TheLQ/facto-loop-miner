use crate::admiral::err::{pretty_panic_admiral, AdmiralResult};
use crate::admiral::executor::client::AdmiralClient;
use crate::admiral::executor::LuaCompiler;
use crate::admiral::generators::rail90::{
    dual_rail_east, dual_rail_east_empty, dual_rail_north, dual_rail_north_empty, dual_rail_south,
    dual_rail_south_empty, dual_rail_west, dual_rail_west_empty, rail_degrees_east,
    rail_degrees_north, rail_degrees_south, rail_degrees_west,
};
use crate::admiral::lua_command::chart_pulse::ChartPulse;
use crate::admiral::lua_command::fac_destroy::FacDestroy;
use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
use crate::admiral::lua_command::lua_batch::LuaBatchCommand;
use crate::admiral::lua_command::raw_lua::RawLuaCommand;
use crate::admiral::lua_command::scanner::BaseScanner;
use crate::admiral::lua_command::LuaCommand;
use crate::admiral::mine_builder::admiral_mines;
use crate::navigator::mori::{Rail, RailDirection};
use crate::state::machine_v1::REMOVE_RESOURCE_BASE_TILES;
use crate::surface::pixel::Pixel;
use crate::surfacev::bit_grid::BitGrid;
use crate::surfacev::vpoint::{VPoint, SHIFT_POINT_ONE};
use crate::surfacev::vsurface::VSurface;
use itertools::Itertools;
use opencv::core::Point2f;
use regex::Regex;
use simd_json::{to_owned_value, OwnedValue, StaticNode};
use std::path::Path;
use tracing::{debug, info};

pub fn admiral_entrypoint(mut admiral: AdmiralClient) {
    info!("admiral entrypoint");

    match 2 {
        1 => admiral_entrypoint_draw_rail_8(&mut admiral),
        2 => admiral_entrypoint_prod(&mut admiral),
        3 => admiral_entrypoint_turn_area_extractor(&mut admiral),
        4 => admiral_entrypoint_turn_viewer(&mut admiral),
        5 => admiral_quick_test(&mut admiral),
        6 => validate_patches(&mut admiral),
        7 => admiral_mines(&mut admiral),
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

    // let step = "step20-nav";
    let step = "step21-demark";
    let surface = VSurface::load(&Path::new("work/out0").join(step))?;
    let radius = surface.get_radius();

    scan_area(admiral, radius)?;
    destroy_placed_entities(admiral, radius)?;

    insert_rail_from_surface(admiral, &surface)?;

    chart_pulse(admiral, radius)?;

    Ok(())
}

fn validate_patches(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    let step = "step21-demark";
    let surface = VSurface::load(&Path::new("work/out0").join(step))?;

    let raw_lua_base = r#"
    bad = 0
    good = 0
    local function test_pos(x,y,name,track)
        local actual = game.surfaces[1].find_entity(name, {x+0.5,y+0.5})
        if actual == nil then
            bad = bad + 1
            local names = {}
            for _,v in ipairs(game.surfaces[1].find_entities({ {x,y}, {x+1,y+1} })) do
                table.insert(names, v.name)
            end
            local actual = game.table_to_json(names)
            rcon.print("pos " .. x .. "," .. y .. " expected " .. name .. " actual " .. actual)
        else
            good = good + 1
        end
    end

    "#
    .replace("\n", " ");
    let mut command = Regex::new("\\s+")
        .unwrap()
        .replace_all(&raw_lua_base, " ")
        .to_string();
    for patch in surface.get_patches_slice() {
        for pixel in &patch.pixel_indexes {
            command.push_str(&format!(
                "test_pos({},{},\"{}\") ",
                pixel.x(),
                pixel.y(),
                patch.resource.to_facto_string().unwrap()
            ));
        }
    }
    command.push_str(r#"rcon.print("good " .. good .. " bad " .. bad)"#);

    debug!("{}", &command[0..2000]);
    let command = RawLuaCommand::new(command);
    let res = admiral._execute_statement(command).unwrap();
    info!("{}", res.body);

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
        vec![
            // "straight-rail",
            // "curved-rail",
            "medium-electric-pole",
            // "steel-chest", "small-lamp"
        ],
    );
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}

fn insert_rail_from_surface(admiral: &mut AdmiralClient, surface: &VSurface) -> AdmiralResult<()> {
    let mut entities = Vec::new();

    for rail in surface.get_rail_TODO() {
        // info!("writing {:?}", rail);
        rail.to_factorio_entities(&mut entities);
        rail.to_tracking_factorio_entities(&mut entities);
    }
    info!("going to insert {} rail entities", entities.len());

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

pub fn chart_pulse(admiral: &mut AdmiralClient, radius: u32) -> AdmiralResult<()> {
    let command = ChartPulse::new_radius(radius);
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}

fn admiral_entrypoint_draw_rail_8(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    const WORK_RADIUS: u32 = REMOVE_RESOURCE_BASE_TILES as u32 + 20;

    scan_area(admiral, WORK_RADIUS)?;
    destroy_placed_entities(admiral, WORK_RADIUS)?;

    {
        let command = RawLuaCommand::new("rendering.clear()".to_string());
        admiral.execute_checked_command(command.into_boxed())?;

        let mut rails = Vec::new();

        let rail = Rail::new_straight(VPoint::new(65, 65), RailDirection::Right);
        rails.push(rail.clone());

        {
            let rail = rail.move_left();
            rails.push(rail.clone());

            let rail = rail.move_forward_step();
            rails.push(rail.clone());

            let rail = rail.move_left();
            rails.push(rail.clone());

            let rail = rail.move_forward_step();
            rails.push(rail.clone());

            let rail = rail.move_left();
            rails.push(rail.clone());

            let rail = rail.move_forward_step();
            rails.push(rail.clone());

            let rail = rail.move_left();
            rails.push(rail.clone());
        }

        {
            let rail = rail.move_right();
            rails.push(rail.clone());

            let rail = rail.move_forward_step();
            rails.push(rail.clone());

            let rail = rail.move_right();
            rails.push(rail.clone());

            let rail = rail.move_forward_step();
            rails.push(rail.clone());

            let rail = rail.move_right();
            rails.push(rail.clone());

            let rail = rail.move_forward_step();
            rails.push(rail.clone());

            let rail = rail.move_right();
            rails.push(rail.clone());
        }

        let mut entities = Vec::new();
        for rail in &rails {
            rail.to_factorio_entities(&mut entities);
        }
        admiral.execute_checked_command(LuaBatchCommand::new(entities).into_boxed())?;

        for rail in rails {
            info!("-----");
            // let mut entities = Vec::new();
            // rail.to_factorio_entities(&mut entities);
            // for entity in entities {
            //     admiral.execute_checked_command(entity)?;
            // }

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

pub fn admiral_entrypoint_turn_viewer(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    const WORK_RADIUS: u32 = REMOVE_RESOURCE_BASE_TILES as u32 + 20;

    scan_area(admiral, WORK_RADIUS)?;
    destroy_placed_entities(admiral, WORK_RADIUS)?;

    {
        let command = rail_degrees_south(VPoint::new(0, 0) + SHIFT_POINT_ONE);
        let command = LuaBatchCommand::new(command);
        admiral.execute_checked_command(command.into_boxed())?;

        let command = rail_degrees_west(VPoint::new(32, 0) + SHIFT_POINT_ONE);
        let command = LuaBatchCommand::new(command);
        admiral.execute_checked_command(command.into_boxed())?;

        let command = rail_degrees_north(VPoint::new(64, 0) + SHIFT_POINT_ONE);
        let command = LuaBatchCommand::new(command);
        admiral.execute_checked_command(command.into_boxed())?;

        let command = rail_degrees_east(VPoint::new(96, 0) + SHIFT_POINT_ONE);
        let command = LuaBatchCommand::new(command);
        admiral.execute_checked_command(command.into_boxed())?;
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
    dual_rail_north(
        VPoint::new(chunk_x_offset, chunk_y_offset) + SHIFT_POINT_ONE,
        &mut commands,
    );

    let chunk_x_offset = 32;
    dual_rail_south(
        VPoint::new(chunk_x_offset, chunk_y_offset) + SHIFT_POINT_ONE,
        &mut commands,
    );

    let chunk_x_offset = 64;
    dual_rail_east(
        VPoint::new(chunk_x_offset, chunk_y_offset) + SHIFT_POINT_ONE,
        &mut commands,
    );

    let chunk_x_offset = 96;
    dual_rail_west(
        VPoint::new(chunk_x_offset, chunk_y_offset) + SHIFT_POINT_ONE,
        &mut commands,
    );

    match 1 {
        1 => create_minified_bitgrid(admiral, commands, chunk_y_offset),
        2 => insert_minified_kit(admiral, commands, chunk_y_offset),
        _ => panic!("asdf"),
    }?;

    Ok(())
}

pub fn create_minified_bitgrid(
    admiral: &mut AdmiralClient,
    mut commands: Vec<Box<dyn LuaCommand>>,
    chunk_y_offset: i32,
) -> AdmiralResult<()> {
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

    // // magic!
    // for turn_grid in turn_grids {
    //     println!();
    //     println!("Numbay");
    //     for info in turn_grid.to_hex_strings() {
    //         println!("{}", info);
    //     }
    // }

    // less magic
    for turn_grid in turn_grids {
        println!("{}", turn_grid.to_array_string());
    }

    Ok(())
}

fn insert_minified_kit(
    admiral: &mut AdmiralClient,
    mut commands: Vec<Box<dyn LuaCommand>>,
    chunk_y_offset: i32,
) -> AdmiralResult<()> {
    for (chunk_x_offset, grid) in [
        (0, dual_rail_south_empty()),
        (32, dual_rail_north_empty()),
        (64, dual_rail_east_empty()),
        (96, dual_rail_west_empty()),
    ] {
        for lamp_x_offset in 0..16 {
            for lamp_y_offset in 0..16 {
                if !grid.get(lamp_x_offset, lamp_y_offset) {
                    continue;
                }
                commands.push(
                    FacSurfaceCreateEntity::new(
                        "steel-chest",
                        Point2f {
                            x: (chunk_x_offset + lamp_x_offset) as f32,
                            y: (chunk_y_offset + lamp_y_offset as i32) as f32,
                        },
                    )
                    .into_boxed(),
                )
            }
        }
    }

    admiral.execute_checked_command(LuaBatchCommand::new(commands).into_boxed())?;

    Ok(())
}

fn admiral_quick_test(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    const WORK_RADIUS: u32 = 10;

    scan_area(admiral, WORK_RADIUS)?;
    destroy_placed_entities(admiral, WORK_RADIUS)?;

    let mut rail_to_place = Vec::new();

    let rail = Rail::new_straight(VPoint::new(5, 5), RailDirection::Left);
    rail.to_factorio_entities(&mut rail_to_place);

    let rail = rail.move_left();
    rail.to_factorio_entities(&mut rail_to_place);

    admiral.execute_checked_command(LuaBatchCommand::new(rail_to_place).into_boxed())?;

    Ok(())
}
