use crate::{admiral::lua_command::LuaCommand, common::vpoint::VPoint};

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
    pub ix: u32,
    pub iy: u32,
}

impl XyGridPosition {
    pub fn point(&self) -> VPoint {
        VPoint::new(self.x, self.y)
    }
}

pub fn xy_grid_vpoint(
    start: VPoint,
    width: u32,
    height: u32,
    step: u32,
) -> impl Iterator<Item = XyGridPosition> {
    xy_grid(start.x(), start.y(), width, height, step)
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
