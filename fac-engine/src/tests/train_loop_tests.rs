use crate::{
    admiral::{
        err::AdmiralResult,
        lua_command::{LuaCommand, train_boot::train_boot},
    },
    blueprint::{
        bpfac::infinity::{FacBpFilter, FacBpInfinitySettings},
        bpitem::BlueprintItem,
        output::FacItemOutput,
    },
    common::{
        entity::FacEntity,
        varea::VArea,
        vpoint::{VPOINT_ZERO, VPoint},
    },
    game_blocks::{
        rail_loop::{FacBlkRailLoop, FacBlkRailLoopProps},
        rail_station::FacExtDelivery,
    },
    game_entities::{
        chest::FacEntChestType, direction::FacDirectionQuarter,
        infinity_power::FacEntInfinityPower, inserter::FacEntInserterType,
    },
};
use std::rc::Rc;

pub fn make_rail_loop(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    let origin = VPOINT_ZERO;

    output.write(BlueprintItem::new(
        FacEntInfinityPower::new().into_boxed(),
        origin.move_xy(4, 2),
    ));
    let mut rail_loop = FacBlkRailLoop::new(FacBlkRailLoopProps {
        name_prefix: "Basic".into(),
        wagons: 3,
        front_engines: 2,
        origin,
        origin_direction: FacDirectionQuarter::West,
        delivery_input: FacExtDelivery::Chest(FacEntChestType::Infinity(FacBpInfinitySettings {
            remove_unfiltered_items: false,
            filters: vec![FacBpFilter::new_for_item("iron-stick")],
        })),
        delivery_output: FacExtDelivery::Chest(FacEntChestType::Infinity(FacBpInfinitySettings {
            remove_unfiltered_items: true,
            filters: vec![
                // FacBpFilter::new_for_item("iron-stick"),
                FacBpFilter::new_for_item("iron-ore"),
            ],
        })),
        inserter_type: FacEntInserterType::Stack,
        is_start_input: true,
        output: output.clone(),
    });
    rail_loop.add_straight();
    rail_loop.add_straight();
    rail_loop.add_straight();
    rail_loop.add_turn90(false);
    rail_loop.add_straight();
    rail_loop.add_straight();
    rail_loop.add_straight();
    rail_loop.add_turn90(false);
    rail_loop.add_straight();
    rail_loop.add_straight();
    rail_loop.add_straight();
    rail_loop.add_base_start_and_end();

    output.admiral_execute_command(
        train_boot(VArea::from_arbitrary_points_pair(
            VPoint::new(-90, -90),
            VPoint::new(90, 90),
        ))
        .into_boxed(),
    )?;

    Ok(())
}
