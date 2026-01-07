use std::path::PathBuf;
use dirs::home_dir;

pub fn config_dir() -> PathBuf {
    let mut path = home_dir().expect("Could not find home directory");
    path.push(".config");
    path.push("tide");
    
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    
    path
}

pub fn state_file_path() -> PathBuf {
    let mut path = config_dir();
    path.push("state.json");
    path
}

pub fn launchers_file_path() -> PathBuf {
    let mut path = config_dir();
    path.push("launchers.json");
    path
}
