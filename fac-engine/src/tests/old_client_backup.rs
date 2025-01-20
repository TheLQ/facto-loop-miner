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
