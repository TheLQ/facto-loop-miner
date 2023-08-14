use crate::surface::pixel::Pixel;
use serde::{Deserialize, Serialize};
use std::any::Any;
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
                println!("[State] Reading {}", path.display());
                serde_json::from_str(&v)
            }
            Err(e) => {
                println!(
                    "[State] No existing state ({}) {}",
                    e.kind(),
                    path.display()
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
        println!("Writing to {}", self.path.display());
        let file = File::create(&self.path).unwrap();
        serde_json::to_writer(BufWriter::new(file), self).unwrap();
    }

    pub fn image_needs_rebuild(&self, path: &Path) -> bool {
        let modified_new = fs::metadata(path).unwrap().modified().unwrap();
        match self.files_to_last_modified.get(path) {
            Some(modified_old) => {
                modified_new.duration_since(UNIX_EPOCH).unwrap().as_millis() == modified_old.clone()
            }
            None => true,
        }
    }
}
