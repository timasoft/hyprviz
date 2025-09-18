use std::{
    cmp::Ordering,
    collections::HashSet,
    env,
    error::Error,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

pub fn get_config_path(write: bool, profile: &str) -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let base_path = Path::new(&home);

    if write {
        if !profile.is_empty() && profile != "Default" {
            let hyprviz_dir = Path::new(HYPRVIZ_PROFILES_PATH);
            let profile_filename = format!("{}.conf", profile);
            base_path.join(hyprviz_dir).join(profile_filename)
        } else {
            base_path.join(HYPRVIZ_CONFIG_PATH)
        }
    } else {
        base_path.join(CONFIG_PATH)
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

/// Returns the latest version of the GitHub repository
pub fn get_latest_version(repo: &str) -> String {
    match Command::new("curl")
        .arg("-s")
        .arg("-H")
        .arg("User-Agent: repository-updater")
        .arg(format!(
            "https://api.github.com/repos/{}/releases/latest",
            repo,
        ))
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout).to_string();
                if let Some(tag_start) = response.find(r#""tag_name":"#) {
                    let after_tag = &response[tag_start + 12..];
                    if let Some(tag_end) = after_tag.find('"') {
                        return after_tag[..tag_end].to_string();
                    }
                }
                "Version parse failed".to_string()
            } else {
                "API request failed".to_string()
            }
        }
        Err(_) => {
            match Command::new("wget")
                .arg("-qO-")
                .arg("-U")
                .arg("repository-updater")
                .arg(format!(
                    "https://api.github.com/repos/{}/releases/latest",
                    repo,
                ))
                .output()
            {
                Ok(output) => {
                    if output.status.success() {
                        let response = String::from_utf8_lossy(&output.stdout).to_string();
                        if let Some(tag_start) = response.find(r#""tag_name":"#) {
                            let after_tag = &response[tag_start + 12..];
                            if let Some(tag_end) = after_tag.find('"') {
                                return after_tag[..tag_end].to_string();
                            }
                        }
                        "Version parse failed".to_string()
                    } else {
                        "API request failed (wget)".to_string()
                    }
                }
                Err(_) => "curl/wget not found".to_string(),
            }
        }
    }
}

pub fn compare_versions(current: &str, latest: &str) -> Ordering {
    let current_parts: Vec<u32> = current
        .trim_start_matches('v')
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    let latest_parts: Vec<u32> = latest
        .trim_start_matches('v')
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    for i in 0..3 {
        let current_val = current_parts.get(i).copied().unwrap_or(0);
        let latest_val = latest_parts.get(i).copied().unwrap_or(0);

        match current_val.cmp(&latest_val) {
            Ordering::Equal => continue,
            ordering => return ordering,
        }
    }

    Ordering::Equal
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

fn extract_brackets(s: &str) -> Option<&str> {
    let start = s.find('[')?;
    let end = s.find(']')?;
    if start < end {
        Some(&s[start + 1..end])
    } else {
        None
    }
}

pub fn check_last_non_empty_line_contains(file_content: &str, expected_text: &str) -> bool {
    let last_non_empty = file_content
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty());

    match last_non_empty {
        Some(line) => line.trim().contains(expected_text),
        None => false,
    }
}

pub fn atomic_write(path: &Path, data: &str) -> io::Result<()> {
    let temp_path = path.with_extension("tmp");

    let result = || -> Result<(), io::Error> {
        let mut temp_file = fs::File::create(&temp_path)?;
        temp_file.write_all(data.as_bytes())?;
        temp_file.sync_all()?;
        fs::rename(&temp_path, path)?;
        Ok(())
    }();

    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    result
}

/// Updates the source line in Hyprland config for the specified profile
/// For "Default" profile: `source = ./hyprviz.conf`
/// For other profiles: `source = ./hyprviz/{profile}.conf`
/// Replaces the last non-empty line starting with "source = ./hyprviz" or appends if not found
pub fn update_source_line(config_path: &PathBuf, profile: &str) -> io::Result<()> {
    let content = fs::read_to_string(config_path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    let mut target_index = None;
    for (i, line) in lines.iter().enumerate().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with("source = ./hyprviz") {
            target_index = Some(i);
            break;
        }
    }

    let new_source = if profile == "Default" {
        "source = ./hyprviz.conf".to_string()
    } else {
        format!("source = ./hyprviz/{}.conf", profile)
    };

    if let Some(idx) = target_index {
        lines[idx] = new_source;
    } else {
        lines.push(new_source);
    }

    let new_content = lines.join("\n") + "\n";
    atomic_write(config_path, &new_content)
}

pub fn get_current_profile(file_content: &str) -> String {
    let last_non_empty_line = file_content
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty());

    if let Some(line) = last_non_empty_line {
        let prefix = "source = ./hyprviz/";
        let suffix = ".conf";

        if let Some(start_prefix_idx) = line.find(prefix) {
            let start_profile_idx = start_prefix_idx + prefix.len();

            let line_after_prefix = &line[start_profile_idx..];
            if let Some(start_suffix_idx_relative) = line_after_prefix.find(suffix) {
                let start_suffix_idx_absolute = start_profile_idx + start_suffix_idx_relative;
                let end_suffix_idx_absolute = start_suffix_idx_absolute + suffix.len();

                let is_valid_ending = end_suffix_idx_absolute == line.len()
                    || line[end_suffix_idx_absolute..]
                        .starts_with(|c: char| c.is_whitespace() || c == '#');

                if is_valid_ending && start_profile_idx < start_suffix_idx_absolute {
                    let extracted = &line[start_profile_idx..start_suffix_idx_absolute];
                    if !extracted.is_empty() {
                        return extracted.to_string();
                    }
                }
            }
        }
    }

    "Default".to_string()
}

/// Finds all files matching pattern `hyprviz_*.conf` in the same directory as default config file
pub fn find_all_profiles() -> Option<Vec<String>> {
    let config_path = get_config_path(true, "None");

    let parent_dir = config_path.parent()?;

    let entries = fs::read_dir(parent_dir).ok()?;

    let mut profiles = Vec::new();
    let suffix = ".conf";

    for entry in entries.flatten() {
        let file_name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };
        if file_name.ends_with(suffix) {
            let profile_name = &file_name[..file_name.len() - suffix.len()];

            if !profile_name.is_empty() {
                profiles.push(profile_name.to_string());
            }
        }
    }

    if profiles.is_empty() {
        None
    } else {
        Some(profiles)
    }
}

/// Expand all `source = <path>` occurrences in file `entry_path` recursively.
pub fn expand_source(entry_path: &Path) -> Result<String, Box<dyn Error>> {
    let mut visited = HashSet::new();
    expand_file_recursive(entry_path, &mut visited)
}

fn expand_file_recursive(
    path: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<String, Box<dyn Error>> {
    let resolved = expand_tilde(path)?;
    let canonical = resolved
        .canonicalize()
        .map_err(|e| format!("failed to canonicalize {}: {}", resolved.display(), e))?;

    if !visited.insert(canonical.clone()) {
        return Err(format!(
            "cycle detected while including file: {}",
            canonical.display()
        )
        .into());
    }

    let content = fs::read_to_string(&resolved)
        .map_err(|e| format!("failed to read {}: {}", resolved.display(), e))?;

    let mut out = String::with_capacity(content.len());

    for line in content.lines() {
        if let Some(include_path_str) = parse_source_line(line) {
            let include_path = resolve_relative(&include_path_str, &resolved);
            let included_text = expand_file_recursive(&include_path, visited)
                .map_err(|e| format!("while including {}: {}", include_path.display(), e))?;
            out.push_str(&included_text);
            out.push('\n');
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }

    visited.remove(&canonical);
    Ok(out)
}

/// Parse a line and return Some(path) if the line is a `source = ...` assignment.
fn parse_source_line(line: &str) -> Option<String> {
    // 1) skip leading whitespace
    // 2) read an identifier token (letters, digits, _)
    // 3) check it's "source"
    // 4) skip whitespace, expect '='
    // 5) skip whitespace, parse rhs:
    //    - if starts with " or ' -> parse quoted string (supporting escaped quotes \" and \')
    //    - else read until first unquoted '#' or end, trim trailing whitespace

    let mut chars = line.chars().peekable();

    fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }
    }

    skip_whitespace(&mut chars);
    let mut ident = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() {
            ident.push(c);
            chars.next();
        } else {
            break;
        }
    }

    if ident != "source" {
        return None;
    }

    skip_whitespace(&mut chars);
    match chars.peek() {
        Some('=') => {
            chars.next();
        }
        _ => return None,
    }

    skip_whitespace(&mut chars);
    let rhs = chars.collect::<String>();
    let rhs = rhs.trim_start();

    if rhs.is_empty() {
        return None;
    }

    let first = rhs.chars().next().unwrap();
    if first == '"' || first == '\'' {
        let quote = first;
        let mut out = String::new();
        let mut escaped = false;
        let chars = rhs.chars().skip(1);
        for c in chars {
            if escaped {
                out.push(c);
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == quote {
                // done
                if out.is_empty() {
                    return None;
                } else {
                    return Some(out);
                }
            } else {
                out.push(c);
            }
        }
        if out.is_empty() { None } else { Some(out) }
    } else {
        let mut out = String::new();
        for c in rhs.chars() {
            if c == '#' {
                break;
            } else {
                out.push(c);
            }
        }
        let res = out.trim_end().to_string();
        if res.is_empty() { None } else { Some(res) }
    }
}

/// Resolve a possibly relative include path `p` (string) relative to `base_file`'s parent.
pub fn resolve_relative(p: &str, base_file: &Path) -> PathBuf {
    let p_expanded = expand_tilde_str(p);
    let p_path = Path::new(&p_expanded);
    if p_path.is_absolute() {
        p_path.to_path_buf()
    } else {
        base_file
            .parent()
            .map(|d| d.join(p_path))
            .unwrap_or_else(|| p_path.to_path_buf())
    }
}

/// Expand `~` in a Path if present.
pub fn expand_tilde(path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let s = path.to_string_lossy();
    Ok(PathBuf::from(expand_tilde_str(&s)))
}

fn expand_tilde_str(s: &str) -> String {
    if let Some(home) = env::var_os("HOME") {
        if s == "~" {
            return home.to_string_lossy().into_owned();
        } else if s.starts_with("~/") {
            let mut home_s = home.to_string_lossy().into_owned();
            home_s.push_str(&s[1..]);
            return home_s;
        }
    }
    s.to_string()
}

pub const CONFIG_PATH: &str = ".config/hypr/hyprland.conf";
pub const HYPRVIZ_CONFIG_PATH: &str = ".config/hypr/hyprviz.conf";
pub const HYPRVIZ_PROFILES_PATH: &str = ".config/hypr/hyprviz/";
pub const BACKUP_SUFFIX: &str = "-bak";
pub const MAX_SAFE_INTEGER_F64: f64 = (1u64 << 53) as f64; // 2^53
