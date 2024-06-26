use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::{read_to_string, File};
use std::io::{BufWriter, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Serialize, Deserialize)]
pub struct State {
    #[serde(skip)]
    path: PathBuf,
    files_to_last_modified: HashMap<PathBuf, u128>,
}

impl Default for State {
    fn default() -> Self {
        State {
            path: PathBuf::new(),
            files_to_last_modified: HashMap::new(),
        }
    }
}

impl State {
    pub fn new(path: &Path) -> Self {
        let mut state = match read_to_string(path) {
            Ok(v) => {
                tracing::debug!("[State] Reading {}", path.display());
                serde_json::from_str(&v)
            }
            Err(e) => {
                tracing::debug!(
                    "[State] No existing state ({}) {}",
                    e.kind(),
                    path.display(),
                );
                match e.kind() {
                    ErrorKind::NotFound => Ok(State::default()),
                    e => {
                        panic!("unknown error with state {}", e)
                    }
                }
            }
        }
        .unwrap();
        state.path = path.to_path_buf();
        state
    }

    pub fn disk_write(&self) {
        tracing::debug!("Writing to {}", self.path.display());
        let file = File::create(&self.path).unwrap();
        serde_json::to_writer(BufWriter::new(file), self).unwrap();
    }

    pub fn update_modified(&mut self, path: &Path) -> bool {
        !path.exists()
    }

    pub fn update_modified_old(&mut self, path: &Path) -> bool {
        let metadata = match fs::metadata(path) {
            Ok(v) => v,
            Err(e) => panic!("Path {} e {}", path.display(), e),
        };
        let mut force_rebuild = false;
        let modified_new = metadata.modified().unwrap();
        let modified_new_milli = modified_new.duration_since(UNIX_EPOCH).unwrap().as_millis();
        self.files_to_last_modified
            .entry(PathBuf::from(path))
            .and_modify(|modified_old| {
                if &modified_new_milli != modified_old {
                    force_rebuild = true;
                    tracing::debug!(
                        "[State] found change on {} old {} new {}",
                        path.display(),
                        modified_old,
                        modified_new_milli,
                    );
                    *modified_old = modified_new_milli;
                }
            })
            .or_insert_with(|| {
                force_rebuild = true;
                modified_new_milli
            });

        force_rebuild
    }
}
