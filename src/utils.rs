use rust_i18n::t;
use std::{
    cmp::Ordering,
    collections::HashSet,
    env,
    error::Error,
    fs,
    io::{self, Write},
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
                        t!("version_parse_failed").to_string()
                    }
                    Err(_) => t!("json_parse_error").to_string(),
                }
            } else {
                t!("http_error", status_code = response.status_code).to_string()
            }
        }
        Err(e) => t!("request_failed", error = e).to_string(),
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

/// Expands all `source = <path>` occurrences in file `entry_path` recursively from str
pub fn expand_source_str(entry_path: &Path, entry: &str) -> Result<String, Box<dyn Error>> {
    let mut visited = HashSet::new();
    expand_file_recursive(entry_path, &mut visited, entry)
}

/// Expand all `source = <path>` occurrences in file `entry_path` recursively.
pub fn expand_source(entry_path: &Path) -> Result<String, Box<dyn Error>> {
    let mut visited = HashSet::new();
    expand_file_recursive(entry_path, &mut visited, "")
}

fn expand_file_recursive(
    path: &Path,
    visited: &mut HashSet<PathBuf>,
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
        if let Some(include_path_str) = parse_source_line(line) {
            let include_path = resolve_relative(&include_path_str, &resolved);
            let included_text = expand_file_recursive(&include_path, visited, "")
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
        .replace("<1280", "&lt;1280")
        .replace("<40%", "&lt;40%")
        .replace(
            "{{< relref \"#executing-with-rules\" >}}",
            "#executing-with-rules",
        )
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

pub const CONFIG_PATH: &str = ".config/hypr/hyprland.conf";
pub const HYPRVIZ_CONFIG_PATH: &str = ".config/hypr/hyprviz.conf";
pub const HYPRVIZ_PROFILES_PATH: &str = ".config/hypr/hyprviz/";
pub const BACKUP_SUFFIX: &str = "-bak";
pub const MAX_SAFE_INTEGER_F64: f64 = (1u64 << 53) as f64; // 2^53
