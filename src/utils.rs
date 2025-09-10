use std::{env, path::Path, path::PathBuf, process::Command};

pub fn get_config_path(write: bool) -> PathBuf {
    if write {
        Path::new(&env::var("HOME").unwrap_or_else(|_| ".".to_string())).join(HYPRVIZ_CONFIG_PATH)
    } else {
        Path::new(&env::var("HOME").unwrap_or_else(|_| ".".to_string())).join(CONFIG_PATH)
    }
}

pub fn reload_hyprland() {
    let cmd = Command::new("hyprctl")
        .arg("reload")
        .output()
        .expect("failed to reload hyprland");

    println!(
        "Reloading Hyprland status: {}",
        cmd.status.code().unwrap_or(-1)
    );
}

pub fn check_last_non_empty_line(file_content: &str, expected_line: &str) -> bool {
    let lines: Vec<&str> = file_content.lines().collect();

    for line in lines.iter().rev() {
        if !line.trim().is_empty() {
            return line.trim() == expected_line;
        }
    }

    false
}

pub const CONFIG_PATH: &str = ".config/hypr/hyprland.conf";
pub const HYPRVIZ_CONFIG_PATH: &str = ".config/hypr/hyprviz.conf";
pub const BACKUP_SUFFIX: &str = "-bak";
