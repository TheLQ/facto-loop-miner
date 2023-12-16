use crate::state::disk::State;
use crate::state::err::XMachineResult;
use crate::surface::metric::Metrics;
use crate::util::duration::BasicWatch;
use std::cell::RefCell;
use std::fs::{create_dir, read_dir, remove_dir, remove_dir_all};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use tracing::{debug, error, info, warn};

type StepBox = Box<dyn Step>;
pub const DEATH_STEP_NAME: &str = "step99-death";

pub struct Machine {
    pub(crate) steps: Vec<StepBox>,
}

pub struct StepParams {
    pub step_out_dir: PathBuf,
    pub metrics: Rc<RefCell<Metrics>>,
    step_history_out_dirs: Vec<PathBuf>,
    pub state: Rc<RefCell<State>>,
}

impl StepParams {
    pub fn previous_step_dir(&self) -> &Path {
        self.step_history_out_dirs.last().unwrap()
    }
}

pub trait Step {
    fn name(&self) -> &'static str;
    fn transformer(&self, params: StepParams) -> XMachineResult<()>;
}

impl Machine {
    pub fn start(self, work_dir: &Path) {
        let output_dir = work_dir.join("out0");
        if !output_dir.is_dir() {
            create_dir(&output_dir).unwrap();
        }

        let state = Rc::new(RefCell::new(State::new(&work_dir.join("state.json"))));

        let step_names: Vec<String> = self.steps.iter().map(|v| v.name()).collect();
        tracing::debug!("[Machine] Steps {}", step_names.join(","));

        let mut step_history_out_dirs = Vec::new();
        let mut enabled = false;
        for step in &self.steps {
            let header_prefix = format!("=== {}", step.name());
            if step.name() == DEATH_STEP_NAME {
                warn!("{} - RIP", header_prefix);
                break;
            }

            let step_out_dir = output_dir.join(step.name());
            if !enabled {
                // let changed = state.borrow_mut().update_modified(&step_out_dir);
                enabled = !step_out_dir.exists();
            }

            if enabled {
                if step_out_dir.exists() {
                    remove_dir_all(&step_out_dir).unwrap()
                }
                create_dir(&step_out_dir).unwrap();
                let metrics = Rc::new(RefCell::new(Metrics::new(step.name())));
                info!("{} - Transforming", header_prefix);

                let mut step_watch = BasicWatch::start();

                let transformer_result = step.transformer(StepParams {
                    step_out_dir: step_out_dir.clone(),
                    metrics: metrics.clone(),
                    step_history_out_dirs: step_history_out_dirs.clone(),
                    state: state.clone(),
                });
                if let Err(e) = transformer_result {
                    error!("Machine failed! {}\n{}", e, e.my_backtrace());
                    return;
                }
                step_watch.stop();

                RefCell::into_inner(Rc::into_inner(metrics).unwrap()).log_final();

                if read_dir(&step_out_dir).unwrap().count() == 0 {
                    tracing::debug!("=== detected empty dir, removing for future re-processing");
                    remove_dir(&step_out_dir).unwrap();
                } else {
                    // dir modify date updated after writing to folder
                    state.borrow_mut().update_modified(&step_out_dir);
                }

                debug!("Step Completed in {}", step_watch,);
            } else {
                info!("{} - No Changes Found", header_prefix)
            }
            step_history_out_dirs.push(step_out_dir.clone());
        }

        RefCell::into_inner(Rc::into_inner(state).unwrap()).disk_write();
    }
}

pub fn search_step_history_dirs<I>(step_history_out_dirs: I, name: &str) -> PathBuf
where
    I: Iterator<Item = PathBuf> + DoubleEndedIterator,
{
    for dir in step_history_out_dirs.rev() {
        let file = dir.join(name);
        tracing::debug!("search {}", file.display());
        if file.exists() {
            return file;
        }
    }
    panic!("cannot find {}", name)
}

#[allow(dead_code)]
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
