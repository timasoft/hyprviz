use rust_i18n::t;
use serde_json::Value;
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    env,
    error::Error,
    fmt::Display,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
    sync::LazyLock,
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
        .replace("<名称>", "&lt;名称&gt;")
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

#[derive(Default, Debug, Clone)]
pub enum MonitorSelector {
    #[default]
    All,
    Name(String),
    Description(String),
}

impl FromStr for MonitorSelector {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(MonitorSelector::All),
            s => {
                if let Some(stripped) = s.strip_prefix("desc:") {
                    Ok(MonitorSelector::Description(stripped.to_string()))
                } else {
                    Ok(MonitorSelector::Name(s.to_string()))
                }
            }
        }
    }
}

impl Display for MonitorSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorSelector::All => write!(f, ""),
            MonitorSelector::Name(name) => write!(f, "{}", name),
            MonitorSelector::Description(desc) => write!(f, "desc:{}", desc),
        }
    }
}

pub enum Position {
    Auto,
    AutoRight,
    AutoLeft,
    AutoUp,
    AutoDown,
    AutoCenterRight,
    AutoCenterLeft,
    AutoCenterUp,
    AutoCenterDown,
    Coordinates(i64, i64),
}

impl Position {
    pub fn get_fancy_list() -> [String; 10] {
        [
            t!("auto").to_string(),
            t!("auto_right").to_string(),
            t!("auto_left").to_string(),
            t!("auto_up").to_string(),
            t!("auto_down").to_string(),
            t!("auto_center_right").to_string(),
            t!("auto_center_left").to_string(),
            t!("auto_center_up").to_string(),
            t!("auto_center_down").to_string(),
            t!("coordinates").to_string(),
        ]
    }

    pub fn get_list() -> [&'static str; 10] {
        [
            "auto",
            "auto-right",
            "auto-left",
            "auto-up",
            "auto-down",
            "auto-center-right",
            "auto-center-left",
            "auto-center-up",
            "auto-center-down",
            "coordinates",
        ]
    }

    pub fn from_id(id: usize, x: Option<i64>, y: Option<i64>) -> Self {
        match id {
            0 => Position::Auto,
            1 => Position::AutoRight,
            2 => Position::AutoLeft,
            3 => Position::AutoUp,
            4 => Position::AutoDown,
            5 => Position::AutoCenterRight,
            6 => Position::AutoCenterLeft,
            7 => Position::AutoCenterUp,
            8 => Position::AutoCenterDown,
            _ => Position::Coordinates(x.unwrap_or(0), y.unwrap_or(0)),
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Position::Auto => write!(f, "auto"),
            Position::AutoRight => write!(f, "auto-right"),
            Position::AutoLeft => write!(f, "auto-left"),
            Position::AutoUp => write!(f, "auto-up"),
            Position::AutoDown => write!(f, "auto-down"),
            Position::AutoCenterRight => write!(f, "auto-center-right"),
            Position::AutoCenterLeft => write!(f, "auto-center-left"),
            Position::AutoCenterUp => write!(f, "auto-center-up"),
            Position::AutoCenterDown => write!(f, "auto-center-down"),
            Position::Coordinates(x, y) => write!(f, "{}x{}", x, y),
        }
    }
}

impl FromStr for Position {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Position::Auto),
            "auto-right" => Ok(Position::AutoRight),
            "auto-left" => Ok(Position::AutoLeft),
            "auto-up" => Ok(Position::AutoUp),
            "auto-down" => Ok(Position::AutoDown),
            "auto-center-right" => Ok(Position::AutoCenterRight),
            "auto-center-left" => Ok(Position::AutoCenterLeft),
            "auto-center-up" => Ok(Position::AutoCenterUp),
            position => {
                if let Some((x_str, y_str)) = position.split_once('x') {
                    let x = x_str.parse::<i64>().map_err(|_| ())?;
                    let y = y_str.parse::<i64>().map_err(|_| ())?;
                    Ok(Position::Coordinates(x, y))
                } else {
                    Err(())
                }
            }
        }
    }
}

impl Clone for Position {
    fn clone(&self) -> Self {
        match self {
            Position::Auto => Position::Auto,
            Position::AutoRight => Position::AutoRight,
            Position::AutoLeft => Position::AutoLeft,
            Position::AutoUp => Position::AutoUp,
            Position::AutoDown => Position::AutoDown,
            Position::AutoCenterRight => Position::AutoCenterRight,
            Position::AutoCenterLeft => Position::AutoCenterLeft,
            Position::AutoCenterUp => Position::AutoCenterUp,
            Position::AutoCenterDown => Position::AutoCenterDown,
            _ => Position::Coordinates(0, 0),
        }
    }
}

pub enum Scale {
    Auto,
    Manual(f64),
}

impl Scale {
    pub fn get_fancy_list() -> [String; 2] {
        [t!("auto").to_string(), t!("manual").to_string()]
    }

    pub fn from_id(id: usize, value: Option<f64>) -> Self {
        match id {
            0 => Scale::Auto,
            _ => Scale::Manual(value.unwrap_or(1.0)),
        }
    }
}

impl Display for Scale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scale::Auto => write!(f, "auto"),
            Scale::Manual(scale) => write!(f, "{:.2}", scale),
        }
    }
}

impl FromStr for Scale {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Scale::Auto),
            scale => Ok(Scale::Manual(scale.parse::<f64>().map_err(|_| ())?)),
        }
    }
}

impl Clone for Scale {
    fn clone(&self) -> Self {
        match self {
            Scale::Auto => Scale::Auto,
            Scale::Manual(scale) => Scale::Manual(*scale),
        }
    }
}

pub enum Cm {
    Auto,
    Srgb,
    Dcip3,
    Dp3,
    Adobe,
    Wide,
    Edid,
    Hdr,
    Hdredid,
}

impl Cm {
    pub fn get_fancy_list() -> [&'static str; 9] {
        [
            "Auto",
            "SRGB",
            "CDI-P3",
            "DP3",
            "AdobeRGB",
            "WideGamut",
            "EDID",
            "HDR",
            "HDR-EDID",
        ]
    }

    pub fn from_id(id: u32) -> Self {
        match id {
            0 => Cm::Auto,
            1 => Cm::Srgb,
            2 => Cm::Dcip3,
            3 => Cm::Dp3,
            4 => Cm::Adobe,
            5 => Cm::Wide,
            6 => Cm::Edid,
            7 => Cm::Hdr,
            8 => Cm::Hdredid,
            _ => Cm::Auto,
        }
    }
}

impl Display for Cm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cm::Auto => write!(f, "auto"),
            Cm::Srgb => write!(f, "srgb"),
            Cm::Dcip3 => write!(f, "dcip3"),
            Cm::Dp3 => write!(f, "dp3"),
            Cm::Adobe => write!(f, "adobe"),
            Cm::Wide => write!(f, "wide"),
            Cm::Edid => write!(f, "edid"),
            Cm::Hdr => write!(f, "hdr"),
            Cm::Hdredid => write!(f, "hdredid"),
        }
    }
}

impl FromStr for Cm {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Cm::Auto),
            "srgb" => Ok(Cm::Srgb),
            "dcip3" => Ok(Cm::Dcip3),
            "dp3" => Ok(Cm::Dp3),
            "adobe" => Ok(Cm::Adobe),
            "wide" => Ok(Cm::Wide),
            "edid" => Ok(Cm::Edid),
            "hdr" => Ok(Cm::Hdr),
            "hdredid" => Ok(Cm::Hdredid),
            _ => Err(()),
        }
    }
}

pub struct MonitorState {
    pub resolution: String,
    pub position: Position,
    pub scale: Scale,
    pub mirror: Option<String>,
    pub bitdepth: Option<u8>,
    pub cm: Option<Cm>,
    pub sdrbrightness: Option<f64>,
    pub sdrsaturation: Option<f64>,
    pub vrr: Option<u8>,
    pub transform: Option<u8>,
}

impl Display for MonitorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let resolution = self.resolution.clone();
        let position = format!(", {}", self.position);
        let scale = format!(", {}", self.scale);
        let mirror = match &self.mirror {
            Some(mirror) => format!(", mirror, {}", mirror),
            None => "".to_string(),
        };
        let bitdepth = match &self.bitdepth {
            Some(bitdepth) => format!(", bitdepth, {}", bitdepth),
            None => "".to_string(),
        };
        let cm = match &self.cm {
            Some(cm) => format!(", cm, {}", cm),
            None => "".to_string(),
        };
        let sdrbrightness = match &self.sdrbrightness {
            Some(sdrbrightness) => format!(", sdrbrightness, {}", sdrbrightness),
            None => "".to_string(),
        };
        let sdrsaturation = match &self.sdrsaturation {
            Some(sdrsaturation) => format!(", sdrsaturation, {}", sdrsaturation),
            None => "".to_string(),
        };
        let vrr = match &self.vrr {
            Some(vrr) => format!(", vrr, {}", vrr),
            None => "".to_string(),
        };
        let transform = match &self.transform {
            Some(transform) => format!(", transform, {}", transform),
            None => "".to_string(),
        };

        write!(
            f,
            "{}{}{}{}{}{}{}{}{}{}",
            resolution,
            position,
            scale,
            mirror,
            bitdepth,
            cm,
            sdrbrightness,
            sdrsaturation,
            vrr,
            transform
        )
    }
}

pub enum Monitor {
    Enabled(MonitorState),
    Disabled,
    AddReserved(i64, i64, i64, i64),
}

pub fn parse_monitor(input: &str) -> (MonitorSelector, Monitor) {
    let values = input
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();
    let monitor_selector =
        MonitorSelector::from_str(values.first().unwrap_or(&"".to_string()).as_str())
            .unwrap_or_default();

    let state = values.get(1).unwrap_or(&"preferred".to_string()).to_owned();

    match state.as_str() {
        "disable" => (monitor_selector, Monitor::Disabled),
        "addreserved" => (
            monitor_selector,
            Monitor::AddReserved(
                values
                    .get(2)
                    .unwrap_or(&"0".to_string())
                    .parse::<i64>()
                    .unwrap_or(0),
                values
                    .get(3)
                    .unwrap_or(&"0".to_string())
                    .parse::<i64>()
                    .unwrap_or(0),
                values
                    .get(4)
                    .unwrap_or(&"0".to_string())
                    .parse::<i64>()
                    .unwrap_or(0),
                values
                    .get(5)
                    .unwrap_or(&"0".to_string())
                    .parse::<i64>()
                    .unwrap_or(0),
            ),
        ),
        resolution => {
            let mut monitor_state = MonitorState {
                resolution: resolution.to_string(),
                position: {
                    Position::from_str(values.get(2).unwrap_or(&"auto".to_string()).as_str())
                        .unwrap_or(Position::Auto)
                },
                scale: {
                    Scale::from_str(values.get(3).unwrap_or(&"auto".to_string()).as_str())
                        .unwrap_or(Scale::Auto)
                },
                mirror: None,
                bitdepth: None,
                cm: None,
                sdrbrightness: None,
                sdrsaturation: None,
                vrr: None,
                transform: None,
            };

            for i in 4..values.len() {
                match values.get(i).unwrap_or(&"".to_string()).as_str() {
                    "mirror" => {
                        monitor_state.mirror =
                            Some(values.get(i + 1).unwrap_or(&"".to_string()).to_owned());
                    }
                    "bitdepth" => {
                        monitor_state.bitdepth = Some(
                            values
                                .get(i + 1)
                                .unwrap_or(&"10".to_string())
                                .parse::<u8>()
                                .unwrap_or(0),
                        );
                    }
                    "cm" => {
                        monitor_state.cm = Some(
                            match values
                                .get(i + 1)
                                .unwrap_or(&"auto".to_string())
                                .to_owned()
                                .as_str()
                            {
                                "auto" => Cm::Auto,
                                "srgb" => Cm::Srgb,
                                "dcip3" => Cm::Dcip3,
                                "dp3" => Cm::Dp3,
                                "adobe" => Cm::Adobe,
                                "wide" => Cm::Wide,
                                "edid" => Cm::Edid,
                                "hdr" => Cm::Hdr,
                                "hdredid" => Cm::Hdredid,
                                _ => Cm::Auto,
                            },
                        );
                    }
                    "sdrbrightness" => {
                        monitor_state.sdrbrightness = Some(
                            values
                                .get(i + 1)
                                .unwrap_or(&"1.0".to_string())
                                .parse::<f64>()
                                .unwrap_or(1.0),
                        )
                    }
                    "sdrsaturation" => {
                        monitor_state.sdrsaturation = Some(
                            values
                                .get(i + 1)
                                .unwrap_or(&"1.0".to_string())
                                .parse::<f64>()
                                .unwrap_or(1.0),
                        );
                    }
                    "vrr" => {
                        monitor_state.vrr = Some(
                            values
                                .get(i + 1)
                                .unwrap_or(&"0".to_string())
                                .parse::<u8>()
                                .unwrap_or(0),
                        );
                    }
                    "transform" => {
                        monitor_state.transform = Some(
                            values
                                .get(i + 1)
                                .unwrap_or(&"0".to_string())
                                .parse::<u8>()
                                .unwrap_or(0),
                        );
                    }
                    _ => {}
                }
            }

            (monitor_selector, Monitor::Enabled(monitor_state))
        }
    }
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

#[derive(Clone, Debug)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug)]
pub enum WorkspaceSelectorNamed {
    IsNamed(bool),
    Starts(String),
    Ends(String),
}

impl Display for WorkspaceSelectorNamed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceSelectorNamed::IsNamed(is_named) => write!(f, "n[{}]", is_named),
            WorkspaceSelectorNamed::Starts(prefix) => write!(f, "n[s:{}]", prefix),
            WorkspaceSelectorNamed::Ends(suffix) => write!(f, "n[e:{}]", suffix),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct WorkspaceSelectorWindowCountFlags {
    pub tiled: bool,
    pub floating: bool,
    pub groups: bool,
    pub visible: bool,
    pub pinned: bool,
}

impl Display for WorkspaceSelectorWindowCountFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = String::new();

        if self.tiled {
            flags.push('t');
        }
        if self.floating {
            flags.push('f');
        }
        if self.groups {
            flags.push('g');
        }
        if self.visible {
            flags.push('v');
        }
        if self.pinned {
            flags.push('p');
        }
        write!(f, "{}", flags)
    }
}

impl FromStr for WorkspaceSelectorWindowCountFlags {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flags = WorkspaceSelectorWindowCountFlags {
            tiled: false,
            floating: false,
            groups: false,
            visible: false,
            pinned: false,
        };

        for c in s.chars() {
            match c {
                't' => flags.tiled = true,
                'f' => flags.floating = true,
                'g' => flags.groups = true,
                'v' => flags.visible = true,
                'p' => flags.pinned = true,
                _ => return Err(format!("Invalid flag: {}", c)),
            }
        }

        Ok(flags)
    }
}

#[derive(Clone, Debug)]
pub enum WorkspaceSelectorWindowCount {
    Range {
        flags: WorkspaceSelectorWindowCountFlags,
        range_start: u32,
        range_end: u32,
    },
    Single {
        flags: WorkspaceSelectorWindowCountFlags,
        count: u32,
    },
}

impl Display for WorkspaceSelectorWindowCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceSelectorWindowCount::Range {
                flags,
                range_start,
                range_end,
            } => {
                write!(f, "w[{}{}-{}]", flags, range_start, range_end)
            }
            WorkspaceSelectorWindowCount::Single { flags, count } => {
                write!(f, "w[{}{}]", flags, count)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum WorkspaceSelector {
    None,
    Range(Range),
    Special(bool),
    Named(WorkspaceSelectorNamed),
    Monitor(MonitorSelector),
    WindowCount(WorkspaceSelectorWindowCount),
    Fullscreen(i32),
}

impl WorkspaceSelector {
    pub fn get_fancy_list() -> [String; 7] {
        [
            t!("none").to_string(),
            t!("range").to_string(),
            t!("special").to_string(),
            t!("named").to_string(),
            t!("monitor").to_string(),
            t!("window_count").to_string(),
            t!("fullscreen").to_string(),
        ]
    }

    pub fn get_id(&self) -> usize {
        match self {
            WorkspaceSelector::None => 0,
            WorkspaceSelector::Range { .. } => 1,
            WorkspaceSelector::Special(_) => 2,
            WorkspaceSelector::Named(_) => 3,
            WorkspaceSelector::Monitor(_) => 4,
            WorkspaceSelector::WindowCount(_) => 5,
            WorkspaceSelector::Fullscreen(_) => 6,
        }
    }
}

impl Display for WorkspaceSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceSelector::None => write!(f, ""),
            WorkspaceSelector::Range(Range { start, end }) => write!(f, "r[{}-{}]", start, end),
            WorkspaceSelector::Special(is_special) => {
                write!(f, "s[{}]", is_special)
            }
            WorkspaceSelector::Named(named) => {
                write!(f, "{}", named)
            }
            WorkspaceSelector::Monitor(monitor) => write!(f, "m[{}]", monitor),
            WorkspaceSelector::WindowCount(window_count) => {
                write!(f, "{}", window_count)
            }
            WorkspaceSelector::Fullscreen(state) => write!(f, "f[{}]", state),
        }
    }
}

#[derive(Debug)]
pub enum WorkspaceType {
    Named(String),
    Special(String),
    Numbered(u32),
    Selector(Vec<WorkspaceSelector>),
}

impl WorkspaceType {
    pub fn get_fancy_list() -> [String; 4] {
        [
            t!("named").to_string(),
            t!("special").to_string(),
            t!("numbered").to_string(),
            t!("selector").to_string(),
        ]
    }
}

impl Display for WorkspaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceType::Named(name) => write!(f, "name:{}", name),
            WorkspaceType::Special(name) => write!(f, "special:{}", name),
            WorkspaceType::Numbered(num) => write!(f, "{}", num),
            WorkspaceType::Selector(selector) => write!(
                f,
                "{}",
                selector
                    .iter()
                    .map(|selector| selector.to_string())
                    .collect::<Vec<String>>()
                    .join("")
            ),
        }
    }
}

#[derive(Debug, Default)]
pub enum Orientation {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Orientation::Left => write!(f, "left"),
            Orientation::Right => write!(f, "right"),
            Orientation::Top => write!(f, "top"),
            Orientation::Bottom => write!(f, "bottom"),
            Orientation::Center => write!(f, "center"),
        }
    }
}

impl FromStr for Orientation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left" => Ok(Orientation::Left),
            "right" => Ok(Orientation::Right),
            "top" => Ok(Orientation::Top),
            "bottom" => Ok(Orientation::Bottom),
            "center" => Ok(Orientation::Center),
            _ => Err(format!("Invalid orientation: {}", s)),
        }
    }
}

#[derive(Default, Debug)]
pub struct WorkspaceRules {
    pub monitor: Option<String>,
    pub default: Option<bool>,
    pub gaps_in: Option<i32>,
    pub gaps_out: Option<i32>,
    pub border_size: Option<i32>,
    pub border: Option<bool>,
    pub shadow: Option<bool>,
    pub rounding: Option<bool>,
    pub decorate: Option<bool>,
    pub persistent: Option<bool>,
    pub on_created_empty: Option<String>,
    pub default_name: Option<String>,
    pub layoutopt_orientation: Option<Orientation>,
}

impl Display for WorkspaceRules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rules = Vec::new();

        if let Some(monitor) = &self.monitor {
            rules.push(format!("monitor:{}", monitor));
        }
        if let Some(default) = self.default {
            rules.push(format!("default:{}", default));
        }
        if let Some(gaps_in) = self.gaps_in {
            rules.push(format!("gapsin:{}", gaps_in));
        }
        if let Some(gaps_out) = self.gaps_out {
            rules.push(format!("gapsout:{}", gaps_out));
        }
        if let Some(border_size) = self.border_size {
            rules.push(format!("bordersize:{}", border_size));
        }
        if let Some(border) = self.border {
            rules.push(format!("border:{}", border));
        }
        if let Some(shadow) = self.shadow {
            rules.push(format!("shadow:{}", shadow));
        }
        if let Some(rounding) = self.rounding {
            rules.push(format!("rounding:{}", rounding));
        }
        if let Some(decorate) = self.decorate {
            rules.push(format!("decorate:{}", decorate));
        }
        if let Some(persistent) = self.persistent {
            rules.push(format!("persistent:{}", persistent));
        }
        if let Some(on_created_empty) = &self.on_created_empty {
            rules.push(format!("on-created-empty:{}", on_created_empty));
        }
        if let Some(default_name) = &self.default_name {
            rules.push(format!("defaultName:{}", default_name));
        }
        if let Some(layoutopt_orientation) = &self.layoutopt_orientation {
            rules.push(format!("layoutopt:orientation:{}", layoutopt_orientation));
        }

        write!(f, "{}", rules.join(", "))
    }
}

#[derive(Debug)]
pub struct Workspace {
    pub workspace_type: WorkspaceType,
    pub rules: WorkspaceRules,
}

impl Display for Workspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let workspace_type = &self.workspace_type;
        let rules = &self.rules;
        if rules.monitor.is_some()
            || rules.default.is_some()
            || rules.gaps_in.is_some()
            || rules.gaps_out.is_some()
            || rules.border_size.is_some()
            || rules.border.is_some()
            || rules.shadow.is_some()
            || rules.rounding.is_some()
            || rules.decorate.is_some()
            || rules.persistent.is_some()
            || rules.on_created_empty.is_some()
            || rules.default_name.is_some()
            || rules.layoutopt_orientation.is_some()
        {
            write!(f, "{}, {}", workspace_type, rules)
        } else {
            write!(f, "{}", workspace_type)
        }
    }
}

pub fn parse_workspace(input: &str) -> Workspace {
    let values: Vec<String> = input.split(',').map(|s| s.trim().to_string()).collect();

    if values.is_empty() {
        return Workspace {
            workspace_type: WorkspaceType::Selector(Vec::new()),
            rules: WorkspaceRules::default(),
        };
    }

    let workspace_str = values.first().expect("Workspace type not found").as_str();
    let workspace_type = parse_workspace_type(workspace_str);
    let mut rules = WorkspaceRules::default();

    for rule_str in values.iter().skip(1) {
        parse_workspace_rule(rule_str, &mut rules);
    }

    Workspace {
        workspace_type,
        rules,
    }
}

pub fn parse_workspace_type(input: &str) -> WorkspaceType {
    if input.starts_with("name:") {
        let name = input.strip_prefix("name:").unwrap_or("").to_string();
        return WorkspaceType::Named(name);
    } else if input.starts_with("special:") {
        let name = input.strip_prefix("special:").unwrap_or("").to_string();
        return WorkspaceType::Special(name);
    } else if let Ok(num) = input.parse::<u32>() {
        return WorkspaceType::Numbered(num);
    }

    let mut selectors = Vec::new();
    let mut remaining = input.trim();

    while !remaining.is_empty() {
        let selector_result = parse_single_selector(remaining);

        match selector_result {
            Some((selector, rest)) => {
                selectors.push(selector);
                remaining = rest.trim();
            }
            None => {
                break;
            }
        }
    }

    if selectors.is_empty() {
        WorkspaceType::Selector(Vec::new())
    } else {
        WorkspaceType::Selector(selectors)
    }
}

pub fn parse_single_selector(input: &str) -> Option<(WorkspaceSelector, &str)> {
    if input.starts_with("r[") {
        if let Some(end_idx) = find_matching_bracket(input, "r[", ']') {
            let content = &input[2..end_idx];
            let rest = &input[end_idx + 1..];

            if let Some((start_str, end_str)) = content.split_once('-')
                && let (Ok(start), Ok(end)) = (start_str.parse::<u32>(), end_str.parse::<u32>())
            {
                return Some((WorkspaceSelector::Range(Range { start, end }), rest));
            }
        }
    } else if input.starts_with("s[") {
        if let Some(end_idx) = find_matching_bracket(input, "s[", ']') {
            let content = &input[2..end_idx];
            let rest = &input[end_idx + 1..];

            let is_special = parse_bool(content).unwrap_or(false);
            return Some((WorkspaceSelector::Special(is_special), rest));
        }
    } else if input.starts_with("n[") {
        if let Some(end_idx) = find_matching_bracket(input, "n[", ']') {
            let content = &input[2..end_idx];
            let rest = &input[end_idx + 1..];

            let selector = if content.starts_with("s:") {
                let prefix = content.strip_prefix("s:").unwrap_or("").to_string();
                WorkspaceSelector::Named(WorkspaceSelectorNamed::Starts(prefix))
            } else if content.starts_with("e:") {
                let suffix = content.strip_prefix("e:").unwrap_or("").to_string();
                WorkspaceSelector::Named(WorkspaceSelectorNamed::Ends(suffix))
            } else {
                let is_named = parse_bool(content).unwrap_or(false);
                WorkspaceSelector::Named(WorkspaceSelectorNamed::IsNamed(is_named))
            };

            return Some((selector, rest));
        }
    } else if input.starts_with("m[") {
        if let Some(end_idx) = find_matching_bracket(input, "m[", ']') {
            let content = &input[2..end_idx];
            let rest = &input[end_idx + 1..];

            let monitor = content.to_string();
            return Some((
                WorkspaceSelector::Monitor(MonitorSelector::from_str(&monitor).unwrap_or_default()),
                rest,
            ));
        }
    } else if input.starts_with("w[") {
        if let Some(end_idx) = find_matching_bracket(input, "w[", ']') {
            let content = &input[2..end_idx];
            let rest = &input[end_idx + 1..];

            let (flags_str, count_str) =
                if let Some(pos) = content.find(|c: char| !c.is_alphabetic()) {
                    (&content[..pos], &content[pos..])
                } else {
                    (content, "")
                };

            let flags = WorkspaceSelectorWindowCountFlags::from_str(flags_str).unwrap_or_default();

            let selector = if count_str.contains('-') {
                if let Some((start_str, end_str)) = count_str.split_once('-')
                    && let (Ok(start), Ok(end)) = (start_str.parse::<u32>(), end_str.parse::<u32>())
                {
                    WorkspaceSelector::WindowCount(WorkspaceSelectorWindowCount::Range {
                        flags,
                        range_start: start,
                        range_end: end,
                    })
                } else {
                    return None;
                }
            } else if !count_str.is_empty()
                && let Ok(count) = count_str.parse::<u32>()
            {
                WorkspaceSelector::WindowCount(WorkspaceSelectorWindowCount::Single {
                    flags,
                    count,
                })
            } else {
                return None;
            };

            return Some((selector, rest));
        }
    } else if input.starts_with("f[")
        && let Some(end_idx) = find_matching_bracket(input, "f[", ']')
    {
        let content = &input[2..end_idx];
        let rest = &input[end_idx + 1..];

        if let Ok(state) = content.parse::<i32>() {
            return Some((WorkspaceSelector::Fullscreen(state), rest));
        }
    }

    None
}

fn find_matching_bracket(input: &str, prefix: &str, closing: char) -> Option<usize> {
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

fn parse_workspace_rule(input: &str, rules: &mut WorkspaceRules) {
    let input = match input.strip_prefix("layoutopt:") {
        Some(value) => value,
        None => return,
    };
    let parts: Vec<&str> = input.splitn(2, ':').collect();
    if parts.len() != 2 {
        return;
    }

    let rule_name = parts[0].trim();
    let rule_value = parts[1].trim();

    match rule_name {
        "monitor" => rules.monitor = Some(rule_value.to_string()),
        "default" => rules.default = parse_bool(rule_value),
        "gapsin" => rules.gaps_in = parse_int(rule_value),
        "gapsout" => rules.gaps_out = parse_int(rule_value),
        "bordersize" => rules.border_size = parse_int(rule_value),
        "border" => rules.border = parse_bool(rule_value),
        "shadow" => rules.shadow = parse_bool(rule_value),
        "rounding" => rules.rounding = parse_bool(rule_value),
        "decorate" => rules.decorate = parse_bool(rule_value),
        "persistent" => rules.persistent = parse_bool(rule_value),
        "on-created-empty" => rules.on_created_empty = Some(rule_value.to_string()),
        "defaultName" => rules.default_name = Some(rule_value.to_string()),
        "orientation" => {
            rules.layoutopt_orientation =
                Some(Orientation::from_str(rule_value).expect("Invalid orientation"))
        }
        _ => {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationName {
    #[default]
    Global,
    Windows,
    WindowsIn,
    WindowsOut,
    WindowsMove,
    Layers,
    LayersIn,
    LayersOut,
    Fade,
    FadeIn,
    FadeOut,
    FadeSwitch,
    FadeShadow,
    FadeDim,
    FadeLayers,
    FadeLayersIn,
    FadeLayersOut,
    FadePopups,
    FadePopupsIn,
    FadePopupsOut,
    FadeDpms,
    Border,
    BorderAngle,
    Workspaces,
    WorkspacesIn,
    WorkspacesOut,
    SpecialWorkspace,
    SpecialWorkspaceIn,
    SpecialWorkspaceOut,
    ZoomFactor,
    MonitorAdded,
}

impl FromStr for AnimationName {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "global" => Ok(AnimationName::Global),
            "windows" => Ok(AnimationName::Windows),
            "windowsIn" => Ok(AnimationName::WindowsIn),
            "windowsOut" => Ok(AnimationName::WindowsOut),
            "windowsMove" => Ok(AnimationName::WindowsMove),
            "layers" => Ok(AnimationName::Layers),
            "layersIn" => Ok(AnimationName::LayersIn),
            "layersOut" => Ok(AnimationName::LayersOut),
            "fade" => Ok(AnimationName::Fade),
            "fadeIn" => Ok(AnimationName::FadeIn),
            "fadeOut" => Ok(AnimationName::FadeOut),
            "fadeSwitch" => Ok(AnimationName::FadeSwitch),
            "fadeShadow" => Ok(AnimationName::FadeShadow),
            "fadeDim" => Ok(AnimationName::FadeDim),
            "fadeLayers" => Ok(AnimationName::FadeLayers),
            "fadeLayersIn" => Ok(AnimationName::FadeLayersIn),
            "fadeLayersOut" => Ok(AnimationName::FadeLayersOut),
            "fadePopups" => Ok(AnimationName::FadePopups),
            "fadePopupsIn" => Ok(AnimationName::FadePopupsIn),
            "fadePopupsOut" => Ok(AnimationName::FadePopupsOut),
            "fadeDpms" => Ok(AnimationName::FadeDpms),
            "border" => Ok(AnimationName::Border),
            "borderangle" => Ok(AnimationName::BorderAngle),
            "workspaces" => Ok(AnimationName::Workspaces),
            "workspacesIn" => Ok(AnimationName::WorkspacesIn),
            "workspacesOut" => Ok(AnimationName::WorkspacesOut),
            "specialWorkspace" => Ok(AnimationName::SpecialWorkspace),
            "specialWorkspaceIn" => Ok(AnimationName::SpecialWorkspaceIn),
            "specialWorkspaceOut" => Ok(AnimationName::SpecialWorkspaceOut),
            "zoomFactor" => Ok(AnimationName::ZoomFactor),
            "monitorAdded" => Ok(AnimationName::MonitorAdded),
            _ => Ok(AnimationName::Global),
        }
    }
}

impl Display for AnimationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimationName::Global => write!(f, "global"),
            AnimationName::Windows => write!(f, "windows"),
            AnimationName::WindowsIn => write!(f, "windowsIn"),
            AnimationName::WindowsOut => write!(f, "windowsOut"),
            AnimationName::WindowsMove => write!(f, "windowsMove"),
            AnimationName::Layers => write!(f, "layers"),
            AnimationName::LayersIn => write!(f, "layersIn"),
            AnimationName::LayersOut => write!(f, "layersOut"),
            AnimationName::Fade => write!(f, "fade"),
            AnimationName::FadeIn => write!(f, "fadeIn"),
            AnimationName::FadeOut => write!(f, "fadeOut"),
            AnimationName::FadeSwitch => write!(f, "fadeSwitch"),
            AnimationName::FadeShadow => write!(f, "fadeShadow"),
            AnimationName::FadeDim => write!(f, "fadeDim"),
            AnimationName::FadeLayers => write!(f, "fadeLayers"),
            AnimationName::FadeLayersIn => write!(f, "fadeLayersIn"),
            AnimationName::FadeLayersOut => write!(f, "fadeLayersOut"),
            AnimationName::FadePopups => write!(f, "fadePopups"),
            AnimationName::FadePopupsIn => write!(f, "fadePopupsIn"),
            AnimationName::FadePopupsOut => write!(f, "fadePopupsOut"),
            AnimationName::FadeDpms => write!(f, "fadeDpms"),
            AnimationName::Border => write!(f, "border"),
            AnimationName::BorderAngle => write!(f, "borderangle"),
            AnimationName::Workspaces => write!(f, "workspaces"),
            AnimationName::WorkspacesIn => write!(f, "workspacesIn"),
            AnimationName::WorkspacesOut => write!(f, "workspacesOut"),
            AnimationName::SpecialWorkspace => write!(f, "specialWorkspace"),
            AnimationName::SpecialWorkspaceIn => write!(f, "specialWorkspaceIn"),
            AnimationName::SpecialWorkspaceOut => write!(f, "specialWorkspaceOut"),
            AnimationName::ZoomFactor => write!(f, "zoomFactor"),
            AnimationName::MonitorAdded => write!(f, "monitorAdded"),
        }
    }
}

impl AnimationName {
    pub fn get_list() -> [&'static str; 31] {
        [
            "global",
            "windows",
            "windowsIn",
            "windowsOut",
            "windowsMove",
            "layers",
            "layersIn",
            "layersOut",
            "fade",
            "fadeIn",
            "fadeOut",
            "fadeSwitch",
            "fadeShadow",
            "fadeDim",
            "fadeLayers",
            "fadeLayersIn",
            "fadeLayersOut",
            "fadePopups",
            "fadePopupsIn",
            "fadePopupsOut",
            "fadeDpms",
            "border",
            "borderangle",
            "workspaces",
            "workspacesIn",
            "workspacesOut",
            "specialWorkspace",
            "specialWorkspaceIn",
            "specialWorkspaceOut",
            "zoomFactor",
            "monitorAdded",
        ]
    }

    pub fn get_fancy_list() -> [String; 31] {
        [
            t!("global").to_string(),
            t!("windows").to_string(),
            t!("windows_in").to_string(),
            t!("windows_out").to_string(),
            t!("windows_move").to_string(),
            t!("layers").to_string(),
            t!("layers_in").to_string(),
            t!("layers_out").to_string(),
            t!("fade").to_string(),
            t!("fade_in").to_string(),
            t!("fade_out").to_string(),
            t!("fade_switch").to_string(),
            t!("fade_shadow").to_string(),
            t!("fade_dim").to_string(),
            t!("fade_layers").to_string(),
            t!("fade_layers_in").to_string(),
            t!("fade_layers_out").to_string(),
            t!("fade_popups").to_string(),
            t!("fade_popups_in").to_string(),
            t!("fade_popups_out").to_string(),
            t!("fade_dpms").to_string(),
            t!("border").to_string(),
            t!("borderangle").to_string(),
            t!("workspaces").to_string(),
            t!("workspaces_in").to_string(),
            t!("workspaces_out").to_string(),
            t!("special_workspace").to_string(),
            t!("special_workspace_in").to_string(),
            t!("special_workspace_out").to_string(),
            t!("zoom_factor").to_string(),
            t!("monitor_added").to_string(),
        ]
    }

    pub fn get_fancy_available_styles(&self) -> Option<Vec<String>> {
        match self {
            AnimationName::Windows | AnimationName::WindowsIn | AnimationName::WindowsOut => {
                Some(vec![
                    t!("none").to_string(),
                    t!("slide").to_string(),
                    t!("slide_with_side").to_string(),
                    t!("popin").to_string(),
                    t!("popin_with_percent").to_string(),
                    t!("gnomed").to_string(),
                ])
            }
            AnimationName::Layers | AnimationName::LayersIn | AnimationName::LayersOut => {
                Some(vec![
                    t!("none").to_string(),
                    t!("slide").to_string(),
                    t!("slide_with_side").to_string(),
                    t!("popin").to_string(),
                    t!("fade").to_string(),
                ])
            }
            AnimationName::BorderAngle => Some(vec![
                t!("none").to_string(),
                t!("once").to_string(),
                t!("loop").to_string(),
            ]),
            AnimationName::Workspaces
            | AnimationName::WorkspacesIn
            | AnimationName::WorkspacesOut
            | AnimationName::SpecialWorkspace
            | AnimationName::SpecialWorkspaceIn
            | AnimationName::SpecialWorkspaceOut => Some(vec![
                t!("none").to_string(),
                t!("slide").to_string(),
                t!("slide_with_percent").to_string(),
                t!("slidevert").to_string(),
                t!("slidevert_with_percent").to_string(),
                t!("fade").to_string(),
                t!("slidefade").to_string(),
                t!("slidefade_with_percent").to_string(),
                t!("slidefadevert").to_string(),
                t!("slidefade_with_percent").to_string(),
            ]),
            _ => None,
        }
    }

    pub fn get_available_styles(&self) -> Option<Vec<AnimationStyle>> {
        match self {
            AnimationName::Windows | AnimationName::WindowsIn | AnimationName::WindowsOut => {
                Some(vec![
                    AnimationStyle::None,
                    AnimationStyle::Slide,
                    AnimationStyle::SlideSide(Side::Left),
                    AnimationStyle::Popin,
                    AnimationStyle::PopinPercent(50.0),
                    AnimationStyle::Gnomed,
                ])
            }
            AnimationName::Layers | AnimationName::LayersIn | AnimationName::LayersOut => {
                Some(vec![
                    AnimationStyle::None,
                    AnimationStyle::Slide,
                    AnimationStyle::SlideSide(Side::Left),
                    AnimationStyle::Popin,
                    AnimationStyle::Fade,
                ])
            }
            AnimationName::BorderAngle => Some(vec![
                AnimationStyle::None,
                AnimationStyle::Once,
                AnimationStyle::Loop,
            ]),
            AnimationName::Workspaces
            | AnimationName::WorkspacesIn
            | AnimationName::WorkspacesOut
            | AnimationName::SpecialWorkspace
            | AnimationName::SpecialWorkspaceIn
            | AnimationName::SpecialWorkspaceOut => Some(vec![
                AnimationStyle::None,
                AnimationStyle::Slide,
                AnimationStyle::SlidePercent(50.0),
                AnimationStyle::SlideVert,
                AnimationStyle::SlideVertPercent(50.0),
                AnimationStyle::Fade,
                AnimationStyle::SlideFade,
                AnimationStyle::SlideFadePercent(50.0),
                AnimationStyle::SlideFadeVert,
                AnimationStyle::SlideFadePercent(50.0),
            ]),
            _ => None,
        }
    }

    pub fn get_id_of_style(&self, style: AnimationStyle) -> Option<usize> {
        match self {
            AnimationName::Windows | AnimationName::WindowsIn | AnimationName::WindowsOut => {
                match style {
                    AnimationStyle::None => Some(0),
                    AnimationStyle::Slide => Some(1),
                    AnimationStyle::SlideSide(_) => Some(2),
                    AnimationStyle::Popin => Some(3),
                    AnimationStyle::PopinPercent(_) => Some(4),
                    AnimationStyle::Gnomed => Some(5),
                    _ => None,
                }
            }
            AnimationName::Layers | AnimationName::LayersIn | AnimationName::LayersOut => {
                match style {
                    AnimationStyle::None => Some(0),
                    AnimationStyle::Slide => Some(1),
                    AnimationStyle::SlideSide(_) => Some(2),
                    AnimationStyle::Popin => Some(3),
                    AnimationStyle::Fade => Some(4),
                    _ => None,
                }
            }
            AnimationName::BorderAngle => match style {
                AnimationStyle::None => Some(0),
                AnimationStyle::Once => Some(1),
                AnimationStyle::Loop => Some(2),
                _ => None,
            },
            AnimationName::Workspaces
            | AnimationName::WorkspacesIn
            | AnimationName::WorkspacesOut
            | AnimationName::SpecialWorkspace
            | AnimationName::SpecialWorkspaceIn
            | AnimationName::SpecialWorkspaceOut => match style {
                AnimationStyle::None => Some(0),
                AnimationStyle::Slide => Some(1),
                AnimationStyle::SlidePercent(_) => Some(2),
                AnimationStyle::SlideVert => Some(3),
                AnimationStyle::SlideVertPercent(_) => Some(4),
                AnimationStyle::Fade => Some(5),
                AnimationStyle::SlideFade => Some(6),
                AnimationStyle::SlideFadePercent(_) => Some(7),
                AnimationStyle::SlideFadeVert => Some(8),
                AnimationStyle::SlideFadeVertPercent(_) => Some(9),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

impl FromStr for Side {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_whitespace()
            .next()
            .map_or(Ok(Side::Left), |first| match first {
                "left" => Ok(Side::Left),
                "right" => Ok(Side::Right),
                "top" => Ok(Side::Top),
                "bottom" => Ok(Side::Bottom),
                _ => Err(()),
            })
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Left => write!(f, "left"),
            Side::Right => write!(f, "right"),
            Side::Top => write!(f, "top"),
            Side::Bottom => write!(f, "bottom"),
        }
    }
}

impl Side {
    pub fn get_list() -> [&'static str; 4] {
        ["left", "right", "top", "bottom"]
    }

    pub fn get_fancy_list() -> [String; 4] {
        [
            t!("left").to_string(),
            t!("right").to_string(),
            t!("top").to_string(),
            t!("bottom").to_string(),
        ]
    }

    pub fn get_id(&self) -> usize {
        match self {
            Side::Left => 0,
            Side::Right => 1,
            Side::Top => 2,
            Side::Bottom => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AnimationStyle {
    #[default]
    None,
    Slide,
    SlideSide(Side),
    SlidePercent(f64),
    Popin,
    PopinPercent(f64),
    Gnomed,
    SlideVert,
    SlideVertPercent(f64),
    Fade,
    SlideFade,
    SlideFadePercent(f64),
    SlideFadeVert,
    SlideFadeVertPercent(f64),
    Once,
    Loop,
}

impl FromStr for AnimationStyle {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_whitespace()
            .next()
            .map_or(Ok(AnimationStyle::None), |first| match first {
                "slide" => {
                    let remainder = s.strip_prefix("slide").map(str::trim).unwrap_or("");
                    if remainder.is_empty() {
                        Ok(AnimationStyle::Slide)
                    } else {
                        match Side::from_str(remainder) {
                            Ok(side) => Ok(AnimationStyle::SlideSide(side)),
                            Err(_) => match f64::from_str(remainder.trim_end_matches('%')) {
                                Ok(percent) => Ok(AnimationStyle::SlidePercent(percent)),
                                Err(_) => Ok(AnimationStyle::None),
                            },
                        }
                    }
                }
                "popin" => {
                    let remainder = s.strip_prefix("popin").map(str::trim).unwrap_or("");
                    if remainder.is_empty() {
                        Ok(AnimationStyle::Popin)
                    } else {
                        let percent = remainder.trim_end_matches('%');
                        match percent.parse::<f64>() {
                            Ok(percent) => Ok(AnimationStyle::PopinPercent(percent)),
                            Err(_) => Err(()),
                        }
                    }
                }
                "gnomed" => Ok(AnimationStyle::Gnomed),
                "slidevert" => {
                    let remainder = s.strip_prefix("slidevert").map(str::trim).unwrap_or("");
                    if remainder.is_empty() {
                        Ok(AnimationStyle::SlideVert)
                    } else {
                        let percent = remainder.trim_end_matches('%');
                        match percent.parse::<f64>() {
                            Ok(percent) => Ok(AnimationStyle::SlideVertPercent(percent)),
                            Err(_) => Err(()),
                        }
                    }
                }
                "fade" => Ok(AnimationStyle::Fade),
                "slidefade" => {
                    let remainder = s.strip_prefix("slidefade").map(str::trim).unwrap_or("");
                    if remainder.is_empty() {
                        Ok(AnimationStyle::SlideFade)
                    } else {
                        let percent = remainder.trim_end_matches('%');
                        match percent.parse::<f64>() {
                            Ok(percent) => Ok(AnimationStyle::SlideFadePercent(percent)),
                            Err(_) => Err(()),
                        }
                    }
                }
                "slidefadevert" => {
                    let remainder = s.strip_prefix("slidefadevert").map(str::trim).unwrap_or("");
                    if remainder.is_empty() {
                        Ok(AnimationStyle::SlideFadeVert)
                    } else {
                        let percent = remainder.trim_end_matches('%');
                        match percent.parse::<f64>() {
                            Ok(percent) => Ok(AnimationStyle::SlideFadeVertPercent(percent)),
                            Err(_) => Err(()),
                        }
                    }
                }
                "once" => Ok(AnimationStyle::Once),
                "loop" => Ok(AnimationStyle::Loop),
                _ => Err(()),
            })
    }
}

impl Display for AnimationStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimationStyle::None => write!(f, ""),
            AnimationStyle::Slide => write!(f, "slide"),
            AnimationStyle::SlideSide(side) => write!(f, "slide {}", side),
            AnimationStyle::SlidePercent(percent) => write!(f, "slide {}%", percent),
            AnimationStyle::Popin => write!(f, "popin"),
            AnimationStyle::PopinPercent(percent) => write!(f, "popin {}%", percent),
            AnimationStyle::Gnomed => write!(f, "gnomed"),
            AnimationStyle::SlideVert => write!(f, "slidevert"),
            AnimationStyle::SlideVertPercent(percent) => write!(f, "slidevert {}%", percent),
            AnimationStyle::Fade => write!(f, "fade"),
            AnimationStyle::SlideFade => write!(f, "slidefade"),
            AnimationStyle::SlideFadePercent(percent) => write!(f, "slidefade {}%", percent),
            AnimationStyle::SlideFadeVert => write!(f, "slidefadevert"),
            AnimationStyle::SlideFadeVertPercent(percent) => {
                write!(f, "slidefadevert {}%", percent)
            }
            AnimationStyle::Once => write!(f, "once"),
            AnimationStyle::Loop => write!(f, "loop"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub name: AnimationName,
    pub enabled: bool,
    pub speed: f64,
    pub curve: String,
    pub style: AnimationStyle,
}

impl FromStr for Animation {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
        if values.is_empty() | values.first().is_none_or(|v| v.is_empty()) {
            return Err(());
        }

        let name = AnimationName::from_str(&values[0]).unwrap_or_default();
        let enabled = values.get(1).is_none_or(|v| parse_bool(v).unwrap_or(true));
        let speed = values
            .get(2)
            .map_or(10.0, |v| v.parse::<f64>().unwrap_or(10.0));
        let curve = values.get(3).map_or("default".to_string(), |v| v.clone());
        let style = values.get(4).map_or(AnimationStyle::None, |v| {
            AnimationStyle::from_str(v).unwrap_or_default()
        });

        Ok(Animation {
            name,
            enabled,
            speed,
            curve,
            style,
        })
    }
}

impl Display for Animation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {:.1}, {}",
            self.name,
            if self.enabled { 1 } else { 0 },
            self.speed,
            self.curve
        )?;
        if !matches!(self.style, AnimationStyle::None) {
            write!(f, ", {}", self.style)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BezierCurve {
    pub name: String,
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl FromStr for BezierCurve {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
        if values.len() < 5 {
            return Err(());
        }

        let name = values[0].clone();
        let x0 = values[1].parse::<f64>().unwrap_or(0.333);
        let y0 = values[2].parse::<f64>().unwrap_or(0.333);
        let x1 = values[3].parse::<f64>().unwrap_or(0.667);
        let y1 = values[4].parse::<f64>().unwrap_or(0.667);

        Ok(BezierCurve {
            name,
            x0,
            y0,
            x1,
            y1,
        })
    }
}

impl Display for BezierCurve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {:.3}, {:.3}, {:.3}, {:.3}",
            self.name, self.x0, self.y0, self.x1, self.y1
        )
    }
}

pub fn parse_animation(input: &str) -> Animation {
    Animation::from_str(input).unwrap_or(Animation {
        name: AnimationName::Global,
        enabled: true,
        speed: 10.0,
        curve: "default".to_string(),
        style: AnimationStyle::None,
    })
}

pub fn parse_bezier(input: &str) -> BezierCurve {
    BezierCurve::from_str(input).unwrap_or(BezierCurve {
        name: "default".to_string(),
        x0: 0.333,
        y0: 0.333,
        x1: 0.667,
        y1: 0.667,
    })
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_int(value: &str) -> Option<i32> {
    value.parse::<i32>().ok()
}

pub const CONFIG_PATH: &str = ".config/hypr/hyprland.conf";
pub const HYPRVIZ_CONFIG_PATH: &str = ".config/hypr/hyprviz.conf";
pub const HYPRVIZ_PROFILES_PATH: &str = ".config/hypr/hyprviz/";
pub const BACKUP_SUFFIX: &str = "-bak";

/// 9007199254740992.0
pub const MAX_SAFE_INTEGER_F64: f64 = (1u64 << 53) as f64; // 2^53
/// -9007199254740992.0
pub const MIN_SAFE_INTEGER_F64: f64 = -MAX_SAFE_INTEGER_F64; // -2^53
/// 140737488355328
pub const MAX_SAFE_STEP_0_01_F64: f64 = (1u64 << 47) as f64; // 2^47
