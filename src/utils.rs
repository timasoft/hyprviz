use crate::hyprland::MonitorSelector;
use rust_i18n::t;
use serde_json::Value;
use std::{
    borrow::Cow,
    cmp::Ordering,
    collections::{HashMap, HashSet},
    env,
    error::Error,
    fs,
    fs::File,
    io::{self, Write},
    os::unix::io::AsRawFd,
    path::{Path, PathBuf},
    process::Command,
    sync::{LazyLock, OnceLock},
};
use strum::IntoEnumIterator;

static IS_DEVELOPMENT_MODE: OnceLock<bool> = OnceLock::new();

pub fn initialize_development_mode() {
    let args: Vec<String> = env::args().collect();

    let has_dev_flag = args.iter().any(|arg| arg == "--dev");

    let is_dev = cfg!(debug_assertions) || has_dev_flag;

    IS_DEVELOPMENT_MODE
        .set(is_dev)
        .expect("Development mode already initialized");

    if is_dev {
        println!("Running in development mode");
    }
}

pub fn is_development_mode() -> bool {
    *IS_DEVELOPMENT_MODE
        .get()
        .expect("Development mode not initialized")
}

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

/// Returns the latest version of the GitHub repository
pub fn get_latest_version(repo: &str) -> String {
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);

    match minreq::get(&url)
        .with_header("User-Agent", "repository-updater")
        .send()
    {
        Ok(response) => {
            if response.status_code == 200 {
                match serde_json::from_str::<serde_json::Value>(response.as_str().unwrap_or("")) {
                    Ok(json) => {
                        if let Some(tag_name) = json.get("tag_name")
                            && let Some(version) = tag_name.as_str()
                        {
                            return version.to_string();
                        }
                        t!("utils.version_parse_failed").to_string()
                    }
                    Err(_) => t!("utils.json_parse_error").to_string(),
                }
            } else {
                t!("utils.http_error", status_code = response.status_code).to_string()
            }
        }
        Err(e) => t!("utils.request_failed", error = e).to_string(),
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

pub fn execute_command(cmd: &str, args: &[&str]) -> Result<std::process::Output, String> {
    Command::new(cmd)
        .args(args)
        .output()
        .map_err(|_| format!("{} not found", cmd))
}

pub fn execute_shell_command(shell_cmd: &str) -> Result<std::process::Output, String> {
    Command::new("sh")
        .arg("-c")
        .arg(shell_cmd)
        .output()
        .map_err(|_| "Failed to execute shell command".to_string())
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
    let final_path = resolve_symlink_fully(path)?;

    if let Some(parent) = final_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_path = generate_temp_path(&final_path)?;

    let result = atomic_replace(&temp_path, &final_path, data);

    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    result
}

pub fn resolve_symlink_fully(path: &Path) -> io::Result<PathBuf> {
    let mut current = path.to_path_buf();
    let mut iterations = 0;
    const MAX_DEPTH: u32 = 40;

    loop {
        let meta = match fs::symlink_metadata(&current) {
            Ok(m) => m,
            Err(e) if e.kind() == io::ErrorKind::NotFound => break,
            Err(e) => return Err(e),
        };

        if !meta.file_type().is_symlink() {
            break;
        }

        if iterations >= MAX_DEPTH {
            return Err(io::Error::other("too many levels of symbolic links"));
        }

        let target = fs::read_link(&current)?;
        current = if target.is_absolute() {
            target
        } else {
            current
                .parent()
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "cannot resolve symlink in root directory",
                    )
                })?
                .join(target)
        };

        iterations += 1;
    }

    Ok(current)
}

fn generate_temp_path(final_path: &Path) -> io::Result<PathBuf> {
    let parent = final_path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "path has no parent directory")
    })?;

    let stem = final_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid filename"))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    Ok(parent.join(format!(".{}.{}.tmp", stem, timestamp)))
}

fn atomic_replace(temp_path: &Path, final_path: &Path, data: &str) -> io::Result<()> {
    let mut temp_file = fs::File::create(temp_path)?;
    temp_file.write_all(data.as_bytes())?;
    temp_file.sync_all()?;

    match fs::rename(temp_path, final_path) {
        Ok(_) => {
            if let Some(parent) = final_path.parent() {
                fs::File::open(parent)?.sync_all()?;
            }
            Ok(())
        }
        // EXDEV
        Err(ref e) if e.raw_os_error() == Some(18) => {
            let temp_in_target = final_path.with_extension(".tmp");
            let mut temp_target = fs::File::create(&temp_in_target)?;
            temp_target.write_all(data.as_bytes())?;
            temp_target.sync_all()?;

            fs::rename(&temp_in_target, final_path)
                .and_then(|_| fs::remove_file(temp_path))
                .inspect_err(|_| {
                    let _ = fs::remove_file(&temp_in_target);
                })
        }
        Err(e) => {
            let _ = fs::remove_file(temp_path);
            Err(e)
        }
    }
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

/// Finds all files matching pattern `*.conf` in the hyprviz profile directory
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

/// Expands all `source = <path>` occurrences in file `entry_path` recursively from str
pub fn expand_source_str(entry_path: &Path, entry: &str) -> Result<String, Box<dyn Error>> {
    let mut visited = HashSet::new();

    let mut env_vars = HashMap::new();
    let home = env::var("HOME").unwrap_or_default();
    env_vars.insert("HOME".to_string(), home.clone());

    expand_file_recursive(entry_path, &mut visited, &mut env_vars, entry)
}

/// Expand all `source = <path>` occurrences in file `entry_path` recursively.
pub fn expand_source(entry_path: &Path) -> Result<String, Box<dyn Error>> {
    let mut visited = HashSet::new();

    let mut env_vars = HashMap::new();
    let home = env::var("HOME").unwrap_or_default();
    env_vars.insert("HOME".to_string(), home.clone());

    expand_file_recursive(entry_path, &mut visited, &mut env_vars, "")
}

fn expand_file_recursive(
    path: &Path,
    visited: &mut HashSet<PathBuf>,
    env_vars: &mut HashMap<String, String>,
    entry: &str,
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

    let content = if entry.is_empty() {
        fs::read_to_string(&resolved)
            .map_err(|e| format!("failed to read {}: {}", resolved.display(), e))?
    } else {
        entry.to_string()
    };

    let mut out = String::with_capacity(content.len());

    for line in content.lines() {
        if let Some((name, value)) = parse_env_var_line(line) {
            env_vars.insert(name, value);
            continue;
        }

        let processed_line = substitute_env_vars(line, env_vars);

        if let Some(include_path_str) = parse_source_line(&processed_line) {
            let include_path = resolve_relative(&include_path_str, &resolved);
            let included_text = expand_file_recursive(&include_path, visited, env_vars, "")
                .map_err(|e| format!("while including {}: {}", include_path.display(), e))?;
            out.push_str(&included_text);
            out.push('\n');
        } else {
            out.push_str(&processed_line);
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
        if c.is_alphanumeric() || c == '_' {
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

/// Parse a line and return Some((name, value)) if the line is a `$var = ...` assignment.
fn parse_env_var_line(line: &str) -> Option<(String, String)> {
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

    match chars.peek() {
        Some('$') => {
            chars.next();
        }
        _ => return None,
    }

    let mut name = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' {
            name.push(c);
            chars.next();
        } else {
            break;
        }
    }

    if name.is_empty() {
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
    let value = if first == '"' || first == '\'' {
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
                if out.is_empty() {
                    return None;
                } else {
                    return Some((name, out));
                }
            } else {
                out.push(c);
            }
        }
        if out.is_empty() {
            return None;
        } else {
            out
        }
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
        if res.is_empty() {
            return None;
        } else {
            res
        }
    };

    Some((name, value))
}

/// Substitute occurrences of `$var` in `line` with values from `env_vars`.
fn substitute_env_vars(line: &str, env_vars: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            let mut var_name = String::new();
            while let Some(&next_c) = chars.peek() {
                if next_c.is_alphanumeric() || next_c == '_' {
                    var_name.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            if !var_name.is_empty() {
                if let Some(val) = env_vars.get(&var_name) {
                    out.push_str(val);
                    continue;
                } else {
                    out.push('$');
                    out.push_str(&var_name);
                }
            } else {
                out.push('$');
            }
        } else {
            out.push(c);
        }
    }

    out
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

pub fn parse_top_level_options(config_str: &str, raw: bool) -> Vec<(String, String)> {
    let mut options = Vec::new();
    let mut brace_depth: usize = 0;

    for line in config_str.lines() {
        let trimmed_line = line.trim_start();

        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }

        let closing_braces_count = trimmed_line.chars().filter(|&c| c == '}').count();
        let opening_braces_count = trimmed_line.chars().filter(|&c| c == '{').count();

        brace_depth += opening_braces_count;
        brace_depth = brace_depth.saturating_sub(closing_braces_count);

        if brace_depth == 0
            && let Some(eq_pos) = trimmed_line.find('=')
        {
            let key = trimmed_line[..eq_pos].trim();
            if key.contains('{') || key.contains(':') {
                continue;
            }

            let value = trimmed_line[eq_pos + 1..].trim_start();

            if raw {
                options.push((line.to_string(), "".to_string()));
            } else {
                options.push((key.to_string(), value.to_string()));
            }
        }
    }

    options
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

pub fn markdown_to_pango(text: &str, guide_name: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    let mut in_bold = false;
    let mut in_italic = false;
    let mut in_code = false;

    while let Some(c) = chars.next() {
        match c {
            '`' => {
                if in_code {
                    if in_bold {
                        result.push_str("</b>");
                    }
                    in_bold = false;
                    in_code = false;
                } else {
                    in_code = true;
                    in_bold = true;
                    result.push_str("<b>");
                }
            }
            '*' => {
                if !in_code
                    && let Some(next) = chars.peek()
                    && *next == '*'
                {
                    chars.next();

                    if in_bold {
                        result.push_str("</b>");
                        in_bold = false;
                    } else {
                        result.push_str("<b>");
                        in_bold = true;
                    }
                } else {
                    result.push('*')
                }
            }
            '_' => {
                if !in_code {
                    if in_italic {
                        result.push_str("</i>");
                        in_italic = false;
                    } else {
                        result.push_str("<i>");
                        in_italic = true;
                    }
                } else {
                    result.push('_');
                }
            }
            '[' => {
                if !in_code {
                    let mut link_text = String::new();
                    let mut is_link = false;
                    let mut closing_bracket_found = false;

                    while let Some(&next) = chars.peek() {
                        if next == ']' {
                            chars.next();
                            closing_bracket_found = true;
                            break;
                        } else {
                            link_text.push(chars.next().unwrap());
                        }
                    }

                    if closing_bracket_found && chars.peek() == Some(&'(') {
                        chars.next();

                        let mut url = String::new();
                        while let Some(&next) = chars.peek() {
                            if next == ')' {
                                chars.next();
                                is_link = true;
                                break;
                            } else {
                                url.push(chars.next().unwrap());
                            }
                        }

                        if is_link {
                            let resolved_url = resolve_hyprwiki_url(&url, guide_name);
                            result.push_str(&format!(
                                "<a href=\"{}\">{}</a>",
                                escape_pango(&resolved_url),
                                escape_pango(&link_text)
                            ));
                        } else {
                            result.push('[');
                            result.push_str(&escape_pango(&link_text));
                            if closing_bracket_found {
                                result.push(']');
                            }
                            result.push('(');
                            result.push_str(&escape_pango(&url));
                        }
                    } else {
                        result.push('[');
                        result.push_str(&escape_pango(&link_text));
                        if closing_bracket_found {
                            result.push(']');
                        }
                    }
                } else {
                    result.push('[');
                }
            }
            '<' => {
                if !in_code
                    && chars.next_if(|&c| c == 'k').is_some()
                    && chars.next_if(|&c| c == 'e').is_some()
                    && chars.next_if(|&c| c == 'y').is_some()
                    && chars.next_if(|&c| c == '>').is_some()
                {
                    let mut key_content = String::new();
                    while chars.peek().is_some() {
                        if chars.next_if(|&c| c == '<').is_some()
                            && chars.next_if(|&c| c == '/').is_some()
                            && chars.next_if(|&c| c == 'k').is_some()
                            && chars.next_if(|&c| c == 'e').is_some()
                            && chars.next_if(|&c| c == 'y').is_some()
                            && chars.next_if(|&c| c == '>').is_some()
                        {
                            break;
                        }

                        key_content.push(chars.next().unwrap());
                    }

                    if !in_bold {
                        result.push_str("<b>");
                        in_bold = true;
                    }
                    result.push_str(&escape_pango(&key_content));
                    if in_bold {
                        result.push_str("</b>");
                        in_bold = false;
                    }
                } else {
                    result.push('<');
                }
            }
            '#' => {
                if !in_code {
                    let mut header_level = 1;
                    while chars.peek() == Some(&'#') {
                        header_level += 1;
                        chars.next();
                    }

                    while chars.peek() == Some(&' ') {
                        chars.next();
                    }

                    let mut header_text = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '\n' {
                            break;
                        }
                        header_text.push(chars.next().unwrap());
                    }

                    let size = match header_level {
                        1 => "x-large",
                        2 => "large",
                        3 => "medium",
                        _ => "medium",
                    };

                    result.push_str(&format!(
                        "<span size=\"{}\" weight=\"bold\">{}</span>",
                        size,
                        escape_pango(header_text.trim())
                    ));

                    while let Some(&c) = chars.peek() {
                        if c == '\n' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                } else {
                    result.push('#');
                }
            }
            _ => {
                if c == '\n' && in_code {
                    result.push(' ');
                } else {
                    result.push(c);
                }
            }
        }
    }

    if in_bold {
        result.push_str("</b>");
    }
    if in_italic {
        result.push_str("</i>");
    }

    escape_pango(&result)
}

fn escape_pango(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace("<br> ", "\n")
        .replace("<br>", "\n")
        .replace(">>", ">&gt;")
        .replace("<<", "&lt;<")
        // HARDCODED PATTERNS
        .replace("<NAME>", "&lt;NAME&gt;")
        .replace("<ИМЯ>", "&lt;ИМЯ&gt;")
        .replace("<名称>", "&lt;名称&gt;")
        .replace("<1280", "&lt;1280")
        .replace("<40%", "&lt;40%")
        .replace(
            "{{< relref \"#executing-with-rules\" >}}",
            "#executing-with-rules",
        )
        .replace("{{< relref \"#workspaces\" >}}", "#workspaces")
}

fn resolve_hyprwiki_url(url: &str, guide_name: &str) -> String {
    if url.contains("://") {
        return url.to_string();
    }

    let base_url = "https://wiki.hypr.land/";

    if let Some(stripped) = url.strip_prefix("../../") {
        return format!("{}{}", base_url, stripped);
    } else if let Some(stripped) = url.strip_prefix("../") {
        return format!("{}Configuring/{}", base_url, stripped);
    } else if let Some(stripped) = url.strip_prefix("./") {
        return format!("{}Configuring/{}", base_url, stripped);
    } else if url.starts_with('#') {
        return format!("{}Configuring/{}/{}", base_url, guide_name, url);
    }

    url.to_string()
}

pub fn get_system_locale() -> String {
    std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .map(|s| s.split('_').next().unwrap_or("en").to_string())
        .unwrap_or_else(|_| "en".to_string())
}

type NameWithCoords = (String, Result<(f64, f64, f64, f64), String>);

pub fn parse_coordinates(input: &str) -> NameWithCoords {
    let parts: Vec<&str> = input.split(',').map(|s| s.trim()).collect();

    let name = parts.first().map(|s| s.to_string()).unwrap_or_default();

    if parts.len() != 5 {
        let error = format!(
            "Error: expected 5 parts (NAME, X0, Y0, X1, Y1), got {}",
            parts.len()
        );
        return (name, Err(error));
    }

    let x0 = parts[1].parse::<f64>().map_err(|e| format!("X0: {}", e));
    let y0 = parts[2].parse::<f64>().map_err(|e| format!("Y0: {}", e));
    let x1 = parts[3].parse::<f64>().map_err(|e| format!("X1: {}", e));
    let y1 = parts[4].parse::<f64>().map_err(|e| format!("Y1: {}", e));

    match (x0.clone(), y0.clone(), x1.clone(), y1.clone()) {
        (Ok(x0), Ok(y0), Ok(x1), Ok(y1)) => (name, Ok((x0, y0, x1, y1))),
        _ => {
            let errors = [x0.err(), y0.err(), x1.err(), y1.err()]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("; ");

            (name, Err(errors))
        }
    }
}

pub fn after_second_comma(s: &str) -> &str {
    let mut comma_indices = s.char_indices().filter(|&(_, c)| c == ',');

    match (comma_indices.next(), comma_indices.next()) {
        (Some((_, _)), Some((second_comma_pos, _))) => &s[second_comma_pos..],
        _ => "",
    }
}

static KEYCODE_MAP: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    map.insert(9, "Escape");
    map.insert(67, "F1");
    map.insert(68, "F2");
    map.insert(69, "F3");
    map.insert(70, "F4");
    map.insert(71, "F5");
    map.insert(72, "F6");
    map.insert(73, "F7");
    map.insert(74, "F8");
    map.insert(75, "F9");
    map.insert(76, "F10");
    map.insert(95, "F11");
    map.insert(96, "F12");
    map.insert(127, "Pause");

    map.insert(10, "1");
    map.insert(11, "2");
    map.insert(12, "3");
    map.insert(13, "4");
    map.insert(14, "5");
    map.insert(15, "6");
    map.insert(16, "7");
    map.insert(17, "8");
    map.insert(18, "9");
    map.insert(19, "0");
    map.insert(20, "minus");
    map.insert(21, "equal");
    map.insert(22, "BackSpace");
    map.insert(23, "Tab");
    map.insert(24, "Q");
    map.insert(25, "W");
    map.insert(26, "E");
    map.insert(27, "R");
    map.insert(28, "T");
    map.insert(29, "Y");
    map.insert(30, "U");
    map.insert(31, "I");
    map.insert(32, "O");
    map.insert(33, "P");
    map.insert(34, "bracketleft");
    map.insert(35, "bracketright");
    map.insert(36, "Return");

    map.insert(38, "A");
    map.insert(39, "S");
    map.insert(40, "D");
    map.insert(41, "F");
    map.insert(42, "G");
    map.insert(43, "H");
    map.insert(44, "J");
    map.insert(45, "K");
    map.insert(46, "L");
    map.insert(47, "semicolon");
    map.insert(48, "apostrophe");
    map.insert(49, "grave");

    map.insert(52, "Z");
    map.insert(53, "X");
    map.insert(54, "C");
    map.insert(55, "V");
    map.insert(56, "B");
    map.insert(57, "N");
    map.insert(58, "M");
    map.insert(59, "comma");
    map.insert(60, "period");
    map.insert(61, "slash");
    map.insert(51, "backslash");

    map.insert(50, "SHIFT");
    map.insert(62, "SHIFT");
    map.insert(37, "CTRL");
    map.insert(105, "CTRL");
    map.insert(64, "ALT");
    map.insert(108, "ALT");
    map.insert(133, "SUPER");
    map.insert(134, "SUPER");
    map.insert(66, "CAPS");
    map.insert(77, "MOD2");
    map.insert(78, "MOD3");

    map.insert(65, "space");
    map.insert(135, "Menu");

    map.insert(110, "Home");
    map.insert(111, "Up");
    map.insert(112, "Page_Up");
    map.insert(113, "Left");
    map.insert(114, "Right");
    map.insert(115, "End");
    map.insert(116, "Down");
    map.insert(117, "Page_Down");
    map.insert(118, "Insert");
    map.insert(119, "Delete");

    map.insert(79, "KP_7");
    map.insert(80, "KP_8");
    map.insert(81, "KP_9");
    map.insert(82, "KP_Subtract");
    map.insert(83, "KP_4");
    map.insert(84, "KP_5");
    map.insert(85, "KP_6");
    map.insert(86, "KP_Add");
    map.insert(87, "KP_1");
    map.insert(88, "KP_2");
    map.insert(89, "KP_3");
    map.insert(90, "KP_0");
    map.insert(91, "KP_Decimal");
    map.insert(104, "KP_Enter");
    map.insert(63, "KP_Multiply");
    map.insert(106, "KP_Divide");
    map.insert(97, "KP_Equal");

    map.insert(166, "LaunchA");
    map.insert(167, "LaunchB");
    map.insert(168, "LaunchC");
    map.insert(170, "Calculator");
    map.insert(178, "Sleep");
    map.insert(179, "WakeUp");

    map.insert(107, "Print");
    map.insert(151, "Sys_Req");

    map.insert(183, "XF86Tools");
    map.insert(187, "XF86Mail");
    map.insert(216, "XF86WWW");
    map.insert(225, "XF86AudioMute");
    map.insert(226, "XF86AudioLowerVolume");
    map.insert(227, "XF86AudioRaiseVolume");
    map.insert(228, "XF86AudioPlay");
    map.insert(229, "XF86AudioStop");
    map.insert(230, "XF86AudioPrev");
    map.insert(231, "XF86AudioNext");
    map.insert(232, "XF86HomePage");
    map.insert(233, "XF86Refresh");
    map.insert(234, "XF86Search");
    map.insert(235, "XF86Favorites");
    map.insert(236, "XF86Back");
    map.insert(237, "XF86Forward");

    map.insert(94, "asciitilde");
    map.insert(102, "KP_Separator");

    map
});

pub fn keycode_to_en_key(keycode: u32) -> String {
    KEYCODE_MAP
        .get(&keycode)
        .map(|&s| s.to_string())
        .unwrap_or_else(|| format!("code:{}", keycode))
}

pub fn is_modifier(key: &str) -> bool {
    matches!(
        key,
        "SHIFT" | "CAPS" | "CTRL" | "ALT" | "MOD2" | "MOD3" | "SUPER" | "MOD5"
    )
}

pub fn get_available_resolutions_for_monitor(monitor_selector: &MonitorSelector) -> Vec<String> {
    let mut special_options = vec![
        "disable".to_string(),
        "addreserved".to_string(),
        "preferred".to_string(),
        "highres".to_string(),
        "highrr".to_string(),
        "maxwidth".to_string(),
    ];

    match Command::new("hyprctl").arg("monitors").arg("-j").output() {
        Ok(output) => {
            let mut target_monitor = None;
            if let Ok(json_str) = String::from_utf8(output.stdout)
                && let Ok(monitors) = serde_json::from_str::<Vec<Value>>(&json_str)
            {
                match monitor_selector {
                    MonitorSelector::Name(monitor_name) => {
                        target_monitor = monitors.iter().map(|m| m.to_owned()).find(|monitor| {
                            if let Some(name) = monitor.get("name").and_then(|n| n.as_str()) {
                                name == monitor_name
                            } else {
                                false
                            }
                        });
                    }
                    MonitorSelector::Description(monitor_description) => {
                        target_monitor = monitors.iter().map(|m| m.to_owned()).find(|monitor| {
                            if let Some(desc) = monitor.get("description").and_then(|d| d.as_str())
                                && desc == monitor_description
                            {
                                true
                            } else {
                                false
                            }
                        });
                    }
                    MonitorSelector::All => {
                        target_monitor = None;
                    }
                }
            }
            if let Some(monitor) = target_monitor
                && let Some(modes) = monitor.get("availableModes").and_then(|m| m.as_array())
            {
                let mut unique_resolutions = HashSet::new();

                for mode in modes {
                    if let Some(mode_str) = mode.as_str() {
                        unique_resolutions.insert(mode_str.to_string());
                    }
                }

                let mut res_vec: Vec<String> = unique_resolutions.into_iter().collect();
                res_vec.sort();
                special_options.extend(res_vec);
            }
        }
        Err(e) => {
            eprintln!("Failed to get monitor resolutions: {}", e);
        }
    }

    special_options
}

pub fn get_available_monitors(only_names: bool) -> HashSet<String> {
    let mut monitors = HashSet::new();

    match Command::new("hyprctl").arg("monitors").arg("-j").output() {
        Ok(output) => {
            if let Ok(json_str) = String::from_utf8(output.stdout)
                && let Ok(monitors_json) = serde_json::from_str::<Vec<Value>>(&json_str)
            {
                for monitor in monitors_json {
                    if let Some(name) = monitor.get("name").and_then(|n| n.as_str()) {
                        monitors.insert(name.to_string());
                    }
                    if !only_names
                        && let Some(desc) = monitor.get("description").and_then(|d| d.as_str())
                    {
                        monitors.insert(format!("desc:{}", desc));
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to get monitor names: {}", e);
        }
    }

    monitors
}

pub fn find_matching_bracket(input: &str, prefix: &str, closing: char) -> Option<usize> {
    if !input.starts_with(prefix) {
        return None;
    }

    let mut depth = 1;
    let mut idx = prefix.len();

    while idx < input.len() {
        let c = input.chars().nth(idx)?;
        if c == '[' {
            depth += 1;
        } else if c == closing {
            depth -= 1;
            if depth == 0 {
                return Some(idx);
            }
        }
        idx += 1;
    }

    None
}

pub fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

pub fn parse_int(value: &str) -> Option<i32> {
    value.parse::<i32>().ok()
}

pub fn join_with_separator<I, T>(iterable: I, separator: &str) -> String
where
    I: IntoIterator<Item = T>,
    T: ToString,
{
    iterable
        .into_iter()
        .map(|item| item.to_string())
        .collect::<Vec<String>>()
        .join(separator)
}

pub fn cow_to_static_str(cow: Cow<'static, str>) -> &'static str {
    match cow {
        Cow::Borrowed(s) => s,
        Cow::Owned(s) => Box::leak(s.into_boxed_str()),
    }
}

pub fn mute_stdout<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let stdout = io::stdout();
    let stdout_fd = stdout.as_raw_fd();

    let saved_stdout = unsafe { libc::dup(stdout_fd) };

    let null = File::open("/dev/null").unwrap();
    let null_fd = null.as_raw_fd();

    unsafe {
        libc::dup2(null_fd, stdout_fd);
    }

    let result = f();

    unsafe {
        libc::dup2(saved_stdout, stdout_fd);
        libc::close(saved_stdout);
    }

    io::stdout().flush().unwrap();

    result
}

pub trait HasDiscriminant {
    type Discriminant: IntoEnumIterator + PartialEq + Eq + Clone + Copy;

    fn to_discriminant(&self) -> Self::Discriminant;

    fn from_discriminant(discriminant: Self::Discriminant) -> Self;

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self;

    fn to_str_without_discriminant(&self) -> Option<String> {
        None
    }

    fn custom_split(_discriminant: Self::Discriminant) -> Option<fn(&str) -> Vec<&str>> {
        None
    }

    fn variant_index(&self) -> usize {
        let discriminant = self.to_discriminant();
        Self::Discriminant::iter()
            .position(|d| d == discriminant)
            .expect("Discriminant should always be found in the iterator")
    }
}

impl<T: IntoEnumIterator + Eq + Copy> HasDiscriminant for T {
    type Discriminant = Self;

    fn to_discriminant(&self) -> Self::Discriminant {
        *self
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        discriminant
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, _str: &str) -> Self {
        discriminant
    }
}

pub const CONFIG_PATH: &str = ".config/hypr/hyprland.conf";
pub const HYPRVIZ_CONFIG_PATH: &str = ".config/hypr/hyprviz.conf";
pub const HYPRVIZ_PROFILES_PATH: &str = ".config/hypr/hyprviz/";
pub const BACKUP_SUFFIX: &str = "-bak";

/// 1 / 255
pub const ONE_OVER_255: f64 = 1.0 / 255.0;
/// 9007199254740992.0
pub const MAX_SAFE_INTEGER_F64: f64 = (1u64 << 53) as f64; // 2^53
/// -9007199254740992.0
pub const MIN_SAFE_INTEGER_F64: f64 = -MAX_SAFE_INTEGER_F64; // -2^53
/// 140737488355328
pub const MAX_SAFE_STEP_0_01_F64: f64 = (1u64 << 47) as f64; // 2^47
/// -140737488355328
pub const MIN_SAFE_STEP_0_01_F64: f64 = -MAX_SAFE_STEP_0_01_F64; // -2^47
