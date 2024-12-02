// use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
// use crate::admiral::lua_command::fac_surface_create_entity_safe::FacSurfaceCreateEntitySafe;
// use crate::admiral::lua_command::{
//     direction_params, LuaCommand, LuaCommandBatch, DEFAULT_SURFACE_VAR,
// };
// use crate::navigator::mori::{Rail, RailDirection};
// use crate::surface::surface::PointU32;
// use crate::common::cvpoint::Point2f;
//
// #[derive(Debug)]
// pub struct RailPanGenerator {
//     pub start: PointU32,
//     pub width: u32,
//     pub height: u32,
// }
//
// impl LuaCommandBatch for RailPanGenerator {
//     fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
//         let mut rail = Rail::new_straight(self.start, RailDirection::Right);
//
//         // forward across base
//         for _ in 0..self.width {
//             rail = rail.move_forward().unwrap();
//             push_rail_to_lua(&rail, lua_commands);
//         }
//         rail = rail.move_force_rotate_clockwise(3);
//
//         // going down the back
//         for _ in 0..self.height {
//             match rail.move_forward() {
//                 Some(v) => rail = v,
//                 None => panic!("can't move forward, prev {:?}", rail),
//             }
//             push_rail_to_lua(&rail, lua_commands);
//         }
//         rail = rail.move_force_rotate_clockwise(3);
//
//         // backward across base
//         for _ in 0..self.width {
//             rail = rail.move_forward().unwrap();
//             push_rail_to_lua(&rail, lua_commands);
//         }
//         rail = rail.move_force_rotate_clockwise(3);
//
//         // backward up base
//         // for _ in 0..self.height {
//         //     rail = rail.move_forward().unwrap();
//         //     push_rail_to_lua(&rail, lua_commands);
//         // }
//         // rail = rail.move_force_rotate_clockwise(3);
//     }
// }
//
// fn push_rail_to_lua(rail: &Rail, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
//     for i in 0..6 {
//         let cur_rail = rail.move_force_rotate_clockwise(2).move_force_forward(i);
//         let cardinal_direction = match cur_rail.direction {
//             RailDirection::Up => "north",
//             RailDirection::Down => "south",
//             RailDirection::Left => "east",
//             RailDirection::Right => "west",
//         };
//
//         lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
//             inner: FacSurfaceCreateEntity {
//                 name: "straight-rail".to_string(),
//                 position: Point2f {
//                     // must be odd
//                     x: cur_rail.endpoint.x as f32,
//                     y: cur_rail.endpoint.y as f32,
//                 },
//                 surface_var: DEFAULT_SURFACE_VAR.to_string(),
//                 extra: Vec::new(),
//                 params: direction_params(cardinal_direction),
//             },
//         }));
//     }
// }
