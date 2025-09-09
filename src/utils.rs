use std::{process::Command, path::Path, path::PathBuf, env};

pub fn get_config_path() -> PathBuf {
    Path::new(&env::var("HOME").unwrap_or_else(|_| ".".to_string())).join(CONFIG_PATH)
}

pub fn reload_hyprland() {
    let cmd = Command::new("hyprctl")
        .arg("reload")
        .output()
        .expect("failed to reload hyprland");

    println!("Reloading Hyprland status: {}", cmd.status.code().unwrap_or(-1));
}

pub const CONFIG_PATH: &str = ".config/hypr/hyprland.conf";
pub const BACKUP_SUFFIX: &str = "-bak";
