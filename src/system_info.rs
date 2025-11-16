use rust_i18n::t;
use serde_json::Value;
use std::{
    io::Read,
    {env, fs},
};

use crate::utils::{execute_command, execute_shell_command};

pub fn get_hyprland_version() -> String {
    let output = match execute_command("hyprctl", &["version"]) {
        Ok(out) => out,
        Err(e) => return e,
    };

    if !output.status.success() {
        return t!("failed_to_get_version").to_string();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().unwrap_or("");

    if let Some(version_line) = first_line.strip_prefix("Hyprland ") {
        let version = version_line.split_whitespace().next().unwrap_or("");
        return version.strip_prefix('v').unwrap_or(version).to_string();
    }

    t!("failed_to_parse_version").to_string()
}

pub fn get_hyprviz_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn get_distro_id() -> String {
    if let Ok(mut file) = fs::File::open("/etc/os-release") {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            for line in content.lines() {
                if let Some(id) = line.strip_prefix("ID=") {
                    return id.trim_matches('"').to_string();
                }
            }
        }
    }

    if let Ok(output) = execute_command("lsb_release", &["-i", "-s"])
        && output.status.success()
    {
        return String::from_utf8_lossy(&output.stdout)
            .trim()
            .trim_matches('"')
            .to_lowercase()
            .to_string();
    }

    "unknown".to_string()
}

pub fn get_distro_logo_path() -> Option<String> {
    let distro_id = get_distro_id();
    let distro_id = if distro_id == "arch" {
        "archlinux".to_string()
    } else {
        distro_id
    };

    let logo_names = vec![
        format!("{}-logo", distro_id),
        format!("{}-logo-icon", distro_id),
        format!("{}-logo-symbolic", distro_id),
        distro_id.clone(),
    ];

    let search_paths = vec![
        "/usr/share/icons/",
        "/usr/share/pixmaps/",
        "/usr/share/icons/hicolor/256x256/apps/",
        "/usr/share/icons/hicolor/128x128/apps/",
    ];

    for path in search_paths {
        for name in &logo_names {
            for ext in &["png", "svg", "jpg"] {
                let full_path = format!("{}/{}.{}", path, name, ext);
                if fs::metadata(&full_path).is_ok() {
                    return Some(full_path);
                }
            }
        }
    }

    None
}

pub fn get_os_info() -> String {
    // Try /etc/os-release
    if let Ok(mut file) = fs::File::open("/etc/os-release") {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            for line in content.lines() {
                if let Some(stripped) = line.strip_prefix("PRETTY_NAME=") {
                    let os_name = stripped.trim_matches('"').to_string();
                    if !os_name.is_empty() {
                        return os_name;
                    }
                }
            }
        }
    }

    // Try lsb_release
    if let Ok(output) = execute_command("lsb_release", &["-d"])
        && output.status.success()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        if let Some(desc) = output_str.split_once(':').map(|x| x.1) {
            return desc.trim().to_string();
        }
    }

    // Try hostnamectl
    if let Ok(output) = execute_command("hostnamectl", &[])
        && output.status.success()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("Operating System")
                && let Some(os_info) = line.split_once(':').map(|x| x.1)
            {
                return os_info.trim().to_string();
            }
        }
    }

    // Fallback to uname
    match execute_command("uname", &["-srm"]) {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => t!("os_information_not_available").to_string(),
    }
}

pub fn get_kernel_info() -> String {
    match execute_command("uname", &["-srm"]) {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        Ok(_) => t!("failed_to_get_kernel_info").to_string(),
        Err(e) => e,
    }
}

pub fn get_user_info() -> String {
    let username = match env::var("USER") {
        Ok(user) => user,
        Err(_) => match execute_command("whoami", &[]) {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            }
            _ => "unknown".to_string(),
        },
    };

    let hostname = match env::var("HOSTNAME") {
        Ok(host) => host,
        Err(_) => match execute_command("hostname", &[]) {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            }
            _ => "localhost".to_string(),
        },
    };

    format!("{}@{}", username, hostname)
}

pub fn get_host_info() -> String {
    let product_name = fs::read_to_string("/sys/class/dmi/id/product_name")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let product_version = fs::read_to_string("/sys/class/dmi/id/product_version")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if !product_name.is_empty() {
        if !product_version.is_empty() {
            format!("{} ({})", product_name, product_version)
        } else {
            product_name
        }
    } else {
        get_user_info()
    }
}

pub fn get_cpu_info() -> String {
    // Method 1: try to get from /proc/cpuinfo
    if let Ok(output) = execute_shell_command(
        "grep 'model name' /proc/cpuinfo | head -n 1 | cut -d ':' -f 2 | xargs",
    ) && output.status.success()
    {
        let cpu_model = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !cpu_model.is_empty() {
            return cpu_model;
        }
    }

    // Method 2: try lscpu
    if let Ok(output) = execute_shell_command("lscpu | grep 'Model name' | cut -d ':' -f 2 | xargs")
        && output.status.success()
    {
        let cpu_model = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !cpu_model.is_empty() {
            return cpu_model;
        }
    }

    // Fallback: get core count
    match execute_command("nproc", &[]) {
        Ok(output) if output.status.success() => {
            let cores = String::from_utf8_lossy(&output.stdout).trim().to_string();
            t!("cpu_with__cores__model_unknown", n = cores).to_string()
        }
        _ => t!("cpu_information_not_available").to_string(),
    }
}

pub fn get_gpu_info() -> String {
    match execute_shell_command("lspci | grep -i vga") {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let gpu_lines: Vec<String> = output_str
                .lines()
                .filter_map(|line| {
                    line.split_once(": ")
                        .map(|(_, info)| info.trim().to_string())
                })
                .collect();

            if gpu_lines.is_empty() {
                t!("failed_to_parse_gpu_info").to_string()
            } else {
                gpu_lines.join("\n")
            }
        }
        _ => t!("failed_to_get_gpu_info").to_string(),
    }
}

pub fn get_memory_info() -> String {
    match execute_shell_command("grep -E 'MemTotal|MemAvailable' /proc/meminfo | awk '{print $2}'")
    {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();

            if lines.len() >= 2 {
                match (
                    lines[0].trim().parse::<f64>(),
                    lines[1].trim().parse::<f64>(),
                ) {
                    (Ok(total_kb), Ok(available_kb)) => {
                        let total_gb = total_kb / 1024.0 / 1024.0;
                        let available_gb = available_kb / 1024.0 / 1024.0;
                        let used_gb = total_gb - available_gb;
                        format!("{:.2} GB / {:.2} GB", used_gb, total_gb)
                    }
                    _ => t!("failed_to_parse_memory_info").to_string(),
                }
            } else {
                t!("failed_to_get_memory_info_from_/proc/meminfo").to_string()
            }
        }
        _ => t!("failed_to_get_memory_info").to_string(),
    }
}

pub fn get_monitor_info() -> String {
    match execute_command("hyprctl", &["monitors", "-j"]) {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);

            match serde_json::from_str::<Value>(&output_str) {
                Ok(Value::Array(monitors)) if !monitors.is_empty() => {
                    let mut result = String::new();

                    for (i, monitor) in monitors.iter().enumerate() {
                        if let Some(obj) = monitor.as_object() {
                            let id = obj.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                            let name = obj
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown");
                            let description = obj
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("No description");
                            let width =
                                obj.get("width").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                            let height =
                                obj.get("height").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                            let refresh_rate = obj
                                .get("refreshRate")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(60.0);
                            let x = obj.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                            let y = obj.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                            let scale = obj.get("scale").and_then(|v| v.as_f64()).unwrap_or(1.0);
                            let transform =
                                obj.get("transform").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                            let current_format = obj
                                .get("currentFormat")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown");
                            let focused = obj
                                .get("focused")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            let dpms_status = obj
                                .get("dpmsStatus")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);

                            let workspace_name = obj
                                .get("activeWorkspace")
                                .and_then(|w| w.as_object())
                                .and_then(|w| w.get("name"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("N/A");

                            let workspace_id = obj
                                .get("activeWorkspace")
                                .and_then(|w| w.as_object())
                                .and_then(|w| w.get("id"))
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0);

                            let focus_indicator = if focused { "*" } else { " " };
                            let dpms_text = if dpms_status {
                                t!("dpms_on")
                            } else {
                                t!("dpms_off")
                            };

                            result.push_str(&format!(
                                "{} {}: #{}: {} [{}]\n",
                                focus_indicator,
                                t!("monitor"),
                                id,
                                name,
                                dpms_text
                            ));
                            result.push_str(&format!(
                                "   {}: {}\n",
                                t!("description"),
                                description
                            ));
                            result.push_str(&format!(
                                "   {}: {}x{} @ {:.1}Hz\n",
                                t!("resolution"),
                                width,
                                height,
                                refresh_rate
                            ));
                            result.push_str(&format!("   {}: {}x{}\n", t!("position"), x, y));
                            result.push_str(&format!("   {}: {:.2}x\n", t!("scale"), scale));

                            let transform_str = match transform {
                                0 => t!("normal"),
                                1 => t!("rotate_90"),
                                2 => t!("rotate_180"),
                                3 => t!("rotate_270"),
                                4 => t!("flip"),
                                5 => t!("flip_rotate_90"),
                                6 => t!("flip_rotate_180"),
                                7 => t!("flip_rotate_270"),
                                _ => t!("unknown"),
                            };

                            result.push_str(&format!(
                                "   {}: {}\n",
                                t!("transform"),
                                transform_str
                            ));
                            result.push_str(&format!(
                                "   {}: {}\n",
                                t!("current_format"),
                                current_format
                            ));
                            result.push_str(&format!(
                                "   {}: #{} ({})\n",
                                t!("active_workspace"),
                                workspace_id,
                                workspace_name
                            ));

                            if i < monitors.len() - 1 {
                                result.push('\n');
                            }
                        }
                    }

                    result
                }
                Ok(_) => t!("no_monitors_found").into_owned(),
                Err(e) => format!("{}: {}", t!("json_parse_error"), e),
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            format!("{}: {}", t!("failed_to_get_monitor_info"), stderr.trim())
        }
        Err(e) => e,
    }
}
