use std::fmt::Debug;

pub mod chart_pulse;
pub mod checked_command;
pub mod fac_destroy;
pub mod fac_execution_define;
pub mod fac_execution_run;
pub mod fac_log;
pub mod fac_render_destroy;
pub mod fac_render_rect;
pub mod fac_render_text;
pub mod fac_surface_create_entity;
pub mod fac_surface_create_tile;
pub mod lua_batch;
mod lua_syntax;
pub mod raw_lua;
pub mod scanner;
pub mod train_boot;

pub const DEFAULT_SURFACE_VAR: &str = "game.surfaces[1]";
pub const DEFAULT_FORCE_VAR: &str = "game.forces[1]";

/// Main Generator - Nestable commands
pub trait LuaCommand: Debug {
    fn make_lua(&self) -> String;

    fn into_boxed(self) -> Box<dyn LuaCommand>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

pub trait LuaCommandBatch: Debug {
    fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>);
}
