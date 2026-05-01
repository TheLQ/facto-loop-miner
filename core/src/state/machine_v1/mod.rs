use crate::state::machine::Machine;

mod step00_import;
mod step03_crop;
mod step04_contours;
mod step10_base;
mod step20_nav;
// mod step21_demark;
// mod step30_facto;
mod step99_death;

pub fn new_v1_machine() -> Machine {
    Machine {
        steps: Vec::from([
            step00_import::Step00::new_boxed(),
            step03_crop::Step03::new_boxed(),
            step04_contours::Step04::new_boxed(),
            step10_base::Step10::new_boxed(),
            step20_nav::Step20::new_boxed(),
            // step30_facto::Step30::new_boxed(),
            step99_death::Step99Death::new_boxed(),
        ]),
    }
}
