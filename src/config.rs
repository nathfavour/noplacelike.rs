use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub upload_folder: String,
    pub download_folder: String,
    pub audio_folders: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            upload_folder: "~/noplacelike/uploads".to_string(),
            download_folder: "~/Downloads".to_string(),
            audio_folders: Vec::new(),
        }
    }
}

// Store config globally for ease of access
lazy_static::lazy_static! {
    pub(crate) static ref CONFIG: Arc<Mutex<Config>> = Arc::new(Mutex::new(Config::default()));
}

pub fn get_config_path() -> PathBuf {
    let mut path = home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".noplacelikeconfig.json");
    path
}

pub fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        let mut home = home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(&path[2..]);
        home
    } else {
        PathBuf::from(path)
    }
}

pub fn load_config() -> Config {
    let path = get_config_path();
    
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<Config>(&content) {
                    Ok(config) => {
                        let mut guard = CONFIG.lock().unwrap();
                        *guard = config.clone();
                        return config;
                    }
                    Err(e) => {
                        eprintln!("Error parsing config file: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading config file: {}", e);
            }
        }
    }

    // Create default config if not exists
    let default_config = Config::default();
    save_config(&default_config);
    default_config
}

pub fn save_config(config: &Config) {
    let path = get_config_path();
    
    // Update global config
    {
        let mut guard = CONFIG.lock().unwrap();
        *guard = config.clone();
    }
    
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| {
            eprintln!("Failed to create config directory: {}", e);
        });
    }
    
    // Write config to file
    let content = serde_json::to_string_pretty(config).unwrap_or_else(|e| {
        eprintln!("Error serializing config: {}", e);
        String::new()
    });
    
    fs::write(&path, content).unwrap_or_else(|e| {
        eprintln!("Error writing config file: {}", e);
    });
}

pub fn update_config(config: Config) -> Config {
    save_config(&config);
    config
}

pub fn ensure_upload_folder() -> PathBuf {
    let config = CONFIG.lock().unwrap();
    let path = expand_path(&config.upload_folder);
    fs::create_dir_all(&path).unwrap_or_else(|e| {
        eprintln!("Failed to create upload directory: {}", e);
    });
    path
}

pub fn get_audio_folders() -> Vec<PathBuf> {
    let config = CONFIG.lock().unwrap();
    let mut folders = Vec::new();
    
    for folder in &config.audio_folders {
        let path = expand_path(folder);
        // Ensure directory exists
        if let Err(e) = fs::create_dir_all(&path) {
            eprintln!("Failed to create audio directory: {}", e);
        }
        folders.push(path);
    }
    
    // Add default folder if none specified
    if folders.is_empty() {
        let default_path = expand_path("~/noplacelike/audio");
        fs::create_dir_all(&default_path).unwrap_or_else(|e| {
            eprintln!("Failed to create default audio directory: {}", e);
        });
        folders.push(default_path);
    }
    
    folders
}

pub fn add_audio_folder(folder: String) -> Result<(), String> {
    let mut config = CONFIG.lock().unwrap().clone();
    
    if !config.audio_folders.contains(&folder) {
        config.audio_folders.push(folder);
        save_config(&config);
    }
    Ok(())
}

pub fn remove_audio_folder(folder: &str) -> Result<(), String> {
    let mut config = CONFIG.lock().unwrap().clone();
    
    config.audio_folders.retain(|f| f != folder);
    save_config(&config);
    Ok(())
}
