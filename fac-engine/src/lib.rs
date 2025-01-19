#![feature(int_roundings)]
#![feature(error_generic_member_access)]
#![feature(iter_array_chunks)]
// Prefer explicit
#![allow(clippy::new_without_default)]
#![warn(clippy::inconsistent_struct_constructor)]

pub mod admiral;
pub mod blueprint;
pub mod common;
pub mod constants;
pub mod err;
pub mod game_blocks;
pub mod game_entities;
pub mod tests;
pub mod util;
pub mod visualizer;

pub use opencv as opencv_re;
