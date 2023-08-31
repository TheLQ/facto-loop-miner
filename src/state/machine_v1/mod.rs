use crate::state::machine::Machine;
use crate::state::machine_v1::step00_import::Step00;
use crate::state::machine_v1::step04_contours::Step04;

mod step00_import;
mod step04_contours;
mod step10_base;

pub fn new_v1_machine() -> Machine {
    Machine {
        steps: Vec::from([
            Step00::new(),
            Step04::new(), // Step10::new()
        ]),
    }
}
