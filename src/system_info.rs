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
        return "Failed to get version".to_string();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().unwrap_or("");

    if let Some(version_line) = first_line.strip_prefix("Hyprland ") {
        let version = version_line.split_whitespace().next().unwrap_or("");
        return version.strip_prefix('v').unwrap_or(version).to_string();
    }

    "Failed to parse version".to_string()
}

pub fn get_hyprviz_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
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
        _ => "OS information not available".to_string(),
    }
}

pub fn get_kernel_info() -> String {
    match execute_command("uname", &["-srm"]) {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        Ok(_) => "Failed to get kernel info".to_string(),
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
            format!("CPU with {} cores (model unknown)", cores)
        }
        _ => "CPU information not available".to_string(),
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
                "Failed to parse GPU info".to_string()
            } else {
                gpu_lines.join("\n")
            }
        }
        _ => "Failed to get GPU info".to_string(),
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
                    _ => "Failed to parse memory info".to_string(),
                }
            } else {
                "Failed to get memory info from /proc/meminfo".to_string()
            }
        }
        _ => "Failed to get memory info".to_string(),
    }
}

pub fn get_monitor_info() -> String {
    match execute_command("hyprctl", &["monitors"]) {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut result = Vec::new();

            for line in output_str.lines() {
                if line.contains("Monitor") {
                    result.push(line.trim().to_string());
                } else if line.contains("description:")
                    || line.contains(" at ")
                    || line.contains("transform:")
                    || line.contains("scale:")
                    || line.contains("currentFormat:")
                {
                    result.push(format!("   {}", line.trim()));
                }
            }

            if result.is_empty() {
                output_str
                    .lines()
                    .next()
                    .map(|s| s.to_string())
                    .unwrap_or_default()
            } else {
                result.join("\n")
            }
        }
        Ok(_) => "Failed to get monitor info".to_string(),
        Err(e) => e,
    }
}
