use crate::blueprint::output::FacItemOutput;
use crate::common::vpoint::VPoint;
use crate::game_blocks::block::{FacBlock2, FacBlockFancy};
use crate::game_blocks::mine_ore::FacBlkMineOre;
use crate::game_blocks::rail_hope::RailHopeLink;
use crate::game_blocks::rail_hope_single::HopeLink;
use crate::game_blocks::rail_hope_soda::{HopeSodaLink, sodas_to_links, sodas_to_rails};
use crate::game_blocks::rail_station::{FacBlkRailStation, FacExtDelivery};
use crate::game_entities::belt::FacEntBeltType;
use crate::game_entities::direction::FacDirectionQuarter;
use crate::game_entities::inserter::FacEntInserterType;
use crate::game_entities::module::FacModule;
use std::rc::Rc;

pub struct FacBlkMineIsland {
    pub rail_entrance_link: HopeSodaLink,
    pub mines: Vec<Vec<VPoint>>,
    pub wagons: u32,
    pub front_engines: u32,
    pub belt: FacEntBeltType,
    pub inserter: FacEntInserterType,
    pub drill_modules: [Option<FacModule>; 3],
    // pub rail_entrance_start: VPoint,
    // pub rail_entrance_direction: FacDirectionQuarter,
    pub output: Rc<FacItemOutput>,
}

impl FacBlockFancy<()> for FacBlkMineIsland {
    fn generate(&self) {
        for link in sodas_to_rails([self.rail_entrance_link.add_straight_section()]) {
            link.write_output(&self.output);
        }

        let start_hope: &HopeLink = &self
            .rail_entrance_link
            .add_straight_section()
            .links_source()[1];
        let origin = start_hope.rails.first().unwrap().position;
        FacBlkRailStation {
            name: "something".into(),
            delivery: FacExtDelivery::Belt {
                btype: self.belt,
                turn_clockwise: false,
            },
            wagons: self.wagons,
            front_engines: self.front_engines,
            fuel_inserter: None,
            fuel_inserter_chest: None,
            inserter: self.inserter,
            place_train: None,
            is_east: true, //todo
            is_up: true,   // todo
            is_electric_initial: false,
            is_input: true,
            output: self.output.clone(),
        }
        .generate(origin);

        // for mine in &self.mines {
        //     let output_belts = FacBlkMineOre {
        //         ore_points: mine.clone(),
        //         exit_direction: todo!(),
        //         exit_clockwise: todo!(),
        //         belt: self.belt,
        //         drill_modules: self.drill_modules,
        //         output: self.output.clone(),
        //     }
        //     .generate();
        // }
    }
}
