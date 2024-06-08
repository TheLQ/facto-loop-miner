// #![feature(cell_update)]
// #![feature(convert_float_to_int)]
#![feature(iter_array_chunks)]
#![feature(portable_simd)]
// #![feature(slice_pattern)]
#![feature(iterator_try_collect)]
// #![feature(iterator_try_reduce)]
// #![feature(slice_flatten)]
#![feature(extract_if)]
#![feature(array_chunks)]
#![feature(error_generic_member_access)]
#![feature(int_roundings)]
#![feature(option_get_or_insert_default)]
#![feature(const_trait_impl)]
// lints
#![allow(unused_variables)]
#![allow(dead_code)]
// todo: This is for something() { Ok(()) } , only testing
#![allow(clippy::unnecessary_wraps)]
// styling
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::missing_errors_doc)]
// TODO: <<<< SHOULD REVIEW
#![allow(clippy::cast_precision_loss)]
// #![deny(clippy::pedantic)]
// #![deny(clippy::all)]
#![allow(clippy::let_and_return)]
#![allow(clippy::result_large_err)]
#![feature(test)]

// TODO #![deny(future-incompatible)]
// TODO #![deny(let-underscore)]
// TODO #![deny(nonstandard-style)]

extern crate core;

use crate::admiral::executor::client::AdmiralClient;
use crate::admiral::executor::entrypoint::admiral_entrypoint;
use crate::navigator::mori::{draw_rail, Rail, RailDirection};
use crate::state::machine_v1::new_v1_machine;
use crate::surface::pixel::generate_lookup_image;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use kiddo::float;
use num_format::Locale;
use num_traits::PrimInt;
use std::path::Path;
use tracing::Level;

mod admiral;
mod gamedata;
pub mod navigator;
mod opencv;
pub mod simd;
pub mod simd_diff;
pub mod state;
pub mod surface;
pub mod surfacev;
pub mod util;

// pub type PixelKdTree = KdTree<f32, 2usize>;
pub type PixelKdTree = float::kdtree::KdTree<f32, usize, 2usize, 32, u32>;

pub const LOCALE: Locale = Locale::en;
pub const TILES_PER_CHUNK: usize = 32;
pub fn inner_main() {
    let tracing_format = tracing_subscriber::fmt::format().compact();

    log_init(None);

    tracing::debug!("hello");
    // let mut data = String::new();
    // stdin().read_line(&mut data).expect("asd");

    let root_dir = Path::new("work");

    match 1 {
        1 => new_v1_machine().start(root_dir),
        3 => generate_lookup_image(),
        5 => admiral::client::admiral(),
        _ => panic!("wtf"),
    }
}

pub fn rcon_inner_main() {
    let tracing_format = tracing_subscriber::fmt::format().compact();
    log_init(Some(Level::TRACE));

    let mut admiral = AdmiralClient::new().unwrap();
    admiral.auth().unwrap();

    admiral_entrypoint(admiral);
}

pub fn inner_surface_tester() {
    let tracing_format = tracing_subscriber::fmt::format().compact();
    log_init(None);

    let mut surface = VSurface::load(Path::new("./work/out0/step20-nav")).unwrap();

    // Rail { endpoint: VPoint { x: 649, y: 49 }, direction: Right, mode: Straight } to Rail { endpoint: VPoint { x: 2273, y: 121 }, direction: Right, mode: Straight }
    let start = Rail::new_straight(VPoint::new(649, 49), RailDirection::Right);
    let end = Rail::new_straight(VPoint::new(2273, 121), RailDirection::Right);

    let rail = end;
    let rail = rail.move_force_rotate_clockwise(2);
    draw_rail(&mut surface, &rail);

    // let rail = rail.move_forward_step();
    // draw_rail(&mut surface, &rail);

    let rail = rail.move_right();
    draw_rail(&mut surface, &rail);

    let out_dir = Path::new("./work/test5");
    std::fs::create_dir(out_dir).unwrap();
    surface.save(out_dir).unwrap()
}

/// what is this called?
pub fn bucket_div<N>(value: N, bucket_size: N) -> N
where
    N: PrimInt,
{
    (value - (value % bucket_size)) / bucket_size
}

pub fn log_init(force_level: Option<Level>) {
    tracing_subscriber::fmt()
        .with_max_level(if let Some(level) = force_level {
            level
        } else {
            Level::INFO
        })
        .compact()
        .init();
}
