use crate::state::machine::Machine;
use crate::state::machine_v1::step00_import::Step00;
use crate::state::machine_v1::step03_crop::Step03;
use crate::state::machine_v1::step04_contours::Step04;
use crate::state::machine_v1::step10_base::Step10;
use crate::state::machine_v1::step20_nav::Step20;
// use crate::state::machine_v1::step21_demark::Step21;
use crate::state::machine_v1::step30_facto::Step30;
use crate::state::machine_v1::step99_death::Step99Death;

mod step00_import;
mod step03_crop;
mod step04_contours;
mod step10_base;
mod step20_nav;
// mod step21_demark;
mod step30_facto;
mod step99_death;

pub fn new_v1_machine() -> Machine {
    Machine {
        steps: Vec::from([
            Step00::new_boxed(),
            Step03::new_boxed(),
            Step04::new_boxed(),
            Step10::new_boxed(),
            Step20::new_boxed(),
            Step30::new_boxed(),
            Step99Death::new_boxed(),
        ]),
    }
}
