use crate::state::machine::Machine;
use crate::state::machine_v1::step00_import::Step00;

mod step00_import;
mod step04_contours;
mod step10_base;

pub fn new_v1_machine() -> Machine {
    Machine {
        steps: Vec::from([
            Step00::new(),
            // Step10::new()
        ]),
    }
}
