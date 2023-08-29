use crate::state::disk::State;
use crate::surface::metric::Metrics;
use std::cell::RefCell;
use std::fs::{create_dir, read_dir};
use std::path::{Path, PathBuf};
use std::rc::Rc;

type StepBox = Box<dyn Step>;

pub struct Machine {
    pub(crate) steps: Vec<StepBox>,
}

pub struct StepParams {
    pub step_out_dir: PathBuf,
    pub metrics: Rc<RefCell<Metrics>>,
    pub step_history_out_dirs: Vec<PathBuf>,
    pub state: Rc<RefCell<State>>,
}

pub trait Step {
    fn name(&self) -> String;
    fn transformer(&self, params: StepParams);
}

impl Machine {
    pub fn start(self, work_dir: &Path) {
        let output_dir = work_dir.join("out0");
        if !output_dir.is_dir() {
            create_dir(&output_dir).unwrap();
        }

        let state = Rc::new(RefCell::new(State::new(&work_dir.join("state.json"))));

        let step_names: Vec<String> = self.steps.iter().map(|v| v.name()).collect();
        println!("[Machine] Steps {}", step_names.join(","));

        let mut step_history_out_dirs = Vec::new();
        for step in &self.steps {
            println!("=== {}", step.name());

            let step_out_dir = output_dir.join(step.name());
            if !step_out_dir.is_dir() {
                create_dir(&step_out_dir).unwrap();
            }
            let changed = state.borrow_mut().update_modified(&step_out_dir);
            if changed {
                let metrics = Rc::new(RefCell::new(Metrics::default()));
                println!("=== Found changes, transforming");

                step.transformer(StepParams {
                    step_out_dir: step_out_dir.clone(),
                    metrics: metrics.clone(),
                    step_history_out_dirs: step_history_out_dirs.clone(),
                    state: state.clone(),
                });

                metrics.borrow().log_final();
                // dir modify date updated after writing to folder
                state.borrow_mut().update_modified(&step_out_dir);

                step_history_out_dirs.push(step_out_dir);
            } else {
                println!("No Changes Found")
            }
        }

        state.borrow().disk_write();
    }
}

fn panic_if_dir_not_empty(dir: &Path) {
    let raw_entries = read_dir(dir).unwrap();
    let children_count: Vec<String> = raw_entries
        .into_iter()
        .map(|v| v.unwrap().file_name().into_string().unwrap())
        .collect();
    if !children_count.is_empty() {
        panic!(
            "output path {} not empty with files {}",
            dir.display(),
            children_count.join(" ")
        )
    }
}
