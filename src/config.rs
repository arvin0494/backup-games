use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub const DEFAULT_SOURCE: &str = "~/Games";
pub const DEFAULT_DEST: &str = "/mnt/HDD4T/GAMES";
pub const LOG_FILE: &str = "/tmp/backup-games.log";
pub const CONFIG_DIR: &str = "backup-games";
pub const MANIFEST_FILE: &str = "~/.local/share/backup-games/manifest";

pub fn load_user_config() -> HashMap<String, String> {
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join(CONFIG_DIR)
        .join("config");

    let mut map = HashMap::new();
    if !config_path.exists() {
        return map;
    }
    let content = match fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return map,
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    map
}

pub fn load_dir_sources(user_cfg: &HashMap<String, String>) -> Vec<String> {
    if let Some(s) = user_cfg.get("dirsources") {
        let list: Vec<String> = s.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !list.is_empty() {
            return list;
        }
    }
    Vec::new()
}

pub fn load_sources(user_cfg: &HashMap<String, String>, cli_source: Option<String>) -> Vec<String> {
    if let Some(s) = cli_source {
        return vec![s];
    }
    if let Some(s) = user_cfg.get("sources") {
        let list: Vec<String> = s.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !list.is_empty() {
            return list;
        }
    }
    if let Some(s) = user_cfg.get("source") {
        return vec![s.clone()];
    }
    vec![DEFAULT_SOURCE.to_string()]
}
