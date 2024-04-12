use facto_loop_miner::navigator::mori::{draw_rail, format_surface_dump, Rail, RailDirection};
use facto_loop_miner::surfacev::vpoint::VPoint;
use facto_loop_miner::surfacev::vsurface::VSurface;

fn main() {
    const TEST_RADIUS: usize = 50;
    let mut surface = VSurface::new(TEST_RADIUS as u32);

    let turn_rail = Rail::new_straight(VPoint::new(-6, 2), RailDirection::Down);
    draw_rail(&mut surface, &turn_rail);

    let turn_rail = turn_rail.move_left();
    // let turn_rail = turn_rail.move_right();
    draw_rail(&mut surface, &turn_rail);

    let turn_rail = turn_rail.move_forward_step();
    draw_rail(&mut surface, &turn_rail);

    let res = format_surface_dump(&surface);

    println!("{}", res);
}
