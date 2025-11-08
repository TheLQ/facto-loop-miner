#![feature(int_roundings)]
#![feature(error_generic_member_access)]
#![feature(iter_array_chunks)]
#![feature(split_array)]
// // super pendantic
// #![warn(clippy::all, clippy::pedantic)]
// #![allow(clippy::missing_panics_doc)]
// #![allow(clippy::missing_errors_doc)]
// #![allow(clippy::module_inception)]
// #![allow(clippy::must_use_candidate)]
// #![allow(clippy::new_without_default)]
// #![allow(clippy::semicolon_if_nothing_returned)]
// #![allow(clippy::doc_markdown)]
// #![allow(clippy::default_trait_access)]
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
