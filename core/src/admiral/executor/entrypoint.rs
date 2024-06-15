use crate::admiral::err::{pretty_panic_admiral, AdmiralResult};
use crate::admiral::executor::client::AdmiralClient;
use crate::admiral::executor::LuaCompiler;
use crate::admiral::generators::rail45::{rail_45_down, rail_45_up};
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
use crate::navigator::mori::{DockFaceDirection, Rail, RailDirection, RailMode, RAIL_STEP_SIZE};
use crate::navigator::path_executor::MINE_FRONT_RAIL_STEPS;
use crate::state::machine_v1::REMOVE_RESOURCE_BASE_TILES;
use crate::surface::pixel::Pixel;
use crate::surfacev::bit_grid::BitGrid;
use crate::surfacev::varea::VArea;
use crate::surfacev::vpoint::{VPoint, SHIFT_POINT_ONE};
use crate::surfacev::vsurface::VSurface;
use crate::LOCALE;
use itertools::Itertools;
use num_format::ToFormattedString;
use opencv::core::Point2f;
use regex::Regex;
use simd_json::{to_owned_value, OwnedValue, StaticNode};
use std::path::Path;
use tracing::{debug, info, trace};

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
        8 => max_command_size_finder(&mut admiral),
        _ => panic!("asdf"),
    }
    .map_err(pretty_panic_admiral)
    .unwrap();
}

fn admiral_entrypoint_prod(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    // let step = "step20-nav";
    let step = "step21-demark";
    let mut surface = VSurface::load(&Path::new("work/out0").join(step))?;
    let radius = surface.get_radius();

    // let patches = surface
    //     .get_mines()
    //     .iter()
    //     // .flat_map(|v| &v.mine_base.patch_indexes)
    //     .collect_vec()
    //     .len();
    // info!("found {} patches", patches);
    // if 1 + 1 == 2 {
    //     return Ok(());
    // }

    scan_area(admiral, radius)?;
    destroy_placed_entities(admiral, radius)?;

    for mine in surface.get_mines_mut() {
        mine.rail.truncate(mine.rail.len() - 5);
    }
    // insert_rail(admiral, &surface)?;
    // insert_electric(admiral, &surface)?;
    // insert_signals(admiral, &surface)?;
    // insert_turn_around_mine(admiral, &surface)?;
    let base_turn_arounds = insert_turn_around_base(admiral, &surface)?;

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
            "roboport",
            "large-electric-pole",
            // "medium-electric-pole",
            // "rail-signal",
            // "steel-chest", "small-lamp"
        ],
    );
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}

fn insert_rail(admiral: &mut AdmiralClient, surface: &VSurface) -> AdmiralResult<()> {
    let mut entities = Vec::new();
    for mine in surface.get_mines() {
        for rail in &mine.rail {
            rail.to_factorio_entities(&mut entities);
        }
    }
    info!("going to insert {} rail entities", entities.len());

    let entities_length = entities.len();
    admiral.execute_checked_commands_in_wrapper_function(entities)?;
    info!("Inserted {} rail", entities_length);

    Ok(())
}

fn insert_turn_around_mine(admiral: &mut AdmiralClient, surface: &VSurface) -> AdmiralResult<()> {
    let mut entities = Vec::new();
    for mine in surface.get_mines() {
        let last_rail = mine.rail.last().unwrap();
        let dock_direction = if last_rail.endpoint.y() > mine.mine_base.area.point_center().y() {
            DockFaceDirection::Down
        } else {
            DockFaceDirection::Up
        };

        last_rail
            .move_force_rotate_clockwise(1)
            .move_forward_single_num(1)
            .move_force_rotate_clockwise(3)
            // todo
            .to_turn_around_factorio_entities(
                &mut entities,
                dock_direction,
                MINE_FRONT_RAIL_STEPS as u32 * RAIL_STEP_SIZE,
            );
    }
    info!("going to insert {} rail entities", entities.len());

    let entities_length = entities.len();
    admiral.execute_checked_commands_in_wrapper_function(entities)?;
    info!("Inserted {} rail", entities_length);
    Ok(())
}

struct TurnArounds {
    positive_columns: [Vec<Rail>; 2],
    negative_columns: [Vec<Rail>; 2],
}

fn insert_turn_around_base(
    admiral: &mut AdmiralClient,
    surface: &VSurface,
) -> AdmiralResult<TurnArounds> {
    let mut base_turn_arounds = TurnArounds {
        negative_columns: [Vec::new(), Vec::new()],
        positive_columns: [Vec::new(), Vec::new()],
    };

    let mut entities = Vec::new();
    let base_rails = surface
        .get_mines()
        .iter()
        .map(|v| &v.rail[0])
        .sorted_by_key(|v| v.endpoint.y())
        .enumerate();

    for (mine_index, base_rail) in base_rails {
        let base_rail = base_rail
            .move_force_rotate_clockwise(2)
            .move_forward_single_num(7);
        let column;
        let turn_around_to_insert;
        if mine_index % 2 == 0 {
            let base_rail = base_rail
                .move_force_rotate_clockwise(1)
                .move_forward_single_num(1)
                .move_force_rotate_clockwise(3);
            base_rail.to_turn_around_factorio_entities(
                &mut entities,
                DockFaceDirection::Up,
                MINE_FRONT_RAIL_STEPS as u32 * RAIL_STEP_SIZE,
            );
            column = 0;
            turn_around_to_insert = base_rail;
        } else {
            let top = rail_45_up(&mut entities, base_rail.endpoint.move_y(-4), 2);
            let bottom = rail_45_up(&mut entities, base_rail.endpoint, 2);

            let mut squeeze_rail = top.move_forward_single_num(3);

            // +1 to go past the curve
            for _ in 0..(MINE_FRONT_RAIL_STEPS + 1) {
                squeeze_rail = squeeze_rail.move_forward_step();
                squeeze_rail.to_factorio_entities(&mut entities);
            }

            //
            squeeze_rail = squeeze_rail
                .move_force_rotate_clockwise(1)
                .move_forward_single_num(1)
                .move_force_rotate_clockwise(3);
            squeeze_rail.to_turn_around_factorio_entities(
                &mut entities,
                DockFaceDirection::Up,
                MINE_FRONT_RAIL_STEPS as u32 * RAIL_STEP_SIZE,
            );
            column = 1;
            turn_around_to_insert = squeeze_rail;
        }
        if turn_around_to_insert.endpoint.y() > 0 {
            base_turn_arounds.positive_columns[column].push(turn_around_to_insert);
        } else {
            base_turn_arounds.negative_columns[column].push(turn_around_to_insert);
        }
    }

    info!("going to insert {} rail entities", entities.len());

    let entities_length = entities.len();
    if 1 + 1 == 3 {
        admiral.execute_checked_commands_in_wrapper_function(entities)?;
    }
    info!("Inserted {} rail", entities_length);
    Ok(base_turn_arounds)
}

fn insert_signals(admiral: &mut AdmiralClient, surface: &VSurface) -> AdmiralResult<()> {
    let mut entities = Vec::new();
    for mine in surface.get_mines() {
        let mut counter = 0;
        for rail in &mine.rail {
            // spacing: no need to densely pack every 8x game rails in straight lines
            if rail.mode.is_turn() || counter % 2 == 0 {
                rail.to_signal_factorio_entities(&mut entities);
            }
            if !rail.mode.is_turn() {
                counter += 1;
            }
        }
    }
    info!("going to insert {} rail entities", entities.len());

    let entities_length = entities.len();
    admiral.execute_checked_commands_in_wrapper_function(entities)?;
    info!("Inserted {} rail", entities_length);

    Ok(())
}

fn insert_electric(admiral: &mut AdmiralClient, surface: &VSurface) -> AdmiralResult<()> {
    let mut entities = Vec::new();
    for mine in surface.get_mines() {
        for rail in &mine.rail {
            rail.to_electric_factorio_entities(&mut entities);
        }
    }
    info!("going to insert {} electric poles", entities.len());

    let entities_length = entities.len();
    admiral.execute_checked_commands_in_wrapper_function(entities)?;
    info!("Inserted {} electric poles", entities_length);

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

fn max_command_size_finder(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    for i in (10_000..).step_by(10000) {
        let mut commands = Vec::new();
        for _ in 0..i {
            commands.push(
                FacSurfaceCreateEntity::new_rail_straight(
                    VPoint::zero().to_f32(),
                    RailDirection::Left,
                )
                .into_boxed(),
            );
        }
        let res = admiral.execute_checked_command(LuaBatchCommand::new(commands).into_boxed())?;
        trace!(
            "counter {} made command size {}",
            i,
            res.lua_text.len().to_formatted_string(&LOCALE)
        );
    }

    Ok(())
}

fn admiral_quick_test(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    const WORK_RADIUS: u32 = 10;

    // scan_area(admiral, WORK_RADIUS)?;
    // destroy_placed_entities(admiral, WORK_RADIUS)?;

    let mut entities = Vec::new();

    entities.push(
        FacDestroy::new_filtered_area(
            VArea::from_arbitrary_points_pair(VPoint::new(2140, -1400), VPoint::new(2300, -1300)),
            vec!["straight-rail", "curved-rail"],
        )
        .into_boxed(),
    );

    let rail = Rail::new_straight(
        VPoint::new(2250, -1345).move_round16_up() + SHIFT_POINT_ONE,
        RailDirection::Left,
    );
    // let rail = Rail::new_straight(
    //     VPoint::new(2143, -1345).move_round16_up() + SHIFT_POINT_ONE,
    //     RailDirection::Right,
    // );
    // rail.to_factorio_entities(&mut entities);

    let rail = rail.move_forward_step();
    // rail.to_factorio_entities(&mut entities);

    rail_45_down(&mut entities, rail.clone().endpoint, 7);
    rail_45_up(&mut entities, rail.endpoint, 7);

    // rail.move_force_rotate_clockwise(1)
    //     .move_forward_single_num(1)
    //     .move_force_rotate_clockwise(3)
    //     // .to_turn_around_factorio_entities(&mut entities, DockFaceDirection::Up, 16);
    //     .to_turn_around_factorio_entities(&mut entities, DockFaceDirection::Up, 16);

    // let rail = rail.move_left();
    // rail.to_factorio_entities(&mut rail_to_place);

    admiral.execute_checked_command(LuaBatchCommand::new(entities).into_boxed())?;

    Ok(())
}
