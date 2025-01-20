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
