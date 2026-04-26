use crate::common::vpoint::VPoint;
use crate::game_blocks::rail_hope_single::HopeLinkType;
use crate::util::slice_pusher::ArrayPusher;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

pub trait RailHopeAppender {
    fn add_straight(&mut self, length: usize);

    fn add_straight_section(&mut self);

    fn add_turn90(&mut self, clockwise: bool);

    fn add_shift45(&mut self, clockwise: bool, length: usize);

    fn pos_next(&self) -> VPoint;
}

/// Superfast magic
///
/// For turn90,
///
/// top is [crate::game_blocks::rail_hope_soda::create_turn_link_from]
/// ```
///   + straight2 (2x2 rail x 2 straight = 8 pixels)
///   + turn90 (44 pixels)
///   + straight2 (2x2 rail x 2 straight = 8 pixels)
///   = 60
/// ```
/// bottom is
/// ```
///   + turn90 (44 pixels)
///   = 104
/// ```
///
/// For straight_section,
///
/// top and bottom is [crate::game_blocks::rail_hope_soda::HopeSodaLink::add_straight_section]
/// ```
///    + straight13 (2x2 rail x 13)
///    = 52
///    * 2
///    = 104
/// ```
pub const SUPERFAST_POINTS_SIZE: usize = 104;

pub trait RailHopeLink {
    type AreaInput<'a>;

    fn add_straight(&self, length: usize) -> Self;

    fn add_straight_section(&self) -> Self;

    fn add_turn90(&self, clockwise: bool) -> Self;

    fn add_shift45(&self, clockwise: bool, length: usize) -> Self;

    fn link_type(&self) -> HopeLinkType;

    fn pos_start(&self) -> VPoint;

    fn pos_next(&self) -> VPoint;

    fn area(&self, output: &mut Self::AreaInput<'_>);
}
