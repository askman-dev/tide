use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::services::config::state_file_path;
use dirs::home_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub version: u32,
    pub workspaces: Vec<PathBuf>,
    pub active_workspace_index: usize,
}

impl Default for AppState {
    fn default() -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| home_dir().expect("Could not find home directory"));
        Self {
            version: 1,
            workspaces: vec![root],
            active_workspace_index: 0,
        }
    }
}

pub fn load_state() -> AppState {
    let path = state_file_path();
    if !path.exists() {
        return AppState::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(state) => state,
            Err(err) => {
                eprintln!("Failed to parse state file: {}", err);
                AppState::default()
            }
        },
        Err(err) => {
            eprintln!("Failed to read state file: {}", err);
            AppState::default()
        }
    }
}

pub fn save_state(workspaces: &[PathBuf], active_index: usize) {
    let state = AppState {
        version: 1,
        workspaces: workspaces.to_vec(),
        active_workspace_index: active_index,
    };

    let path = state_file_path();
    match serde_json::to_string_pretty(&state) {
        Ok(content) => {
            if let Err(err) = fs::write(path, content) {
                eprintln!("Failed to write state file: {}", err);
            }
        }
        Err(err) => {
            eprintln!("Failed to serialize state: {}", err);
        }
    }
}
