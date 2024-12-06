use crate::{admiral::lua_command::LuaCommand, common::vpoint::VPoint};

pub mod assembler_farm;
pub mod assembler_robo_farm;
pub mod beacon_farm;
// pub mod rail45;
// pub mod rail90;
pub mod rail_beacon_farm;
pub mod rail_line;
pub mod rail_pan;
pub mod rail_station;
pub mod terapower;

pub fn join_commands<'a>(
    creation_commands: impl Iterator<Item = &'a Box<dyn LuaCommand>>,
) -> String {
    let mut result = "".to_string();
    for command in creation_commands {
        result.push_str(&command.make_lua());
        result.push('\n');
    }
    result
}

pub struct XyGridPosition {
    x: i32,
    y: i32,
    ix: u32,
    iy: u32,
}

impl XyGridPosition {
    pub fn to_vpoint(&self) -> VPoint {
        VPoint::new(self.x, self.y)
    }
}

pub fn xy_grid(
    start_x: i32,
    start_y: i32,
    width: u32,
    height: u32,
    step: u32,
) -> impl Iterator<Item = XyGridPosition> {
    let mut res = Vec::new();
    for ix in 0..width {
        for iy in 0..height {
            res.push(XyGridPosition {
                x: start_x + (ix * step) as i32,
                y: start_y + (iy * step) as i32,
                ix,
                iy,
            })
        }
    }
    res.into_iter()
}
