use std::rc::Rc;

use crate::{
    admiral::err::AdmiralResult,
    blueprint::{
        bpfac::{
            schedule::{
                FacBpCircuitCondition, FacBpLogic, FacBpSchedule, FacBpScheduleData,
                FacBpScheduleWait, FacBpWaitType,
            },
            signal_id::{FacBpSignalId, FacBpSignalIdType},
        },
        output::FacItemOutput,
    },
    common::vpoint::VPOINT_ZERO,
    game_blocks::{
        block::FacBlock2,
        rail_station::{FacBlkRailStation, FacExtDelivery},
    },
    game_entities::{belt::FacEntBeltType, chest::FacEntChestType, inserter::FacEntInserterType},
};

pub fn make_rail_station(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    let station = FacBlkRailStation {
        name: "test".into(),
        wagons: 4,
        front_engines: 2,
        delivery: FacExtDelivery::Belt(FacEntBeltType::Basic),
        // chests: Some(FacEntChestType::Steel),
        // chests: None,
        inserter: FacEntInserterType::Basic,
        fuel_inserter: Some(FacEntInserterType::Basic),
        fuel_inserter_chest: Some(FacEntChestType::Steel),
        is_east: true,
        // is_east: false,
        is_up: true,
        // is_up: false,
        is_input: true,
        is_create_train: true,
        schedule: Some(FacBpSchedule {
            locomotives: Vec::new(),
            schdata: [
                FacBpScheduleData {
                    station: "SomeTestStart".into(),
                    wait_conditions: [
                        FacBpScheduleWait {
                            compare_type: FacBpLogic::Or,
                            ctype: FacBpWaitType::ItemCount,
                            condition: Some(FacBpCircuitCondition {
                                comparator: Some("<".into()),
                                first_signal: Some(FacBpSignalId {
                                    stype: FacBpSignalIdType::Item,
                                    name: "heavy-oil-barrel".into(),
                                }),
                                second_signal: None,
                                constant: Some(800),
                            }),
                        },
                        FacBpScheduleWait {
                            compare_type: FacBpLogic::Or,
                            ctype: FacBpWaitType::Empty,
                            condition: None,
                        },
                    ]
                    .into(),
                },
                FacBpScheduleData {
                    station: "SomeTestEnd".into(),
                    wait_conditions: [FacBpScheduleWait {
                        compare_type: FacBpLogic::Or,
                        ctype: FacBpWaitType::Full,
                        condition: None,
                    }]
                    .into(),
                },
            ]
            .into(),
        }),
        output,
    };
    let belts = station.generate(VPOINT_ZERO);
    for mut belt in belts {
        belt.add_straight_underground(5);
    }
    Ok(())
}
