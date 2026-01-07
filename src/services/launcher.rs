use serde::{Deserialize, Serialize};
use std::fs;
use crate::services::config::launchers_file_path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LauncherRunIn {
    Current,
    NewSplit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Launcher {
    pub name: String,
    pub command: String,
    pub run_in: LauncherRunIn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    pub version: u32,
    pub launchers: Vec<Launcher>,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            version: 1,
            launchers: vec![
                Launcher {
                    name: "Claude".to_string(),
                    command: "claude".to_string(),
                    run_in: LauncherRunIn::Current,
                },
                Launcher {
                    name: "Gemini".to_string(),
                    command: "gemini".to_string(),
                    run_in: LauncherRunIn::Current,
                },
            ],
        }
    }
}

pub fn load_launchers() -> Vec<Launcher> {
    let path = launchers_file_path();
    if !path.exists() {
        // Return default launchers if file doesn't exist, 
        // effectively providing a template for the user
        return LauncherConfig::default().launchers;
    }

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<LauncherConfig>(&content) {
            Ok(config) => config.launchers,
            Err(err) => {
                eprintln!("Failed to parse launchers file: {}", err);
                LauncherConfig::default().launchers
            }
        },
        Err(err) => {
            eprintln!("Failed to read launchers file: {}", err);
            LauncherConfig::default().launchers
        }
    }
}
