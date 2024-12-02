use crate::admiral::lua_command::LuaCommand;

#[derive(Debug)]
pub struct ChartPulse {
    radius: u32,
}

impl ChartPulse {
    pub fn new_radius(radius: u32) -> Self {
        Self { radius }
    }

    // pub fn new(surface: &VSurface) -> Self {
    //     Self::new_radius(surface.get_radius())
    // }
}

impl LuaCommand for ChartPulse {
    fn make_lua(&self) -> String {
        let radius = self.radius;
        format!(
            "game.forces[1].chart(game.surfaces[1], {{ {{ -{radius}, -{radius} }}, {{ {radius}, {radius} }} }})"
        )
    }
}
