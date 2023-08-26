use crate::gamedata::lua::LuaData;
use crate::state::disk::State;
use crate::surface::metric::Metrics;
use crate::surface::surface::Surface;
use std::cell::Cell;
use std::fs::{create_dir, read_dir};
use std::path::{Path, PathBuf};

type StepBox = Box<dyn Step>;

pub struct Machine {
    pub(crate) steps: Vec<StepBox>,
}

pub trait Step {
    fn name(&self) -> String;
    fn dependency_files(&self) -> Option<Vec<PathBuf>>;
    fn transformer(&self, surface: &mut Surface, data: &mut LuaData, metrics: &mut Metrics);
    // fn force_transformer_run(&self);
}

impl Machine {
    pub fn start(self, root_dir: &Path) {
        let output_dir = root_dir.join("out0");
        if !output_dir.is_dir() {
            create_dir(&output_dir).unwrap();
        }
        panic_if_dir_not_empty(&output_dir);

        let mut state = State::new(&root_dir.join("state.json"));

        let step_names: Vec<String> = self.steps.iter().map(|v| v.name()).collect();
        println!("[Machine] Steps {}", step_names.join(","));

        let previous_step_opt: Cell<Option<&StepBox>> = Cell::new(None);
        // dummy values immediately replaced
        let mut lua_surface = Surface::new(1, 1);
        let lua_files = self.steps[0].dependency_files().unwrap();
        let mut lua_data = LuaData::open(&lua_files[0], &lua_files[1]);
        for step in &self.steps {
            println!("=== {}", step.name());

            let mut changed = true;
            if let Some(dependency_files) = step.dependency_files() {
                for dependency_file_path in dependency_files {
                    let dependency_changed = state.update_modified(&dependency_file_path);
                    changed = if changed { dependency_changed } else { false };
                }
            }
            if changed {
                let mut metrics = Metrics::default();

                if let Some(_) = previous_step_opt.get() {
                    println!("=== Found changes, transforming");

                    step.transformer(&mut lua_surface, &mut lua_data, &mut metrics)
                } else {
                    println!("=== Found changes on first step, transforming");

                    step.transformer(&mut lua_surface, &mut lua_data, &mut metrics);
                }
                lua_surface.save(&output_dir.join(step.name()));

                metrics.log_final();
            } else {
                println!("No Changes Found")
            }
            previous_step_opt.set(Some(step));
        }
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
