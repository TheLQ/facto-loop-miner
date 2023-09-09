use crate::state::machine::Machine;
use crate::state::machine_v1::step00_import::Step00;
use crate::state::machine_v1::step03_crop::Step03;
use crate::state::machine_v1::step04_contours::Step04;
use crate::state::machine_v1::step10_base::Step10;
use crate::state::machine_v1::step20_nav::Step20;
use crate::state::machine_v1::step99_death::Step99Death;

mod step00_import;
mod step03_crop;
mod step04_contours;
mod step10_base;
mod step20_nav;
mod step99_death;

pub fn new_v1_machine() -> Machine {
    Machine {
        steps: Vec::from([
            Step00::new(),
            Step03::new(),
            Step04::new(),
            Step10::new(),
            Step20::new(),
            Step99Death::new(),
        ]),
    }
}
