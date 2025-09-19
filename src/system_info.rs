use std::{
    io::Read,
    process::Command,
    {env, fs},
};

use crate::utils::extract_brackets;

pub fn get_hyprland_version() -> String {
    match Command::new("hyprctl").arg("version").output() {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let first_line = stdout.lines().next().unwrap_or("");
                if let Some(version_line) = first_line.strip_prefix("Hyprland ") {
                    let version = version_line.split_whitespace().next().unwrap_or("");
                    return version.strip_prefix('v').unwrap_or(version).to_string();
                }
                "Failed to parse version".to_string()
            } else {
                "Failed to get version".to_string()
            }
        }
        Err(_) => "hyprctl not found".to_string(),
    }
}

pub fn get_hyprviz_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn get_os_info() -> String {
    if let Ok(mut file) = fs::File::open("/etc/os-release") {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            let mut os_name = String::new();

            for line in content.lines() {
                if let Some(stripped) = line.strip_prefix("PRETTY_NAME=") {
                    os_name = stripped.trim_matches('"').to_string();
                }
            }

            if !os_name.is_empty() {
                return os_name;
            }
        }
    }

    if let Ok(output) = Command::new("lsb_release").arg("-d").output()
        && output.status.success()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        if let Some(desc) = output_str.split_once(':').map(|x| x.1) {
            return desc.trim().to_string();
        }
    }

    if let Ok(output) = Command::new("hostnamectl").output()
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

    if let Ok(output) = Command::new("uname").arg("-srm").output()
        && output.status.success()
    {
        return String::from_utf8_lossy(&output.stdout).trim().to_string();
    }

    "OS information not available".to_string()
}

pub fn get_kernel_info() -> String {
    match Command::new("uname").arg("-srm").output() {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                "Failed to get kernel info".to_string()
            }
        }
        Err(_) => "Command failed".to_string(),
    }
}

pub fn get_user_info() -> String {
    let username = match env::var("USER") {
        Ok(user) => user,
        Err(_) => match Command::new("whoami").output() {
            Ok(output) => {
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).trim().to_string()
                } else {
                    "unknown".to_string()
                }
            }
            Err(_) => "unknown".to_string(),
        },
    };

    let hostname = match env::var("HOSTNAME") {
        Ok(host) => host,
        Err(_) => match Command::new("hostname").output() {
            Ok(output) => {
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).trim().to_string()
                } else {
                    "localhost".to_string()
                }
            }
            Err(_) => "localhost".to_string(),
        },
    };

    format!("{}@{}", username, hostname)
}

pub fn get_host_info() -> String {
    let product_name = fs::read_to_string("/sys/class/dmi/id/product_name")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| String::new());

    let product_version = fs::read_to_string("/sys/class/dmi/id/product_version")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| String::new());

    if !product_name.is_empty() {
        if !product_version.is_empty() {
            return format!("{} ({})", product_name, product_version);
        } else {
            return product_name;
        }
    }

    get_user_info()
}

pub fn get_cpu_info() -> String {
    match Command::new("sh")
        .arg("-c")
        .arg("grep 'model name' /proc/cpuinfo | head -n 1 | cut -d ':' -f 2 | xargs")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let cpu_model = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !cpu_model.is_empty() {
                    cpu_model
                } else {
                    match Command::new("nproc").output() {
                        Ok(nproc_output) => {
                            if nproc_output.status.success() {
                                let cores = String::from_utf8_lossy(&nproc_output.stdout)
                                    .trim()
                                    .to_string();
                                format!("CPU with {} cores (model unknown)", cores)
                            } else {
                                "CPU information not available".to_string()
                            }
                        }
                        Err(_) => "CPU information not available".to_string(),
                    }
                }
            } else {
                match Command::new("sh")
                    .arg("-c")
                    .arg("lscpu | grep 'Model name' | cut -d ':' -f 2 | xargs")
                    .output()
                {
                    Ok(lscpu_output) => {
                        if lscpu_output.status.success() {
                            let cpu_model = String::from_utf8_lossy(&lscpu_output.stdout)
                                .trim()
                                .to_string();
                            if !cpu_model.is_empty() {
                                cpu_model
                            } else {
                                "CPU model not found".to_string()
                            }
                        } else {
                            "Failed to get CPU info with lscpu".to_string()
                        }
                    }
                    Err(_) => "lscpu command not found".to_string(),
                }
            }
        }
        Err(_) => {
            match Command::new("sh")
                .arg("-c")
                .arg("lscpu | grep 'Model name' | cut -d ':' -f 2 | xargs")
                .output()
            {
                Ok(lscpu_output) => {
                    if lscpu_output.status.success() {
                        let cpu_model = String::from_utf8_lossy(&lscpu_output.stdout)
                            .trim()
                            .to_string();
                        if !cpu_model.is_empty() {
                            cpu_model
                        } else {
                            "CPU model not found".to_string()
                        }
                    } else {
                        "Failed to get CPU info with lscpu".to_string()
                    }
                }
                Err(_) => "lscpu command not found".to_string(),
            }
        }
    }
}

pub fn get_gpu_info() -> String {
    match Command::new("sh")
        .arg("-c")
        .arg("lspci | grep -i vga")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout).to_string();
                let gpu_info = extract_brackets(&output_str);
                match gpu_info {
                    Some(gpu_info) => gpu_info.trim().to_string(),
                    None => "Failed to get GPU info".to_string(),
                }
            } else {
                "Failed to get GPU info".to_string()
            }
        }
        Err(_) => "lspci not found".to_string(),
    }
}

pub fn get_memory_info() -> String {
    match Command::new("sh")
        .arg("-c")
        .arg("grep -E 'MemTotal|MemAvailable' /proc/meminfo | awk '{print $2}'")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = output_str.lines().collect();

                if lines.len() >= 2 {
                    if let (Ok(total_kb), Ok(available_kb)) = (
                        lines[0].trim().parse::<f64>(),
                        lines[1].trim().parse::<f64>(),
                    ) {
                        let total_gb = total_kb / 1024.0 / 1024.0;
                        let available_gb = available_kb / 1024.0 / 1024.0;
                        let used_gb = total_gb - available_gb;

                        format!("{:.2} GB / {:.2} GB", used_gb, total_gb)
                    } else {
                        "Failed to parse memory info".to_string()
                    }
                } else {
                    match (
                        Command::new("sh")
                            .arg("-c")
                            .arg("grep 'MemTotal' /proc/meminfo | awk '{print $2}'")
                            .output(),
                        Command::new("sh")
                            .arg("-c")
                            .arg("grep 'MemAvailable' /proc/meminfo | awk '{print $2}'")
                            .output(),
                    ) {
                        (Ok(total_out), Ok(available_out)) => {
                            if total_out.status.success() && available_out.status.success() {
                                let total_str = String::from_utf8_lossy(&total_out.stdout)
                                    .trim()
                                    .to_string();
                                let available_str = String::from_utf8_lossy(&available_out.stdout)
                                    .trim()
                                    .to_string();

                                if let (Ok(total_kb), Ok(available_kb)) =
                                    (total_str.parse::<f64>(), available_str.parse::<f64>())
                                {
                                    let total_gb = total_kb / 1024.0 / 1024.0;
                                    let available_gb = available_kb / 1024.0 / 1024.0;
                                    let used_gb = total_gb - available_gb;

                                    format!("{:.2} GB / {:.2} GB", used_gb, total_gb)
                                } else {
                                    "Failed to parse memory info (backup)".to_string()
                                }
                            } else {
                                "Failed to get memory info (backup)".to_string()
                            }
                        }
                        _ => "Failed to execute memory info commands (backup)".to_string(),
                    }
                }
            } else {
                "Failed to get memory info from /proc/meminfo".to_string()
            }
        }
        Err(_) => "Failed to get memory info".to_string(),
    }
}

pub fn get_monitor_info() -> String {
    match Command::new("hyprctl").arg("monitors").output() {
        Ok(output) => {
            if output.status.success() {
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
            } else {
                "Failed to get monitor info".to_string()
            }
        }
        Err(_) => "hyprctl not found".to_string(),
    }
}
