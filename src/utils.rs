use rust_i18n::t;
use serde_json::Value;
use std::{
    borrow::Cow,
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
    sync::{LazyLock, OnceLock},
};
use strum::{EnumDiscriminants, EnumIter, IntoEnumIterator};

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

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(MonitorSelectorDiscriminant))]
pub enum MonitorSelector {
    #[default]
    All,
    Name(String),
    Description(String),
}

impl HasDiscriminant for MonitorSelector {
    type Discriminant = MonitorSelectorDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::All => Self::All,
            Self::Discriminant::Name => Self::Name("".to_string()),
            Self::Discriminant::Description => Self::Description("".to_string()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::All => Self::All,
            Self::Discriminant::Name => Self::Name(str.to_string()),
            Self::Discriminant::Description => Self::Description(str.to_string()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Self::All => None,
            Self::Name(name) => Some(name.to_string()),
            Self::Description(desc) => Some(desc.to_string()),
        }
    }
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
            t!("utils.auto").to_string(),
            t!("utils.auto_right").to_string(),
            t!("utils.auto_left").to_string(),
            t!("utils.auto_up").to_string(),
            t!("utils.auto_down").to_string(),
            t!("utils.auto_center_right").to_string(),
            t!("utils.auto_center_left").to_string(),
            t!("utils.auto_center_up").to_string(),
            t!("utils.auto_center_down").to_string(),
            t!("utils.coordinates").to_string(),
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
        match s.trim().to_lowercase().as_str() {
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
        [t!("utils.auto").to_string(), t!("utils.manual").to_string()]
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
        match s.trim().to_lowercase().as_str() {
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

#[derive(EnumIter)]
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
        match s.trim().to_lowercase().as_str() {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}

impl Default for Range {
    fn default() -> Self {
        Range { start: 1, end: 1 }
    }
}

impl FromStr for Range {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start, end) = s.split_once('-').unwrap_or((s, "1"));
        Ok(Range {
            start: start.parse().unwrap_or(1),
            end: end.parse().unwrap_or(1),
        })
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WorkspaceSelectorNamedDiscriminant))]
pub enum WorkspaceSelectorNamed {
    IsNamed(bool),
    Starts(String),
    Ends(String),
}

impl HasDiscriminant for WorkspaceSelectorNamed {
    type Discriminant = WorkspaceSelectorNamedDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::IsNamed => Self::IsNamed(false),
            Self::Discriminant::Starts => Self::Starts("".to_string()),
            Self::Discriminant::Ends => Self::Ends("".to_string()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::IsNamed => Self::IsNamed(parse_bool(str).unwrap_or(false)),
            Self::Discriminant::Starts => Self::Starts(str.to_string()),
            Self::Discriminant::Ends => Self::Ends(str.to_string()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Self::IsNamed(is_named) => Some(format!("{}", is_named)),
            Self::Starts(prefix) => Some(prefix.to_string()),
            Self::Ends(suffix) => Some(suffix.to_string()),
        }
    }
}

impl Default for WorkspaceSelectorNamed {
    fn default() -> Self {
        WorkspaceSelectorNamed::IsNamed(false)
    }
}

impl FromStr for WorkspaceSelectorNamed {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if let Some(starts_with) = s.strip_prefix("s:") {
            Ok(WorkspaceSelectorNamed::Starts(starts_with.to_string()))
        } else if let Some(ends_with) = s.strip_prefix("e:") {
            Ok(WorkspaceSelectorNamed::Ends(ends_with.to_string()))
        } else {
            Ok(WorkspaceSelectorNamed::IsNamed(
                parse_bool(s).unwrap_or(false),
            ))
        }
    }
}

impl Display for WorkspaceSelectorNamed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceSelectorNamed::IsNamed(is_named) => write!(f, "{}", is_named),
            WorkspaceSelectorNamed::Starts(prefix) => write!(f, "s:{}", prefix),
            WorkspaceSelectorNamed::Ends(suffix) => write!(f, "e:{}", suffix),
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorkspaceSelectorWindowCountFlags {
    pub tiled: bool,
    pub floating: bool,
    pub groups: bool,
    pub visible: bool,
    pub pinned: bool,
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

        for c in s.trim().chars() {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WorkspaceSelectorWindowCountDiscriminant))]
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

impl HasDiscriminant for WorkspaceSelectorWindowCount {
    type Discriminant = WorkspaceSelectorWindowCountDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Range => Self::Range {
                flags: WorkspaceSelectorWindowCountFlags::default(),
                range_start: 0,
                range_end: 0,
            },
            Self::Discriminant::Single => Self::Single {
                flags: WorkspaceSelectorWindowCountFlags::default(),
                count: 0,
            },
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Range => match Self::from_str(str).unwrap_or_default() {
                Self::Range {
                    flags,
                    range_start,
                    range_end,
                } => Self::Range {
                    flags,
                    range_start,
                    range_end,
                },
                Self::Single { flags, count } => Self::Range {
                    flags,
                    range_start: count,
                    range_end: count,
                },
            },
            Self::Discriminant::Single => match Self::from_str(str).unwrap_or_default() {
                Self::Range {
                    flags,
                    range_start,
                    range_end: _,
                } => Self::Single {
                    flags,
                    count: range_start,
                },
                Self::Single { flags, count } => Self::Single { flags, count },
            },
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Self::Range {
                flags,
                range_start,
                range_end,
            } => Some(format!("{}{}-{}", flags, range_start, range_end)),
            Self::Single { flags, count } => Some(format!("{}{}", flags, count)),
        }
    }

    fn custom_split(discriminant: Self::Discriminant) -> Option<fn(&str) -> Vec<&str>> {
        match discriminant {
            Self::Discriminant::Range => Some(|s: &str| {
                let s = s.trim();
                let (flags_str, count_str) = if let Some(pos) = s.find(|c: char| !c.is_alphabetic())
                {
                    (&s[..pos], &s[pos..])
                } else {
                    (s, "")
                };
                let count_str = count_str.trim().trim_matches('-');
                let (start_str, end_str) =
                    count_str.split_once('-').unwrap_or((count_str, count_str));

                vec![flags_str, start_str, end_str]
            }),
            Self::Discriminant::Single => Some(|s: &str| {
                let s = s.trim();
                let (flags_str, count_str) = if let Some(pos) = s.find(|c: char| !c.is_alphabetic())
                {
                    (&s[..pos], &s[pos..])
                } else {
                    (s, "")
                };
                let count_str = count_str.trim().trim_matches('-');

                vec![flags_str, count_str]
            }),
        }
    }
}

impl Default for WorkspaceSelectorWindowCount {
    fn default() -> Self {
        WorkspaceSelectorWindowCount::Single {
            flags: WorkspaceSelectorWindowCountFlags::default(),
            count: 0,
        }
    }
}

impl FromStr for WorkspaceSelectorWindowCount {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        let (flags_str, count_str) = if let Some(pos) = s.find(|c: char| !c.is_alphabetic()) {
            (&s[..pos], &s[pos..])
        } else {
            (s, "")
        };

        let flags = WorkspaceSelectorWindowCountFlags::from_str(flags_str).unwrap_or_default();

        let count_str = count_str.trim().trim_matches('-');

        if count_str.contains('-') {
            if let Some((start_str, end_str)) = count_str.split_once('-')
                && let (Ok(start), Ok(end)) = (
                    start_str.trim().parse::<u32>(),
                    end_str.trim().parse::<u32>(),
                )
            {
                Ok(WorkspaceSelectorWindowCount::Range {
                    flags,
                    range_start: start,
                    range_end: end,
                })
            } else {
                Err(())
            }
        } else if !count_str.is_empty()
            && let Ok(count) = count_str.parse::<u32>()
        {
            Ok(WorkspaceSelectorWindowCount::Single { flags, count })
        } else {
            Err(())
        }
    }
}

impl Display for WorkspaceSelectorWindowCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceSelectorWindowCount::Range {
                flags,
                range_start,
                range_end,
            } => {
                write!(f, "{}{}-{}", flags, range_start, range_end)
            }
            WorkspaceSelectorWindowCount::Single { flags, count } => {
                write!(f, "{}{}", flags, count)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter, Default)]
pub enum WorkspaceSelectorFullscreen {
    NoFullscreen,
    #[default]
    Fullscreen,
    Maximized,
    FullscreenWithoutFullscreenStateSentToTheWindow,
}

impl WorkspaceSelectorFullscreen {
    pub fn from_num(num: i8) -> Self {
        match num {
            -1 => WorkspaceSelectorFullscreen::NoFullscreen,
            0 => WorkspaceSelectorFullscreen::Fullscreen,
            1 => WorkspaceSelectorFullscreen::Maximized,
            2 => WorkspaceSelectorFullscreen::FullscreenWithoutFullscreenStateSentToTheWindow,
            _ => WorkspaceSelectorFullscreen::default(),
        }
    }

    pub fn to_num(self) -> i8 {
        match self {
            WorkspaceSelectorFullscreen::NoFullscreen => -1,
            WorkspaceSelectorFullscreen::Fullscreen => 0,
            WorkspaceSelectorFullscreen::Maximized => 1,
            WorkspaceSelectorFullscreen::FullscreenWithoutFullscreenStateSentToTheWindow => 2,
        }
    }
}

impl FromStr for WorkspaceSelectorFullscreen {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().parse::<i8>() {
            Ok(num) => Ok(WorkspaceSelectorFullscreen::from_num(num)),
            Err(_) => Err(()),
        }
    }
}

impl Display for WorkspaceSelectorFullscreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_num())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WorkspaceSelectorDiscriminant))]
pub enum WorkspaceSelector {
    #[default]
    None,
    Range(Range),
    Special(bool),
    Named(WorkspaceSelectorNamed),
    Monitor(MonitorSelector),
    WindowCount(WorkspaceSelectorWindowCount),
    Fullscreen(WorkspaceSelectorFullscreen),
}

impl WorkspaceSelector {
    pub fn get_fancy_list() -> [String; 7] {
        [
            t!("utils.none").to_string(),
            t!("utils.range").to_string(),
            t!("utils.special").to_string(),
            t!("utils.named").to_string(),
            t!("utils.monitor").to_string(),
            t!("utils.window_count").to_string(),
            t!("utils.fullscreen").to_string(),
        ]
    }
}

impl HasDiscriminant for WorkspaceSelector {
    type Discriminant = WorkspaceSelectorDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::None => Self::None,
            Self::Discriminant::Range => Self::Range(Range::default()),
            Self::Discriminant::Special => Self::Special(false),
            Self::Discriminant::Named => Self::Named(WorkspaceSelectorNamed::default()),
            Self::Discriminant::Monitor => Self::Monitor(MonitorSelector::default()),
            Self::Discriminant::WindowCount => {
                Self::WindowCount(WorkspaceSelectorWindowCount::default())
            }
            Self::Discriminant::Fullscreen => {
                Self::Fullscreen(WorkspaceSelectorFullscreen::default())
            }
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::None => Self::None,
            Self::Discriminant::Range => Self::Range(Range::from_str(str).unwrap_or_default()),
            Self::Discriminant::Special => Self::Special(str.parse().unwrap_or_default()),
            Self::Discriminant::Named => {
                Self::Named(WorkspaceSelectorNamed::from_str(str).unwrap_or_default())
            }
            Self::Discriminant::Monitor => {
                Self::Monitor(MonitorSelector::from_str(str).unwrap_or_default())
            }
            Self::Discriminant::WindowCount => {
                Self::WindowCount(WorkspaceSelectorWindowCount::from_str(str).unwrap_or_default())
            }
            Self::Discriminant::Fullscreen => {
                Self::Fullscreen(WorkspaceSelectorFullscreen::from_str(str).unwrap_or_default())
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Self::None => None,
            Self::Range(Range { start, end }) => Some(format!("[{}-{}]", start, end)),
            Self::Special(is_special) => Some(format!("[{}]", is_special)),
            Self::Named(named) => Some(format!("[{}]", named)),
            Self::Monitor(monitor) => Some(format!("[{}]", monitor)),
            Self::WindowCount(window_count) => Some(format!("[{}]", window_count)),
            Self::Fullscreen(state) => Some(format!("[{}]", state)),
        }
    }
}

impl FromStr for WorkspaceSelector {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        match parse_single_selector(s) {
            Some((selector, _)) => Ok(selector),
            None => Err(()),
        }
    }
}

impl Display for WorkspaceSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceSelector::None => write!(f, ""),
            WorkspaceSelector::Range(range) => write!(f, "r[{}]", range),
            WorkspaceSelector::Special(is_special) => {
                write!(f, "s[{}]", is_special)
            }
            WorkspaceSelector::Named(named) => {
                write!(f, "n[{}]", named)
            }
            WorkspaceSelector::Monitor(monitor) => write!(f, "m[{}]", monitor),
            WorkspaceSelector::WindowCount(window_count) => {
                write!(f, "w[{}]", window_count)
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
            t!("utils.named").to_string(),
            t!("utils.special").to_string(),
            t!("utils.numbered").to_string(),
            t!("utils.selector").to_string(),
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

#[derive(Debug, Default, EnumIter)]
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
        match s.trim() {
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

pub fn parse_workspace_selector(input: &str) -> Vec<WorkspaceSelector> {
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

    selectors
}

pub fn parse_workspace_type(input: &str) -> WorkspaceType {
    if let Some(name) = input.strip_prefix("name:") {
        WorkspaceType::Named(name.to_string())
    } else if let Some(name) = input.strip_prefix("special:") {
        WorkspaceType::Special(name.to_string())
    } else if let Ok(num) = input.parse::<u32>() {
        WorkspaceType::Numbered(num)
    } else {
        WorkspaceType::Selector(parse_workspace_selector(input))
    }
}

pub fn parse_single_selector(input: &str) -> Option<(WorkspaceSelector, &str)> {
    if input.starts_with("r[") {
        if let Some(end_idx) = find_matching_bracket(input, "r[", ']') {
            let content = &input[2..end_idx];
            let rest = &input[end_idx + 1..];

            let selector = match Range::from_str(content) {
                Ok(range) => WorkspaceSelector::Range(range),
                Err(_) => return None,
            };

            return Some((selector, rest));
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

            let selector = match WorkspaceSelectorNamed::from_str(content) {
                Ok(selector) => WorkspaceSelector::Named(selector),
                Err(_) => return None,
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

            let selector = match WorkspaceSelectorWindowCount::from_str(content) {
                Ok(selector) => WorkspaceSelector::WindowCount(selector),
                Err(_) => return None,
            };

            return Some((selector, rest));
        }
    } else if input.starts_with("f[")
        && let Some(end_idx) = find_matching_bracket(input, "f[", ']')
    {
        let content = &input[2..end_idx];
        let rest = &input[end_idx + 1..];

        if let Ok(state) = content.parse::<WorkspaceSelectorFullscreen>() {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter)]
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
        match s.trim() {
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
            t!("utils.global").to_string(),
            t!("utils.windows").to_string(),
            t!("utils.windows_in").to_string(),
            t!("utils.windows_out").to_string(),
            t!("utils.windows_move").to_string(),
            t!("utils.layers").to_string(),
            t!("utils.layers_in").to_string(),
            t!("utils.layers_out").to_string(),
            t!("utils.fade").to_string(),
            t!("utils.fade_in").to_string(),
            t!("utils.fade_out").to_string(),
            t!("utils.fade_switch").to_string(),
            t!("utils.fade_shadow").to_string(),
            t!("utils.fade_dim").to_string(),
            t!("utils.fade_layers").to_string(),
            t!("utils.fade_layers_in").to_string(),
            t!("utils.fade_layers_out").to_string(),
            t!("utils.fade_popups").to_string(),
            t!("utils.fade_popups_in").to_string(),
            t!("utils.fade_popups_out").to_string(),
            t!("utils.fade_dpms").to_string(),
            t!("utils.border").to_string(),
            t!("utils.borderangle").to_string(),
            t!("utils.workspaces").to_string(),
            t!("utils.workspaces_in").to_string(),
            t!("utils.workspaces_out").to_string(),
            t!("utils.special_workspace").to_string(),
            t!("utils.special_workspace_in").to_string(),
            t!("utils.special_workspace_out").to_string(),
            t!("utils.zoom_factor").to_string(),
            t!("utils.monitor_added").to_string(),
        ]
    }

    pub fn get_fancy_available_styles(&self) -> Option<Vec<String>> {
        match self {
            AnimationName::Windows | AnimationName::WindowsIn | AnimationName::WindowsOut => {
                Some(vec![
                    t!("utils.none").to_string(),
                    t!("utils.slide").to_string(),
                    t!("utils.slide_with_side").to_string(),
                    t!("utils.popin").to_string(),
                    t!("utils.popin_with_percent").to_string(),
                    t!("utils.gnomed").to_string(),
                ])
            }
            AnimationName::Layers | AnimationName::LayersIn | AnimationName::LayersOut => {
                Some(vec![
                    t!("utils.none").to_string(),
                    t!("utils.slide").to_string(),
                    t!("utils.slide_with_side").to_string(),
                    t!("utils.popin").to_string(),
                    t!("utils.fade").to_string(),
                ])
            }
            AnimationName::BorderAngle => Some(vec![
                t!("utils.none").to_string(),
                t!("utils.once").to_string(),
                t!("utils.loop").to_string(),
            ]),
            AnimationName::Workspaces
            | AnimationName::WorkspacesIn
            | AnimationName::WorkspacesOut
            | AnimationName::SpecialWorkspace
            | AnimationName::SpecialWorkspaceIn
            | AnimationName::SpecialWorkspaceOut => Some(vec![
                t!("utils.none").to_string(),
                t!("utils.slide").to_string(),
                t!("utils.slide_with_percent").to_string(),
                t!("utils.slidevert").to_string(),
                t!("utils.slidevert_with_percent").to_string(),
                t!("utils.fade").to_string(),
                t!("utils.slidefade").to_string(),
                t!("utils.slidefade_with_percent").to_string(),
                t!("utils.slidefadevert").to_string(),
                t!("utils.slidefade_with_percent").to_string(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Default)]
pub enum Side {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
}

impl FromStr for Side {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "left" => Ok(Side::Left),
            "right" => Ok(Side::Right),
            "top" => Ok(Side::Top),
            "bottom" => Ok(Side::Bottom),
            _ => Err(()),
        }
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
            t!("utils.left").to_string(),
            t!("utils.right").to_string(),
            t!("utils.top").to_string(),
            t!("utils.bottom").to_string(),
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

#[derive(Debug, Clone, Copy, PartialEq, Default, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(AnimationStyleDiscriminant))]
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

impl HasDiscriminant for AnimationStyle {
    type Discriminant = AnimationStyleDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::None => Self::None,
            Self::Discriminant::Slide => Self::Slide,
            Self::Discriminant::SlideSide => Self::SlideSide(Side::default()),
            Self::Discriminant::SlidePercent => Self::SlidePercent(0.0),
            Self::Discriminant::Popin => Self::Popin,
            Self::Discriminant::PopinPercent => Self::PopinPercent(0.0),
            Self::Discriminant::Gnomed => Self::Gnomed,
            Self::Discriminant::SlideVert => Self::SlideVert,
            Self::Discriminant::SlideVertPercent => Self::SlideVertPercent(0.0),
            Self::Discriminant::Fade => Self::Fade,
            Self::Discriminant::SlideFade => Self::SlideFade,
            Self::Discriminant::SlideFadePercent => Self::SlideFadePercent(0.0),
            Self::Discriminant::SlideFadeVert => Self::SlideFadeVert,
            Self::Discriminant::SlideFadeVertPercent => Self::SlideFadeVertPercent(0.0),
            Self::Discriminant::Once => Self::Once,
            Self::Discriminant::Loop => Self::Loop,
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        let s = str.trim();
        match discriminant {
            Self::Discriminant::None => Self::None,
            Self::Discriminant::Slide => Self::Slide,
            Self::Discriminant::SlideSide => Self::SlideSide(Side::from_str(s).unwrap_or_default()),
            Self::Discriminant::SlidePercent => {
                Self::SlidePercent(s.trim_end_matches('%').parse().unwrap_or_default())
            }
            Self::Discriminant::Popin => Self::Popin,
            Self::Discriminant::PopinPercent => {
                Self::PopinPercent(s.trim_end_matches('%').parse().unwrap_or_default())
            }
            Self::Discriminant::Gnomed => Self::Gnomed,
            Self::Discriminant::SlideVert => Self::SlideVert,
            Self::Discriminant::SlideVertPercent => {
                Self::SlideVertPercent(s.trim_end_matches('%').parse().unwrap_or_default())
            }
            Self::Discriminant::Fade => Self::Fade,
            Self::Discriminant::SlideFade => Self::SlideFade,
            Self::Discriminant::SlideFadePercent => {
                Self::SlideFadePercent(s.trim_end_matches('%').parse().unwrap_or_default())
            }
            Self::Discriminant::SlideFadeVert => Self::SlideFadeVert,
            Self::Discriminant::SlideFadeVertPercent => {
                Self::SlideFadeVertPercent(s.trim_end_matches('%').parse().unwrap_or_default())
            }
            Self::Discriminant::Once => Self::Once,
            Self::Discriminant::Loop => Self::Loop,
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            AnimationStyle::None => None,
            AnimationStyle::Slide => None,
            AnimationStyle::SlideSide(side) => Some(side.to_string()),
            AnimationStyle::SlidePercent(percent) => Some(percent.to_string()),
            AnimationStyle::Popin => None,
            AnimationStyle::PopinPercent(percent) => Some(percent.to_string()),
            AnimationStyle::Gnomed => None,
            AnimationStyle::SlideVert => None,
            AnimationStyle::SlideVertPercent(percent) => Some(percent.to_string()),
            AnimationStyle::Fade => None,
            AnimationStyle::SlideFade => None,
            AnimationStyle::SlideFadePercent(percent) => Some(percent.to_string()),
            AnimationStyle::SlideFadeVert => None,
            AnimationStyle::SlideFadeVertPercent(percent) => Some(percent.to_string()),
            AnimationStyle::Once => None,
            AnimationStyle::Loop => None,
        }
    }
}

impl FromStr for AnimationStyle {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
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
        let s = s.trim();
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
        let s = s.trim();
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Default)]
pub enum Modifier {
    Shift,
    Caps,
    Ctrl,
    Alt,
    Mod2,
    Mod3,
    #[default]
    Super,
    Mod5,
}

impl FromStr for Modifier {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "SHIFT" => Ok(Modifier::Shift),
            "CAPS" => Ok(Modifier::Caps),
            "CTRL" => Ok(Modifier::Ctrl),
            "ALT" => Ok(Modifier::Alt),
            "MOD2" => Ok(Modifier::Mod2),
            "MOD3" => Ok(Modifier::Mod3),
            "SUPER" => Ok(Modifier::Super),
            "MOD5" => Ok(Modifier::Mod5),
            _ => Err(()),
        }
    }
}

impl Display for Modifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Modifier::Shift => write!(f, "SHIFT"),
            Modifier::Caps => write!(f, "CAPS"),
            Modifier::Ctrl => write!(f, "CTRL"),
            Modifier::Alt => write!(f, "ALT"),
            Modifier::Mod2 => write!(f, "MOD2"),
            Modifier::Mod3 => write!(f, "MOD3"),
            Modifier::Super => write!(f, "SUPER"),
            Modifier::Mod5 => write!(f, "MOD5"),
        }
    }
}

pub fn parse_modifiers(s: &str) -> HashSet<Modifier> {
    let mut mods = HashSet::new();

    let s_upper = s.trim_start().to_uppercase();
    if s_upper.contains("SUPER")
        || s_upper.contains("WIN")
        || s_upper.contains("LOGO")
        || s_upper.contains("MOD4")
    {
        mods.insert(Modifier::Super);
    }
    if s_upper.contains("SHIFT") {
        mods.insert(Modifier::Shift);
    }
    if s_upper.contains("CTRL") || s_upper.contains("CONTROL") {
        mods.insert(Modifier::Ctrl);
    }
    if s_upper.contains("ALT") {
        mods.insert(Modifier::Alt);
    }
    if s_upper.contains("CAPS") {
        mods.insert(Modifier::Caps);
    }
    if s_upper.contains("MOD2") {
        mods.insert(Modifier::Mod2);
    }
    if s_upper.contains("MOD3") {
        mods.insert(Modifier::Mod3);
    }
    if s_upper.contains("MOD5") {
        mods.insert(Modifier::Mod5);
    }

    mods
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter)]
pub enum BindFlagsEnum {
    Locked,
    Release,
    Click,
    Drag,
    LongPress,
    Repeat,
    NonConsuming,
    Mouse,
    Transparent,
    IgnoreMods,
    Separate,
    HasDescription,
    Bypass,
}

impl BindFlagsEnum {
    pub fn get_all() -> [BindFlagsEnum; 13] {
        [
            BindFlagsEnum::Locked,
            BindFlagsEnum::Release,
            BindFlagsEnum::Click,
            BindFlagsEnum::Drag,
            BindFlagsEnum::LongPress,
            BindFlagsEnum::Repeat,
            BindFlagsEnum::NonConsuming,
            BindFlagsEnum::Mouse,
            BindFlagsEnum::Transparent,
            BindFlagsEnum::IgnoreMods,
            BindFlagsEnum::Separate,
            BindFlagsEnum::HasDescription,
            BindFlagsEnum::Bypass,
        ]
    }

    pub fn to_fancy_string(&self) -> String {
        match self {
            BindFlagsEnum::Locked => t!("utils.locked").to_string(),
            BindFlagsEnum::Release => t!("utils.release").to_string(),
            BindFlagsEnum::Click => t!("utils.click").to_string(),
            BindFlagsEnum::Drag => t!("utils.drag").to_string(),
            BindFlagsEnum::LongPress => t!("utils.long_press").to_string(),
            BindFlagsEnum::Repeat => t!("utils.repeat").to_string(),
            BindFlagsEnum::NonConsuming => t!("utils.non_consuming").to_string(),
            BindFlagsEnum::Mouse => t!("utils.mouse").to_string(),
            BindFlagsEnum::Transparent => t!("utils.transparent").to_string(),
            BindFlagsEnum::IgnoreMods => t!("utils.ignore_mods").to_string(),
            BindFlagsEnum::Separate => t!("utils.separate").to_string(),
            BindFlagsEnum::HasDescription => t!("utils.has_description").to_string(),
            BindFlagsEnum::Bypass => t!("utils.bypass").to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BindFlags {
    pub locked: bool,
    pub release: bool,
    pub click: bool,
    pub drag: bool,
    pub long_press: bool,
    pub repeat: bool,
    pub non_consuming: bool,
    pub mouse: bool,
    pub transparent: bool,
    pub ignore_mods: bool,
    pub separate: bool,
    pub has_description: bool,
    pub bypass: bool,
}

impl FromStr for BindFlags {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flags = BindFlags::default();
        for c in s.trim().chars() {
            match c {
                'l' => flags.locked = true,
                'r' => flags.release = true,
                'c' => flags.click = true,
                'g' => flags.drag = true,
                'o' => flags.long_press = true,
                'e' => flags.repeat = true,
                'n' => flags.non_consuming = true,
                'm' => flags.mouse = true,
                't' => flags.transparent = true,
                'i' => flags.ignore_mods = true,
                's' => flags.separate = true,
                'd' => flags.has_description = true,
                'p' => flags.bypass = true,
                _ => {}
            }
        }
        Ok(flags)
    }
}

impl Display for BindFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = String::new();
        if self.locked {
            flags.push('l');
        }
        if self.release {
            flags.push('r');
        }
        if self.click {
            flags.push('c');
        }
        if self.drag {
            flags.push('g');
        }
        if self.long_press {
            flags.push('o');
        }
        if self.repeat {
            flags.push('e');
        }
        if self.non_consuming {
            flags.push('n');
        }
        if self.mouse {
            flags.push('m');
        }
        if self.transparent {
            flags.push('t');
        }
        if self.ignore_mods {
            flags.push('i');
        }
        if self.separate {
            flags.push('s');
        }
        if self.has_description {
            flags.push('d');
        }
        if self.bypass {
            flags.push('p');
        }
        write!(f, "{}", flags)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindLeft {
    Bind(BindFlags),
    Unbind,
}

impl FromStr for BindLeft {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Some(flags) = s.strip_prefix("bind") {
            if flags.is_empty() {
                Ok(BindLeft::Bind(BindFlags::default()))
            } else {
                let flags = flags.parse().unwrap_or_default();
                Ok(BindLeft::Bind(flags))
            }
        } else if s == "unbind" {
            Ok(BindLeft::Unbind)
        } else {
            Err(())
        }
    }
}

impl Display for BindLeft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BindLeft::Bind(flags) if flags == &BindFlags::default() => write!(f, "bind"),
            BindLeft::Bind(flags) => write!(f, "bind{}", flags),
            BindLeft::Unbind => write!(f, "unbind"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(MonitorTargetDescriminants))]
#[derive(Default)]
pub enum MonitorTarget {
    Direction(Direction),
    Id(u32),
    Name(String),
    #[default]
    Current,
    Relative(i32),
}

impl HasDiscriminant for MonitorTarget {
    type Discriminant = MonitorTargetDescriminants;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Direction => Self::Direction(Direction::default()),
            Self::Discriminant::Id => Self::Id(0),
            Self::Discriminant::Name => Self::Name("".to_string()),
            Self::Discriminant::Current => Self::Current,
            Self::Discriminant::Relative => Self::Relative(0),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Direction => {
                Self::Direction(Direction::from_str(str).unwrap_or_default())
            }
            Self::Discriminant::Id => Self::Id(str.parse().unwrap_or_default()),
            Self::Discriminant::Name => Self::Name(str.to_string()),
            Self::Discriminant::Current => Self::Current,
            Self::Discriminant::Relative => Self::Relative(str.parse().unwrap_or_default()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            MonitorTarget::Direction(direction) => Some(direction.to_string()),
            MonitorTarget::Id(id) => Some(id.to_string()),
            MonitorTarget::Name(name) => Some(name.to_string()),
            MonitorTarget::Current => None,
            MonitorTarget::Relative(rel_id) => Some(format!("{:+}", rel_id)),
        }
    }
}

impl FromStr for MonitorTarget {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if (s.starts_with("+") || s.starts_with("-"))
            && let Ok(rel_id) = s.parse::<i32>()
        {
            return Ok(MonitorTarget::Relative(rel_id));
        }

        if let Ok(dir) = s.parse::<Direction>() {
            Ok(MonitorTarget::Direction(dir))
        } else if let Ok(id) = s.parse::<u32>() {
            Ok(MonitorTarget::Id(id))
        } else if s == "current" {
            Ok(MonitorTarget::Current)
        } else if let Ok(rel_id) = s.parse::<i32>() {
            Ok(MonitorTarget::Relative(rel_id))
        } else {
            Ok(MonitorTarget::Name(s.to_string()))
        }
    }
}

impl Display for MonitorTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorTarget::Direction(dir) => write!(f, "{}", dir),
            MonitorTarget::Id(id) => write!(f, "{}", id),
            MonitorTarget::Name(name) => write!(f, "{}", name),
            MonitorTarget::Current => write!(f, "current"),
            MonitorTarget::Relative(rel_id) => write!(f, "{:+}", rel_id),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(PixelOrPercentDiscriminant))]
pub enum PixelOrPercent {
    Pixel(i32),
    Percent(f64),
}

impl HasDiscriminant for PixelOrPercent {
    type Discriminant = PixelOrPercentDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Pixel => Self::Pixel(0),
            Self::Discriminant::Percent => Self::Percent(0.0),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Pixel => Self::Pixel(str.parse().unwrap_or_default()),
            Self::Discriminant::Percent => Self::Percent(str.parse().unwrap_or_default()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            PixelOrPercent::Pixel(p) => Some(p.to_string()),
            PixelOrPercent::Percent(p) => Some(format!("{:.2}", p)),
        }
    }
}

impl Default for PixelOrPercent {
    fn default() -> Self {
        PixelOrPercent::Pixel(0)
    }
}

impl FromStr for PixelOrPercent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Ok(p) = s.parse::<i32>() {
            Ok(PixelOrPercent::Pixel(p))
        } else if let Some(stripped) = s.strip_suffix("%") {
            if let Ok(p) = stripped.parse::<f64>() {
                Ok(PixelOrPercent::Percent(p))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

impl Display for PixelOrPercent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PixelOrPercent::Pixel(p) => write!(f, "{}", p),
            PixelOrPercent::Percent(p) => write!(f, "{}%", p),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(ResizeParamsDiscriminant))]
pub enum ResizeParams {
    Relative(PixelOrPercent, PixelOrPercent),
    Exact(PixelOrPercent, PixelOrPercent),
}

impl HasDiscriminant for ResizeParams {
    type Discriminant = ResizeParamsDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Relative => {
                Self::Relative(PixelOrPercent::default(), PixelOrPercent::default())
            }
            Self::Discriminant::Exact => {
                Self::Exact(PixelOrPercent::default(), PixelOrPercent::default())
            }
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Relative => match Self::from_str(str) {
                Ok(ResizeParams::Relative(p1, p2)) => ResizeParams::Relative(p1, p2),
                Ok(ResizeParams::Exact(p1, p2)) => ResizeParams::Relative(p1, p2),
                Err(_) => {
                    ResizeParams::Relative(PixelOrPercent::default(), PixelOrPercent::default())
                }
            },
            Self::Discriminant::Exact => match Self::from_str(str) {
                Ok(ResizeParams::Relative(p1, p2)) => ResizeParams::Exact(p1, p2),
                Ok(ResizeParams::Exact(p1, p2)) => ResizeParams::Exact(p1, p2),
                Err(_) => ResizeParams::Exact(PixelOrPercent::default(), PixelOrPercent::default()),
            },
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            ResizeParams::Relative(width, height) => Some(format!("{} {}", width, height)),
            ResizeParams::Exact(width, height) => Some(format!("{} {}", width, height)),
        }
    }
}

impl Default for ResizeParams {
    fn default() -> Self {
        ResizeParams::Relative(PixelOrPercent::default(), PixelOrPercent::default())
    }
}

impl FromStr for ResizeParams {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Some(s) = s.strip_prefix("exact ") {
            let (width, height) = s.split_once(' ').unwrap_or(("", ""));
            let width = width
                .parse::<PixelOrPercent>()
                .unwrap_or(PixelOrPercent::Pixel(0));
            let height = height
                .parse::<PixelOrPercent>()
                .unwrap_or(PixelOrPercent::Pixel(0));
            Ok(ResizeParams::Exact(width, height))
        } else {
            let (width, height) = s.split_once(' ').unwrap_or(("", ""));
            let width = width
                .parse::<PixelOrPercent>()
                .unwrap_or(PixelOrPercent::Pixel(0));
            let height = height
                .parse::<PixelOrPercent>()
                .unwrap_or(PixelOrPercent::Pixel(0));
            Ok(ResizeParams::Relative(width, height))
        }
    }
}

impl Display for ResizeParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResizeParams::Relative(width, height) => write!(f, "{} {}", width, height),
            ResizeParams::Exact(width, height) => write!(f, "exact {} {}", width, height),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(FloatValueDiscriminant))]
pub enum FloatValue {
    Relative(f64),
    Exact(f64),
}

impl HasDiscriminant for FloatValue {
    type Discriminant = FloatValueDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Relative => Self::Relative(0.0),
            Self::Discriminant::Exact => Self::Exact(0.0),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Relative => match Self::from_str(str) {
                Ok(FloatValue::Relative(f)) => FloatValue::Relative(f),
                Ok(FloatValue::Exact(f)) => FloatValue::Relative(f),
                Err(_) => FloatValue::Relative(0.0),
            },
            Self::Discriminant::Exact => match Self::from_str(str) {
                Ok(FloatValue::Relative(f)) => FloatValue::Exact(f),
                Ok(FloatValue::Exact(f)) => FloatValue::Exact(f),
                Err(_) => FloatValue::Exact(0.0),
            },
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            FloatValue::Relative(f) => Some(format!("{:+}", f)),
            FloatValue::Exact(f) => Some(format!("{}", f)),
        }
    }
}

impl Default for FloatValue {
    fn default() -> Self {
        FloatValue::Relative(0.0)
    }
}

impl FromStr for FloatValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Some(s) = s.strip_prefix("exact ") {
            let float = s.parse::<f64>().unwrap_or(0.0);
            Ok(FloatValue::Exact(float.abs()))
        } else {
            let float = s.parse::<f64>().unwrap_or(0.0);
            Ok(FloatValue::Relative(float))
        }
    }
}

impl Display for FloatValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FloatValue::Relative(float) => write!(f, "{:+}", float),
            FloatValue::Exact(float) => write!(f, "exact {}", float),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter)]
pub enum ZHeight {
    #[default]
    Top,
    Bottom,
}

impl FromStr for ZHeight {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "top" => Ok(ZHeight::Top),
            "bottom" => Ok(ZHeight::Bottom),
            _ => Err(()),
        }
    }
}

impl Display for ZHeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZHeight::Top => write!(f, "top"),
            ZHeight::Bottom => write!(f, "bottom"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum Direction {
    #[default]
    Left,
    Right,
    Up,
    Down,
}

impl FromStr for Direction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "l" | "left" => Ok(Direction::Left),
            "r" | "right" => Ok(Direction::Right),
            "u" | "up" => Ok(Direction::Up),
            "d" | "down" => Ok(Direction::Down),
            _ => Err(()),
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Left => write!(f, "left"),
            Direction::Right => write!(f, "right"),
            Direction::Up => write!(f, "up"),
            Direction::Down => write!(f, "down"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter)]
pub enum FullscreenMode {
    #[default]
    Fullscreen,
    Maximize,
}

impl FullscreenMode {
    pub fn from_num(num: u8) -> Self {
        match num {
            0 => FullscreenMode::Fullscreen,
            1 => FullscreenMode::Maximize,
            _ => FullscreenMode::Fullscreen,
        }
    }

    pub fn to_num(self) -> u8 {
        match self {
            FullscreenMode::Fullscreen => 0,
            FullscreenMode::Maximize => 1,
        }
    }
}

impl FromStr for FullscreenMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Ok(num) = s.parse::<u8>() {
            Ok(FullscreenMode::from_num(num))
        } else {
            Err(())
        }
    }
}

impl Display for FullscreenMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_num())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(RelativeIdDiscriminant))]
pub enum RelativeId {
    Absolute(u32),
    Previous(u32),
    Next(u32),
}

impl HasDiscriminant for RelativeId {
    type Discriminant = RelativeIdDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Absolute => Self::Absolute(1),
            Self::Discriminant::Previous => Self::Previous(1),
            Self::Discriminant::Next => Self::Next(1),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Absolute => Self::Absolute(str.parse().unwrap_or_default()),
            Self::Discriminant::Previous => Self::Previous(str.parse().unwrap_or_default()),
            Self::Discriminant::Next => Self::Next(str.parse().unwrap_or_default()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            RelativeId::Absolute(id) => Some(id.to_string()),
            RelativeId::Previous(id) => Some(id.to_string()),
            RelativeId::Next(id) => Some(id.to_string()),
        }
    }
}

impl Default for RelativeId {
    fn default() -> Self {
        RelativeId::Absolute(1)
    }
}

impl FromStr for RelativeId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        match s.chars().next().unwrap() {
            '~' => Ok(RelativeId::Absolute(s[1..].parse::<u32>().unwrap_or(1))),
            '-' => Ok(RelativeId::Previous(s[1..].parse::<u32>().unwrap_or(1))),
            '+' => Ok(RelativeId::Next(s[1..].parse::<u32>().unwrap_or(1))),
            _ => Err(()),
        }
    }
}

impl Display for RelativeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelativeId::Absolute(id) => write!(f, "~{}", id),
            RelativeId::Previous(id) => write!(f, "-{}", id),
            RelativeId::Next(id) => write!(f, "+{}", id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WorkspaceTargetDiscriminant))]
pub enum WorkspaceTarget {
    Id(u32),
    Relative(i32),
    OnMonitor(RelativeId),
    OnMonitorIncludingEmptyWorkspace(RelativeId),
    Open(RelativeId),
    Name(String),
    Previous,
    PreviousPerMonitor,
    FirstAvailableEmptyWorkspace,
    NextAvailableEmptyWorkspace,
    FirstAvailableEmptyWorkspaceOnMonitor,
    NextAvailableEmptyWorkspaceOnMonitor,
    Special,
    SpecialWithName(String),
}

impl HasDiscriminant for WorkspaceTarget {
    type Discriminant = WorkspaceTargetDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Id => Self::Id(1),
            Self::Discriminant::Relative => Self::Relative(0),
            Self::Discriminant::OnMonitor => Self::OnMonitor(RelativeId::default()),
            Self::Discriminant::OnMonitorIncludingEmptyWorkspace => {
                Self::OnMonitorIncludingEmptyWorkspace(RelativeId::default())
            }
            Self::Discriminant::Open => Self::Open(RelativeId::default()),
            Self::Discriminant::Name => Self::Name("".to_string()),
            Self::Discriminant::Previous => Self::Previous,
            Self::Discriminant::PreviousPerMonitor => Self::PreviousPerMonitor,
            Self::Discriminant::FirstAvailableEmptyWorkspace => Self::FirstAvailableEmptyWorkspace,
            Self::Discriminant::NextAvailableEmptyWorkspace => Self::NextAvailableEmptyWorkspace,
            Self::Discriminant::FirstAvailableEmptyWorkspaceOnMonitor => {
                Self::FirstAvailableEmptyWorkspaceOnMonitor
            }
            Self::Discriminant::NextAvailableEmptyWorkspaceOnMonitor => {
                Self::NextAvailableEmptyWorkspaceOnMonitor
            }
            Self::Discriminant::Special => Self::Special,
            Self::Discriminant::SpecialWithName => Self::SpecialWithName("".to_string()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Id => Self::Id(str.parse().unwrap_or(1)),
            Self::Discriminant::Relative => Self::Relative(str.parse().unwrap_or(0)),
            Self::Discriminant::OnMonitor => {
                Self::OnMonitor(str.parse().unwrap_or(RelativeId::default()))
            }
            Self::Discriminant::OnMonitorIncludingEmptyWorkspace => {
                Self::OnMonitorIncludingEmptyWorkspace(str.parse().unwrap_or(RelativeId::default()))
            }
            Self::Discriminant::Open => Self::Open(str.parse().unwrap_or(RelativeId::default())),
            Self::Discriminant::Name => Self::Name(str.to_string()),
            Self::Discriminant::Previous => Self::Previous,
            Self::Discriminant::PreviousPerMonitor => Self::PreviousPerMonitor,
            Self::Discriminant::FirstAvailableEmptyWorkspace => Self::FirstAvailableEmptyWorkspace,
            Self::Discriminant::NextAvailableEmptyWorkspace => Self::NextAvailableEmptyWorkspace,
            Self::Discriminant::FirstAvailableEmptyWorkspaceOnMonitor => {
                Self::FirstAvailableEmptyWorkspaceOnMonitor
            }
            Self::Discriminant::NextAvailableEmptyWorkspaceOnMonitor => {
                Self::NextAvailableEmptyWorkspaceOnMonitor
            }
            Self::Discriminant::Special => Self::Special,
            Self::Discriminant::SpecialWithName => Self::SpecialWithName(str.to_string()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            WorkspaceTarget::Id(id) => Some(id.to_string()),
            WorkspaceTarget::Relative(rel_id) => Some(format!("{:+}", rel_id)),
            WorkspaceTarget::OnMonitor(rel_id) => Some(rel_id.to_string()),
            WorkspaceTarget::OnMonitorIncludingEmptyWorkspace(rel_id) => Some(rel_id.to_string()),
            WorkspaceTarget::Open(rel_id) => Some(rel_id.to_string()),
            WorkspaceTarget::Name(name) => Some(name.clone()),
            WorkspaceTarget::Previous => None,
            WorkspaceTarget::PreviousPerMonitor => None,
            WorkspaceTarget::FirstAvailableEmptyWorkspace => None,
            WorkspaceTarget::NextAvailableEmptyWorkspace => None,
            WorkspaceTarget::FirstAvailableEmptyWorkspaceOnMonitor => None,
            WorkspaceTarget::NextAvailableEmptyWorkspaceOnMonitor => None,
            WorkspaceTarget::Special => None,
            WorkspaceTarget::SpecialWithName(name) => Some(name.clone()),
        }
    }
}

impl Default for WorkspaceTarget {
    fn default() -> Self {
        WorkspaceTarget::Id(1)
    }
}

impl FromStr for WorkspaceTarget {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Ok(id) = s.parse::<u32>() {
            let id = match id {
                0 => 1,
                id => id,
            };
            Ok(WorkspaceTarget::Id(id))
        } else if let Ok(rel_id) = s.parse::<i32>() {
            Ok(WorkspaceTarget::Relative(rel_id))
        } else if let Some(s) = s.strip_prefix("m") {
            Ok(WorkspaceTarget::OnMonitor(s.parse().unwrap_or_default()))
        } else if let Some(s) = s.strip_prefix("r") {
            Ok(WorkspaceTarget::OnMonitorIncludingEmptyWorkspace(
                s.parse().unwrap_or_default(),
            ))
        } else if let Some(s) = s.strip_prefix("e") {
            Ok(WorkspaceTarget::Open(s.parse().unwrap_or_default()))
        } else if let Some(s) = s.strip_prefix("name:") {
            Ok(WorkspaceTarget::Name(
                s.trim_start_matches("name:").to_string(),
            ))
        } else if s == "previous" {
            Ok(WorkspaceTarget::Previous)
        } else if s == "previous_per_monitor" {
            Ok(WorkspaceTarget::PreviousPerMonitor)
        } else if s == "empty" {
            Ok(WorkspaceTarget::FirstAvailableEmptyWorkspace)
        } else if s == "emptyn" {
            Ok(WorkspaceTarget::NextAvailableEmptyWorkspace)
        } else if s == "emptym" {
            Ok(WorkspaceTarget::FirstAvailableEmptyWorkspaceOnMonitor)
        } else if s == "emptymn" || s == "emptynm" {
            Ok(WorkspaceTarget::NextAvailableEmptyWorkspaceOnMonitor)
        } else if s == "special" {
            Ok(WorkspaceTarget::Special)
        } else if let Some(s) = s.strip_prefix("special:") {
            Ok(WorkspaceTarget::SpecialWithName(
                s.trim_start_matches("special:").to_string(),
            ))
        } else {
            Err(())
        }
    }
}

impl Display for WorkspaceTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceTarget::Id(id) => write!(f, "{}", id),
            WorkspaceTarget::Relative(rel_id) => write!(f, "{:+}", rel_id),
            WorkspaceTarget::OnMonitor(rel_id) => write!(f, "m{}", rel_id),
            WorkspaceTarget::OnMonitorIncludingEmptyWorkspace(rel_id) => write!(f, "r{}", rel_id),
            WorkspaceTarget::Open(rel_id) => write!(f, "e{}", rel_id),
            WorkspaceTarget::Name(name) => write!(f, "name:{}", name),
            WorkspaceTarget::Previous => write!(f, "previous"),
            WorkspaceTarget::PreviousPerMonitor => write!(f, "previous_per_monitor"),
            WorkspaceTarget::FirstAvailableEmptyWorkspace => write!(f, "empty"),
            WorkspaceTarget::NextAvailableEmptyWorkspace => write!(f, "emptyn"),
            WorkspaceTarget::FirstAvailableEmptyWorkspaceOnMonitor => write!(f, "emptym"),
            WorkspaceTarget::NextAvailableEmptyWorkspaceOnMonitor => write!(f, "emptynm"),
            WorkspaceTarget::Special => write!(f, "special"),
            WorkspaceTarget::SpecialWithName(name) => write!(f, "special:{}", name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WindowTargetDiscriminant))]
pub enum WindowTarget {
    Class(String),
    InitialClass(String),
    Title(String),
    InitialTitle(String),
    Tag(String),
    Pid(String),
    Address(String),
    ActiveWindow,
    Floating,
    Tiled,
}

impl HasDiscriminant for WindowTarget {
    type Discriminant = WindowTargetDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Class => Self::Class("".to_string()),
            Self::Discriminant::InitialClass => Self::InitialClass("".to_string()),
            Self::Discriminant::Title => Self::Title("".to_string()),
            Self::Discriminant::InitialTitle => Self::InitialTitle("".to_string()),
            Self::Discriminant::Tag => Self::Tag("".to_string()),
            Self::Discriminant::Pid => Self::Pid("".to_string()),
            Self::Discriminant::Address => Self::Address("".to_string()),
            Self::Discriminant::ActiveWindow => Self::ActiveWindow,
            Self::Discriminant::Floating => Self::Floating,
            Self::Discriminant::Tiled => Self::Tiled,
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Class => Self::Class(str.to_string()),
            Self::Discriminant::InitialClass => Self::InitialClass(str.to_string()),
            Self::Discriminant::Title => Self::Title(str.to_string()),
            Self::Discriminant::InitialTitle => Self::InitialTitle(str.to_string()),
            Self::Discriminant::Tag => Self::Tag(str.to_string()),
            Self::Discriminant::Pid => Self::Pid(str.to_string()),
            Self::Discriminant::Address => Self::Address(str.to_string()),
            Self::Discriminant::ActiveWindow => Self::ActiveWindow,
            Self::Discriminant::Floating => Self::Floating,
            Self::Discriminant::Tiled => Self::Tiled,
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            WindowTarget::Class(class) => Some(class.clone()),
            WindowTarget::InitialClass(initial_class) => Some(initial_class.clone()),
            WindowTarget::Title(title) => Some(title.clone()),
            WindowTarget::InitialTitle(initial_title) => Some(initial_title.clone()),
            WindowTarget::Tag(tag) => Some(tag.clone()),
            WindowTarget::Pid(pid) => Some(pid.clone()),
            WindowTarget::Address(addr) => Some(addr.clone()),
            WindowTarget::ActiveWindow => None,
            WindowTarget::Floating => None,
            WindowTarget::Tiled => None,
        }
    }
}

impl Default for WindowTarget {
    fn default() -> Self {
        WindowTarget::Class("".to_string())
    }
}

impl FromStr for WindowTarget {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if s.starts_with("class:") {
            Ok(WindowTarget::Class(
                s.trim_start_matches("class:").to_string(),
            ))
        } else if s.starts_with("initialclass:") {
            Ok(WindowTarget::InitialClass(
                s.trim_start_matches("initialclass:").to_string(),
            ))
        } else if s.starts_with("title:") {
            Ok(WindowTarget::Title(
                s.trim_start_matches("title:").to_string(),
            ))
        } else if s.starts_with("initialtitle:") {
            Ok(WindowTarget::InitialTitle(
                s.trim_start_matches("initialtitle:").to_string(),
            ))
        } else if s.starts_with("tag:") {
            Ok(WindowTarget::Tag(s.trim_start_matches("tag:").to_string()))
        } else if s.starts_with("pid:") {
            Ok(WindowTarget::Pid(s.trim_start_matches("pid:").to_string()))
        } else if s == "address" {
            Ok(WindowTarget::Address(s.to_string()))
        } else if s == "activewindow" {
            Ok(WindowTarget::ActiveWindow)
        } else if s == "floating" {
            Ok(WindowTarget::Floating)
        } else if s == "tiled" {
            Ok(WindowTarget::Tiled)
        } else {
            Ok(WindowTarget::Class(s.to_string()))
        }
    }
}

impl Display for WindowTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowTarget::Class(class) => write!(f, "{}", class),
            WindowTarget::InitialClass(initial_class) => {
                write!(f, "initialclass:{}", initial_class)
            }
            WindowTarget::Title(title) => write!(f, "title:{}", title),
            WindowTarget::InitialTitle(initial_title) => {
                write!(f, "initialtitle:{}", initial_title)
            }
            WindowTarget::Tag(tag) => write!(f, "tag:{}", tag),
            WindowTarget::Pid(pid) => write!(f, "pid:{}", pid),
            WindowTarget::Address(addr) => write!(f, "address:{}", addr),
            WindowTarget::ActiveWindow => write!(f, "activewindow"),
            WindowTarget::Floating => write!(f, "floating"),
            WindowTarget::Tiled => write!(f, "tiled"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum CursorCorner {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl CursorCorner {
    pub fn to_num(self) -> u8 {
        match self {
            CursorCorner::TopLeft => 0,
            CursorCorner::TopRight => 1,
            CursorCorner::BottomLeft => 2,
            CursorCorner::BottomRight => 3,
        }
    }

    pub fn from_num(num: u8) -> Self {
        match num {
            0 => CursorCorner::TopLeft,
            1 => CursorCorner::TopRight,
            2 => CursorCorner::BottomLeft,
            3 => CursorCorner::BottomRight,
            _ => CursorCorner::TopLeft,
        }
    }
}

impl FromStr for CursorCorner {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_num(s.parse().unwrap_or_default()))
    }
}

impl Display for CursorCorner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_num())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum GroupLockAction {
    Lock,
    Unlock,
    #[default]
    Toggle,
}

impl FromStr for GroupLockAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "lock" => Ok(GroupLockAction::Lock),
            "unlock" => Ok(GroupLockAction::Unlock),
            "toggle" => Ok(GroupLockAction::Toggle),
            _ => Err(()),
        }
    }
}

impl Display for GroupLockAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupLockAction::Lock => write!(f, "lock"),
            GroupLockAction::Unlock => write!(f, "unlock"),
            GroupLockAction::Toggle => write!(f, "toggle"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum ToggleState {
    On,
    Off,
    #[default]
    Toggle,
}

impl FromStr for ToggleState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "on" => Ok(ToggleState::On),
            "off" => Ok(ToggleState::Off),
            "toggle" => Ok(ToggleState::Toggle),
            _ => Err(()),
        }
    }
}

impl Display for ToggleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToggleState::On => write!(f, "on"),
            ToggleState::Off => write!(f, "off"),
            ToggleState::Toggle => write!(f, "toggle"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Default)]
pub enum FullscreenState {
    #[default]
    None,
    Maximize,
    Fullscreen,
    MaximizeAndFullscreen,
}

impl FullscreenState {
    pub fn from_num(num: u8) -> Self {
        match num {
            0 => FullscreenState::None,
            1 => FullscreenState::Maximize,
            2 => FullscreenState::Fullscreen,
            3 => FullscreenState::MaximizeAndFullscreen,
            _ => FullscreenState::None,
        }
    }

    pub fn to_num(self) -> u8 {
        match self {
            FullscreenState::None => 0,
            FullscreenState::Maximize => 1,
            FullscreenState::Fullscreen => 2,
            FullscreenState::MaximizeAndFullscreen => 3,
        }
    }
}

impl FromStr for FullscreenState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        if let Ok(num) = s.parse::<u8>() {
            Ok(FullscreenState::from_num(num))
        } else {
            Err(())
        }
    }
}

impl Display for FullscreenState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_num())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HyprCoord {
    pub x: PixelOrPercent,
    pub y: PixelOrPercent,
    pub x_sub: u32,
    pub y_sub: u32,
    pub under_cursor: bool,
    pub on_screen: bool,
}

impl FromStr for HyprCoord {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        let mut result = HyprCoord::default();

        let parts: Vec<&str> = s.split(' ').collect();

        if parts.is_empty() {
            return Err(());
        }

        let mut is_x = true;

        for part in parts {
            let part = part.trim();
            if part == "onscreen" {
                result.on_screen = true;
            } else if part == "undercursor" {
                result.under_cursor = true;
            } else {
                // parse "100", "100%", "100%-100"
                let (num_or_percent, sub) = part.split_once('-').unwrap_or((part, ""));
                let num_or_percent: PixelOrPercent =
                    PixelOrPercent::from_str(num_or_percent).unwrap_or_default();
                let sub: u32 = sub.parse().unwrap_or_default();
                if is_x {
                    result.x = num_or_percent;
                    result.x_sub = sub;
                    is_x = false;
                } else {
                    result.y = num_or_percent;
                    result.y_sub = sub;
                    break;
                }
            }
        }

        Ok(result)
    }
}

impl Display for HyprCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        if self.on_screen {
            result.push_str("onscreen ");
        }

        if self.under_cursor {
            result.push_str("undercursor ");
        }

        match self.x {
            PixelOrPercent::Percent(p) => {
                result.push_str(&p.to_string());
                result.push('%');
                if self.x_sub > 0 {
                    result.push('-');
                    result.push_str(&self.x_sub.to_string());
                }
            }
            PixelOrPercent::Pixel(p) => {
                result.push_str(&p.to_string());
            }
        }

        match self.y {
            PixelOrPercent::Percent(p) => {
                result.push_str(&p.to_string());
                result.push('%');
                if self.y_sub > 0 {
                    result.push('-');
                    result.push_str(&self.y_sub.to_string());
                }
            }
            PixelOrPercent::Pixel(p) => {
                result.push_str(&p.to_string());
            }
        }

        write!(f, "{}", result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum SizeBound {
    #[default]
    Exact,
    Max,
    Min,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HyprSize {
    pub width: PixelOrPercent,
    pub height: PixelOrPercent,
    pub width_bound: SizeBound,
    pub height_bound: SizeBound,
}

impl FromStr for HyprSize {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        let mut result = HyprSize {
            width: PixelOrPercent::Percent(50.0),
            height: PixelOrPercent::Percent(50.0),
            width_bound: SizeBound::Exact,
            height_bound: SizeBound::Exact,
        };

        let parts: Vec<&str> = s.split(' ').collect();

        let width = parts.first().unwrap_or(&"");
        let height = parts.get(1).unwrap_or(&"");

        if let Some(stripped) = width.strip_prefix("<") {
            result.width_bound = SizeBound::Max;
            result.width = PixelOrPercent::from_str(stripped).unwrap_or_default();
        } else if let Some(stripped) = width.strip_prefix(">") {
            result.width_bound = SizeBound::Min;
            result.width = PixelOrPercent::from_str(stripped).unwrap_or_default();
        } else {
            result.width = PixelOrPercent::from_str(width).unwrap_or_default();
        }

        if let Some(stripped) = height.strip_prefix("<") {
            result.height_bound = SizeBound::Max;
            result.height = PixelOrPercent::from_str(stripped).unwrap_or_default();
        } else if let Some(stripped) = height.strip_prefix(">") {
            result.height_bound = SizeBound::Min;
            result.height = PixelOrPercent::from_str(stripped).unwrap_or_default();
        } else {
            result.height = PixelOrPercent::from_str(height).unwrap_or_default();
        }

        Ok(result)
    }
}

impl Display for HyprSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        match self.width_bound {
            SizeBound::Exact => {}
            SizeBound::Max => result.push('<'),
            SizeBound::Min => result.push('>'),
        }

        match self.width {
            PixelOrPercent::Percent(p) => {
                result.push_str(&p.to_string());
                result.push('%');
            }
            PixelOrPercent::Pixel(p) => {
                result.push_str(&p.to_string());
            }
        }

        match self.height_bound {
            SizeBound::Exact => {}
            SizeBound::Max => result.push('<'),
            SizeBound::Min => result.push('>'),
        }

        match self.height {
            PixelOrPercent::Percent(p) => {
                result.push_str(&p.to_string());
                result.push('%');
            }
            PixelOrPercent::Pixel(p) => {
                result.push_str(&p.to_string());
            }
        }

        write!(f, "{}", result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(IdOrNameDiscriminant))]
pub enum IdOrName {
    Id(u32),
    Name(String),
}

impl HasDiscriminant for IdOrName {
    type Discriminant = IdOrNameDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Id => Self::Id(0),
            Self::Discriminant::Name => Self::Name("".to_string()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Id => Self::Id(str.parse().unwrap_or_default()),
            Self::Discriminant::Name => Self::Name(str.to_string()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            IdOrName::Id(id) => Some(id.to_string()),
            IdOrName::Name(name) => Some(name.clone()),
        }
    }
}

impl Default for IdOrName {
    fn default() -> Self {
        IdOrName::Id(0)
    }
}

impl FromStr for IdOrName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Ok(id) = s.parse::<u32>() {
            Ok(IdOrName::Id(id))
        } else {
            Ok(IdOrName::Name(s.to_string()))
        }
    }
}

impl Display for IdOrName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdOrName::Id(id) => write!(f, "{}", id),
            IdOrName::Name(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum WindowGroupOption {
    #[default]
    Set,
    SetAlways,
    New,
    Lock,
    LockAlways,
    Barred,
    Deny,
    Invade,
    Override,
    Unset,
}

impl FromStr for WindowGroupOption {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        match s {
            "set" => Ok(WindowGroupOption::Set),
            "set always" => Ok(WindowGroupOption::SetAlways),
            "new" => Ok(WindowGroupOption::New),
            "lock" => Ok(WindowGroupOption::Lock),
            "lock always" => Ok(WindowGroupOption::LockAlways),
            "barred" => Ok(WindowGroupOption::Barred),
            "deny" => Ok(WindowGroupOption::Deny),
            "invade" => Ok(WindowGroupOption::Invade),
            "override" => Ok(WindowGroupOption::Override),
            "unset" => Ok(WindowGroupOption::Unset),
            _ => Err(()),
        }
    }
}

impl Display for WindowGroupOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowGroupOption::Set => write!(f, "set"),
            WindowGroupOption::SetAlways => write!(f, "set always"),
            WindowGroupOption::New => write!(f, "new"),
            WindowGroupOption::Lock => write!(f, "lock"),
            WindowGroupOption::LockAlways => write!(f, "lock always"),
            WindowGroupOption::Barred => write!(f, "barred"),
            WindowGroupOption::Deny => write!(f, "deny"),
            WindowGroupOption::Invade => write!(f, "invade"),
            WindowGroupOption::Override => write!(f, "override"),
            WindowGroupOption::Unset => write!(f, "unset"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum WindowEvent {
    #[default]
    Fullscreen,
    Maximize,
    Activate,
    ActivateFocus,
    FullscreenOutput,
}

impl FromStr for WindowEvent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        match s {
            "fullscreen" => Ok(WindowEvent::Fullscreen),
            "maximize" => Ok(WindowEvent::Maximize),
            "activate" => Ok(WindowEvent::Activate),
            "activatefocus" => Ok(WindowEvent::ActivateFocus),
            "fullscreenoutput" => Ok(WindowEvent::FullscreenOutput),
            _ => Err(()),
        }
    }
}

impl Display for WindowEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowEvent::Fullscreen => write!(f, "fullscreen"),
            WindowEvent::Maximize => write!(f, "maximize"),
            WindowEvent::Activate => write!(f, "activate"),
            WindowEvent::ActivateFocus => write!(f, "activatefocus"),
            WindowEvent::FullscreenOutput => write!(f, "fullscreenoutput"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum ContentType {
    #[default]
    None,
    Photo,
    Video,
    Game,
}

impl FromStr for ContentType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        match s {
            "none" => Ok(ContentType::None),
            "photo" => Ok(ContentType::Photo),
            "video" => Ok(ContentType::Video),
            "game" => Ok(ContentType::Game),
            _ => Err(()),
        }
    }
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::None => write!(f, "none"),
            ContentType::Photo => write!(f, "photo"),
            ContentType::Video => write!(f, "video"),
            ContentType::Game => write!(f, "game"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(HyprColorDiscriminant))]
pub enum HyprColor {
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, u8),
}

impl Default for HyprColor {
    fn default() -> Self {
        HyprColor::Rgb(0, 0, 0)
    }
}

impl HasDiscriminant for HyprColor {
    type Discriminant = HyprColorDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Rgb => Self::Rgb(0, 0, 0),
            Self::Discriminant::Rgba => Self::Rgba(0, 0, 0, 255),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Rgb => {
                let parts: Vec<&str> = str.split(',').collect();
                if parts.len() >= 3 {
                    let r = parts[0].parse().unwrap_or(0);
                    let g = parts[1].parse().unwrap_or(0);
                    let b = parts[2].parse().unwrap_or(0);
                    Self::Rgb(r, g, b)
                } else {
                    Self::Rgb(0, 0, 0)
                }
            }
            Self::Discriminant::Rgba => {
                let parts: Vec<&str> = str.split(',').collect();
                if parts.len() >= 4 {
                    let r = parts[0].parse().unwrap_or(0);
                    let g = parts[1].parse().unwrap_or(0);
                    let b = parts[2].parse().unwrap_or(0);
                    let a = parts[3].parse().unwrap_or(255);
                    Self::Rgba(r, g, b, a)
                } else if parts.len() == 3 {
                    let r = parts[0].parse().unwrap_or(0);
                    let g = parts[1].parse().unwrap_or(0);
                    let b = parts[2].parse().unwrap_or(0);
                    Self::Rgba(r, g, b, 255)
                } else {
                    Self::Rgba(0, 0, 0, 255)
                }
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Self::Rgb(r, g, b) => Some(format!("{},{},{}", r, g, b)),
            Self::Rgba(r, g, b, a) => Some(format!("{},{},{},{}", r, g, b, a)),
        }
    }
}

impl FromStr for HyprColor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() || !s.is_ascii() {
            return Err(());
        }

        if s.starts_with("rgb(") && s.ends_with(')') {
            // rgb(255,0,0) and rgb(ff0000)
            let rgb_vec: Vec<&str> = s[4..s.len() - 1].split(',').collect();
            if rgb_vec.len() == 1 && rgb_vec[0].len() == 6 {
                let r = u8::from_str_radix(&rgb_vec[0][0..2], 16).unwrap_or_default();
                let g = u8::from_str_radix(&rgb_vec[0][2..4], 16).unwrap_or_default();
                let b = u8::from_str_radix(&rgb_vec[0][4..6], 16).unwrap_or_default();
                Ok(HyprColor::Rgb(r, g, b))
            } else if rgb_vec.len() == 3 {
                let r = u8::from_str(rgb_vec[0]).unwrap_or_default();
                let g = u8::from_str(rgb_vec[1]).unwrap_or_default();
                let b = u8::from_str(rgb_vec[2]).unwrap_or_default();
                Ok(HyprColor::Rgb(r, g, b))
            } else {
                Err(())
            }
        } else if s.starts_with("rgba(") && s.ends_with(')') {
            // rgba(255,0,0,1) and rgba(ff0000ff)
            let rgba_vec: Vec<&str> = s[5..s.len() - 1].split(',').collect();
            if rgba_vec.len() == 1 && rgba_vec[0].len() == 8 {
                let r = u8::from_str_radix(&rgba_vec[0][0..2], 16).unwrap_or_default();
                let g = u8::from_str_radix(&rgba_vec[0][2..4], 16).unwrap_or_default();
                let b = u8::from_str_radix(&rgba_vec[0][4..6], 16).unwrap_or_default();
                let a = u8::from_str_radix(&rgba_vec[0][6..8], 16).unwrap_or_default();
                Ok(HyprColor::Rgba(r, g, b, a))
            } else if rgba_vec.len() == 4 {
                let r = u8::from_str(rgba_vec[0]).unwrap_or_default();
                let g = u8::from_str(rgba_vec[1]).unwrap_or_default();
                let b = u8::from_str(rgba_vec[2]).unwrap_or_default();
                let a = (f64::from_str(rgba_vec[3]).unwrap_or_default() * 255.0) as u8;
                Ok(HyprColor::Rgba(r, g, b, a))
            } else {
                Err(())
            }
        } else if s.starts_with("0x") && s.len() == 10 {
            // 0xffff0000
            let a = u8::from_str_radix(&s[2..4], 16).unwrap_or_default();
            let r = u8::from_str_radix(&s[4..6], 16).unwrap_or_default();
            let g = u8::from_str_radix(&s[6..8], 16).unwrap_or_default();
            let b = u8::from_str_radix(&s[8..10], 16).unwrap_or_default();
            Ok(HyprColor::Rgba(r, g, b, a))
        } else {
            Err(())
        }
    }
}

impl Display for HyprColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyprColor::Rgb(r, g, b) => write!(f, "rgb({},{},{})", r, g, b),
            HyprColor::Rgba(r, g, b, a) => {
                write!(f, "rgba({},{},{},{})", r, g, b, *a as f64 / 255.0)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Angle {
    Degrees(u16),
}

impl Default for Angle {
    fn default() -> Self {
        Angle::Degrees(0)
    }
}

impl FromStr for Angle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        if let Some(stripped) = s.strip_suffix("deg") {
            let degrees = stripped.parse::<u16>().unwrap_or_default();
            Ok(Angle::Degrees(degrees))
        } else {
            Err(())
        }
    }
}

impl Display for Angle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Angle::Degrees(degrees) => write!(f, "{}deg", degrees),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BorderColor {
    Color(HyprColor),
    Gradient(Vec<HyprColor>, Angle),
    DoubleColor(HyprColor, HyprColor),
    DoubleGradient(Vec<HyprColor>, Angle, Vec<HyprColor>, Option<Angle>),
}

impl BorderColor {
    pub const SEPARATOR: char = ' ';
}

impl Default for BorderColor {
    fn default() -> Self {
        BorderColor::Color(HyprColor::default())
    }
}

impl FromStr for BorderColor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        let parts: Vec<&str> = s.split(Self::SEPARATOR).collect();
        if parts.len() == 1 {
            // Color
            let color = HyprColor::from_str(parts[0]).unwrap_or_default();
            Ok(BorderColor::Color(color))
        } else if parts.len() == 2 {
            // Double Color and Simple Gradient
            let color1 = HyprColor::from_str(parts[0]).unwrap_or_default();
            match parts[1].parse::<Angle>() {
                Ok(angle) => Ok(BorderColor::Gradient(vec![color1, color1], angle)),
                Err(_) => Ok(BorderColor::DoubleColor(
                    color1,
                    HyprColor::from_str(parts[1]).unwrap_or_default(),
                )),
            }
        } else {
            // Gradient or Double Gradient
            let mut first_gradient: Vec<HyprColor> = Vec::new();
            let mut first_angle: Angle = Angle::default();
            let mut first_angle_idx = 0;

            for (i, part) in parts.iter().enumerate() {
                if let Ok(angle) = Angle::from_str(part) {
                    first_angle = angle;
                    first_angle_idx = i;
                    break;
                } else if let Ok(color) = HyprColor::from_str(part) {
                    first_gradient.push(color);
                }
            }

            if first_gradient.len() == 1 {
                first_gradient.push(first_gradient[0]);
            }

            if first_angle_idx == parts.len() - 1 {
                // Gradient
                Ok(BorderColor::Gradient(first_gradient, first_angle))
            } else {
                // Double Gradient
                let mut second_gradient: Vec<HyprColor> = Vec::new();
                let mut second_angle: Option<Angle> = None;

                for part in parts[first_angle_idx + 1..].iter() {
                    if let Ok(angle) = Angle::from_str(part) {
                        second_angle = Some(angle);
                        break;
                    } else if let Ok(color) = HyprColor::from_str(part) {
                        second_gradient.push(color);
                    }
                }

                if second_gradient.len() == 1 {
                    second_gradient.push(second_gradient[0]);
                }

                Ok(BorderColor::DoubleGradient(
                    first_gradient,
                    first_angle,
                    second_gradient,
                    second_angle,
                ))
            }
        }
    }
}

impl Display for BorderColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BorderColor::Color(color) => write!(f, "{}", color),
            BorderColor::Gradient(colors, angle) => {
                let colors: String = join_with_separator(colors, &Self::SEPARATOR.to_string());
                write!(f, "{}{}{}", colors, Self::SEPARATOR, angle)
            }
            BorderColor::DoubleColor(color1, color2) => {
                write!(f, "{}{}{}", color1, Self::SEPARATOR, color2)
            }
            BorderColor::DoubleGradient(colors1, angle1, colors2, angle2) => {
                let colors1: String = join_with_separator(colors1, &Self::SEPARATOR.to_string());
                let colors2: String = join_with_separator(colors2, &Self::SEPARATOR.to_string());
                match angle2 {
                    None => write!(
                        f,
                        "{}{}{}{}{}",
                        colors1,
                        Self::SEPARATOR,
                        angle1,
                        Self::SEPARATOR,
                        colors2
                    ),
                    Some(angle2) => write!(
                        f,
                        "{}{}{}{}{}{}{}",
                        colors1,
                        Self::SEPARATOR,
                        angle1,
                        Self::SEPARATOR,
                        colors2,
                        Self::SEPARATOR,
                        angle2
                    ),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum IdleIngibitMode {
    #[default]
    None,
    Always,
    Focus,
    Fullscreen,
}

impl FromStr for IdleIngibitMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "none" => Ok(IdleIngibitMode::None),
            "always" => Ok(IdleIngibitMode::Always),
            "focus" => Ok(IdleIngibitMode::Focus),
            "fullscreen" => Ok(IdleIngibitMode::Fullscreen),
            _ => Err(()),
        }
    }
}

impl Display for IdleIngibitMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdleIngibitMode::None => write!(f, "none"),
            IdleIngibitMode::Always => write!(f, "always"),
            IdleIngibitMode::Focus => write!(f, "focus"),
            IdleIngibitMode::Fullscreen => write!(f, "fullscreen"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(HyprOpacityDiscriminant))]
pub enum HyprOpacity {
    Overall(f64, bool),
    ActiveAndInactive(f64, bool, f64, bool),
    ActiveAndInactiveAndFullscreen(f64, bool, f64, bool, f64, bool),
}

impl HasDiscriminant for HyprOpacity {
    type Discriminant = HyprOpacityDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Overall => Self::Overall(1.0, false),
            Self::Discriminant::ActiveAndInactive => {
                Self::ActiveAndInactive(1.0, false, 1.0, false)
            }
            Self::Discriminant::ActiveAndInactiveAndFullscreen => {
                Self::ActiveAndInactiveAndFullscreen(1.0, false, 1.0, false, 1.0, false)
            }
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Overall => {
                let parts: Vec<&str> = str.split_whitespace().collect();
                if parts.len() >= 2 && parts[1].to_lowercase() == "override" {
                    let opacity = parts[0].parse::<f64>().unwrap_or(1.0);
                    Self::Overall(opacity, true)
                } else {
                    let opacity = parts
                        .first()
                        .unwrap_or(&"1.0")
                        .parse::<f64>()
                        .unwrap_or(1.0);
                    Self::Overall(opacity, false)
                }
            }
            Self::Discriminant::ActiveAndInactive => {
                let parts: Vec<&str> = str.split_whitespace().collect();
                match parts.len() {
                    0 => Self::ActiveAndInactive(1.0, false, 1.0, false),
                    1 => {
                        let opacity1 = parts[0].parse::<f64>().unwrap_or(1.0);
                        Self::ActiveAndInactive(opacity1, false, 1.0, false)
                    }
                    2 => {
                        let opacity1 = parts[0].parse::<f64>().unwrap_or(1.0);
                        let opacity2 = parts[1].parse::<f64>().unwrap_or(1.0);
                        Self::ActiveAndInactive(opacity1, false, opacity2, false)
                    }
                    3 => {
                        let opacity1 = parts[0].parse::<f64>().unwrap_or(1.0);
                        let opacity2 = parts[1].parse::<f64>().unwrap_or(1.0);
                        let is_override2 = parts[2].to_lowercase() == "override";
                        Self::ActiveAndInactive(opacity1, false, opacity2, is_override2)
                    }
                    4 => {
                        let opacity1 = parts[0].parse::<f64>().unwrap_or(1.0);
                        let opacity2 = parts[1].parse::<f64>().unwrap_or(1.0);
                        Self::ActiveAndInactive(opacity1, true, opacity2, true)
                    }
                    _ => Self::ActiveAndInactive(1.0, false, 1.0, false),
                }
            }
            Self::Discriminant::ActiveAndInactiveAndFullscreen => {
                let parts: Vec<&str> = str.split_whitespace().collect();
                match parts.len() {
                    0 => Self::ActiveAndInactiveAndFullscreen(1.0, false, 1.0, false, 1.0, false),
                    1 => {
                        let opacity1 = parts[0].parse::<f64>().unwrap_or(1.0);
                        Self::ActiveAndInactiveAndFullscreen(
                            opacity1, false, 1.0, false, 1.0, false,
                        )
                    }
                    2 => {
                        let opacity1 = parts[0].parse::<f64>().unwrap_or(1.0);
                        let opacity2 = parts[1].parse::<f64>().unwrap_or(1.0);
                        Self::ActiveAndInactiveAndFullscreen(
                            opacity1, false, opacity2, false, 1.0, false,
                        )
                    }
                    3 => {
                        let opacity1 = parts[0].parse::<f64>().unwrap_or(1.0);
                        let opacity2 = parts[1].parse::<f64>().unwrap_or(1.0);
                        let opacity3 = parts[2].parse::<f64>().unwrap_or(1.0);
                        Self::ActiveAndInactiveAndFullscreen(
                            opacity1, false, opacity2, false, opacity3, false,
                        )
                    }
                    _ => {
                        let mut opacities = Vec::new();
                        let mut overrides = Vec::new();

                        for part in &parts {
                            if part.to_lowercase() == "override" {
                                if let Some(last) = overrides.last_mut() {
                                    *last = true;
                                }
                            } else if let Ok(opacity) = part.parse::<f64>() {
                                opacities.push(opacity);
                                overrides.push(false);
                            }
                        }

                        match (opacities.len(), overrides.len()) {
                            (3, 3) => Self::ActiveAndInactiveAndFullscreen(
                                opacities[0],
                                overrides[0],
                                opacities[1],
                                overrides[1],
                                opacities[2],
                                overrides[2],
                            ),
                            (3, 2) => Self::ActiveAndInactiveAndFullscreen(
                                opacities[0],
                                overrides[0],
                                opacities[1],
                                overrides[1],
                                opacities[2],
                                false,
                            ),
                            (2, 2) => Self::ActiveAndInactiveAndFullscreen(
                                opacities[0],
                                overrides[0],
                                opacities[1],
                                overrides[1],
                                1.0,
                                false,
                            ),
                            (1, 1) => Self::ActiveAndInactiveAndFullscreen(
                                opacities[0],
                                overrides[0],
                                1.0,
                                false,
                                1.0,
                                false,
                            ),
                            _ => Self::ActiveAndInactiveAndFullscreen(
                                1.0, false, 1.0, false, 1.0, false,
                            ),
                        }
                    }
                }
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            HyprOpacity::Overall(opacity, is_override) => {
                if *is_override {
                    Some(format!("{} override", opacity))
                } else {
                    Some(opacity.to_string())
                }
            }
            HyprOpacity::ActiveAndInactive(opacity1, is_override1, opacity2, is_override2) => {
                let mut parts = Vec::new();
                parts.push(opacity1.to_string());
                if *is_override1 {
                    parts.push("override".to_string());
                }
                parts.push(opacity2.to_string());
                if *is_override2 {
                    parts.push("override".to_string());
                }
                Some(parts.join(" "))
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                is_override1,
                opacity2,
                is_override2,
                opacity3,
                is_override3,
            ) => {
                let mut parts = Vec::new();
                parts.push(opacity1.to_string());
                if *is_override1 {
                    parts.push("override".to_string());
                }
                parts.push(opacity2.to_string());
                if *is_override2 {
                    parts.push("override".to_string());
                }
                parts.push(opacity3.to_string());
                if *is_override3 {
                    parts.push("override".to_string());
                }
                Some(parts.join(" "))
            }
        }
    }
}

impl Default for HyprOpacity {
    fn default() -> Self {
        HyprOpacity::Overall(1.0, false)
    }
}

impl FromStr for HyprOpacity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        let parts: Vec<&str> = s.split(' ').collect();
        if parts.len() == 1 {
            // Overall
            let opacity = parts[0].parse::<f64>().unwrap_or_default();
            Ok(HyprOpacity::Overall(opacity, false))
        } else if parts.len() == 2 {
            // Active and Inactive or Active override
            let opacity1 = parts[0].parse::<f64>().unwrap_or_default();
            match parts[1].trim().to_lowercase().as_str() {
                "override" => Ok(HyprOpacity::Overall(opacity1, true)),
                opacity2 => Ok(HyprOpacity::ActiveAndInactive(
                    opacity1,
                    false,
                    opacity2.parse::<f64>().unwrap_or_default(),
                    false,
                )),
            }
        } else if parts.len() == 3 {
            // A - Active
            // I - Inactive
            // F - Fullscreen
            // o - Override
            // variants for 3 parts: AoI, AIo, AIF
            let opacity1 = parts[0].parse::<f64>().unwrap_or_default();
            match parts[1].trim().to_lowercase().as_str() {
                "override" => Ok(HyprOpacity::ActiveAndInactive(
                    opacity1,
                    true,
                    parts[2].parse::<f64>().unwrap_or_default(),
                    false,
                )),
                opacity2 => {
                    let opacity2 = opacity2.parse::<f64>().unwrap_or_default();
                    match parts[2].trim().to_lowercase().as_str() {
                        "override" => Ok(HyprOpacity::ActiveAndInactive(
                            opacity1, false, opacity2, true,
                        )),
                        opacity3 => Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                            opacity1,
                            false,
                            opacity2,
                            false,
                            opacity3.parse::<f64>().unwrap_or_default(),
                            false,
                        )),
                    }
                }
            }
        } else if parts.len() == 4 {
            // A - Active
            // I - Inactive
            // F - Fullscreen
            // o - Override
            // variants for 4 parts: AoIo, AoIF, AIoF, AIFo
            let opacity1 = parts[0].parse::<f64>().unwrap_or_default();
            match parts[1].trim().to_lowercase().as_str() {
                "override" => match parts[3].trim().to_lowercase().as_str() {
                    "override" => Ok(HyprOpacity::ActiveAndInactive(
                        opacity1,
                        true,
                        parts[2].parse::<f64>().unwrap_or_default(),
                        false,
                    )),
                    opacity3 => Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                        opacity1,
                        true,
                        parts[2].parse::<f64>().unwrap_or_default(),
                        false,
                        opacity3.parse::<f64>().unwrap_or_default(),
                        false,
                    )),
                },
                opacity2 => {
                    let opacity2 = opacity2.parse::<f64>().unwrap_or_default();
                    match parts[2].trim().to_lowercase().as_str() {
                        "override" => Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                            opacity1,
                            false,
                            opacity2,
                            true,
                            parts[3].parse::<f64>().unwrap_or_default(),
                            false,
                        )),
                        opacity3 => Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                            opacity1,
                            false,
                            opacity2,
                            false,
                            opacity3.parse::<f64>().unwrap_or_default(),
                            true,
                        )),
                    }
                }
            }
        } else if parts.len() == 5 {
            // A - Active
            // I - Inactive
            // F - Fullscreen
            // o - Override
            // variants for 5 parts: AoIoF, AoIFo, AIoFo
            let opacity1 = parts[0].parse::<f64>().unwrap_or_default();
            match parts[1].trim().to_lowercase().as_str() {
                "override" => match parts[3].trim().to_lowercase().as_str() {
                    "override" => Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                        opacity1,
                        true,
                        parts[2].parse::<f64>().unwrap_or_default(),
                        true,
                        parts[4].parse::<f64>().unwrap_or_default(),
                        false,
                    )),
                    opacity3 => Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                        opacity1,
                        true,
                        parts[2].parse::<f64>().unwrap_or_default(),
                        false,
                        opacity3.parse::<f64>().unwrap_or_default(),
                        true,
                    )),
                },
                opacity2 => Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                    opacity1,
                    false,
                    opacity2.parse::<f64>().unwrap_or_default(),
                    true,
                    parts[3].parse::<f64>().unwrap_or_default(),
                    true,
                )),
            }
        } else if parts.len() == 6 {
            // A - Active
            // I - Inactive
            // F - Fullscreen
            // o - Override
            // variants for 6 parts: AoIoFo
            Ok(HyprOpacity::ActiveAndInactiveAndFullscreen(
                parts[0].parse::<f64>().unwrap_or_default(),
                true,
                parts[2].parse::<f64>().unwrap_or_default(),
                true,
                parts[4].parse::<f64>().unwrap_or_default(),
                true,
            ))
        } else {
            Err(())
        }
    }
}

impl Display for HyprOpacity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyprOpacity::Overall(opacity, false) => write!(f, "{}", opacity),
            HyprOpacity::Overall(opacity, true) => write!(f, "{} override", opacity),
            HyprOpacity::ActiveAndInactive(opacity1, false, opacity2, false) => {
                write!(f, "{} {}", opacity1, opacity2)
            }
            HyprOpacity::ActiveAndInactive(opacity1, true, opacity2, false) => {
                write!(f, "{} override {}", opacity1, opacity2)
            }
            HyprOpacity::ActiveAndInactive(opacity1, false, opacity2, true) => {
                write!(f, "{} {} override", opacity1, opacity2)
            }
            HyprOpacity::ActiveAndInactive(opacity1, true, opacity2, true) => {
                write!(f, "{} override {} override", opacity1, opacity2)
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                false,
                opacity2,
                false,
                opacity3,
                false,
            ) => {
                write!(f, "{} {} {}", opacity1, opacity2, opacity3)
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                true,
                opacity2,
                false,
                opacity3,
                false,
            ) => {
                write!(f, "{} override {} {}", opacity1, opacity2, opacity3)
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                false,
                opacity2,
                true,
                opacity3,
                false,
            ) => {
                write!(f, "{} {} override {}", opacity1, opacity2, opacity3)
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                true,
                opacity2,
                true,
                opacity3,
                false,
            ) => {
                write!(
                    f,
                    "{} override {} override {}",
                    opacity1, opacity2, opacity3
                )
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                false,
                opacity2,
                false,
                opacity3,
                true,
            ) => {
                write!(f, "{} {} {} override", opacity1, opacity2, opacity3)
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                true,
                opacity2,
                false,
                opacity3,
                true,
            ) => {
                write!(
                    f,
                    "{} override {} {} override",
                    opacity1, opacity2, opacity3
                )
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                false,
                opacity2,
                true,
                opacity3,
                true,
            ) => {
                write!(
                    f,
                    "{} {} override {} override",
                    opacity1, opacity2, opacity3
                )
            }
            HyprOpacity::ActiveAndInactiveAndFullscreen(
                opacity1,
                true,
                opacity2,
                true,
                opacity3,
                true,
            ) => {
                write!(
                    f,
                    "{} override {} override {} override",
                    opacity1, opacity2, opacity3
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Default)]
pub enum TagToggleState {
    Set,
    Unset,
    #[default]
    Toggle,
}

impl FromStr for TagToggleState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "+" => Ok(Self::Set),
            "-" => Ok(Self::Unset),
            "" => Ok(Self::Toggle),
            _ => Err(()),
        }
    }
}

impl Display for TagToggleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagToggleState::Set => write!(f, "+"),
            TagToggleState::Unset => write!(f, "-"),
            TagToggleState::Toggle => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WindowRuleDiscriminant))]
#[derive(Default)]
pub enum WindowRule {
    #[default]
    Float,
    Tile,
    Fullscreen,
    Maximize,
    PersistentSize,
    FullscreenState(FullscreenState, FullscreenState),
    Move(HyprCoord),
    Size(HyprSize),
    Center,
    CenterWithRespectToMonitorReservedArea,
    Pseudo,
    Monitor(IdOrName),
    Workspace(WorkspaceTarget),
    NoInitialFocus,
    Pin,
    Unset,
    NoMaxSize,
    StayFocused,
    Group(Vec<WindowGroupOption>),
    SuppressEvent(HashSet<WindowEvent>),
    Content(ContentType),
    NoCloseFor(u32),
    Animation(AnimationStyle),
    BorderColor(BorderColor),
    IdleIngibit(IdleIngibitMode),
    Opacity(HyprOpacity),
    Tag(TagToggleState, String),
    MaxSize(u32, u32),
    MinSize(u32, u32),
}

impl HasDiscriminant for WindowRule {
    type Discriminant = WindowRuleDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Float => Self::Float,
            Self::Discriminant::Tile => Self::Tile,
            Self::Discriminant::Fullscreen => Self::Fullscreen,
            Self::Discriminant::Maximize => Self::Maximize,
            Self::Discriminant::PersistentSize => Self::PersistentSize,
            Self::Discriminant::FullscreenState => {
                Self::FullscreenState(FullscreenState::None, FullscreenState::None)
            }
            Self::Discriminant::Move => Self::Move(HyprCoord::default()),
            Self::Discriminant::Size => Self::Size(HyprSize::default()),
            Self::Discriminant::Center => Self::Center,
            Self::Discriminant::CenterWithRespectToMonitorReservedArea => {
                Self::CenterWithRespectToMonitorReservedArea
            }
            Self::Discriminant::Pseudo => Self::Pseudo,
            Self::Discriminant::Monitor => Self::Monitor(IdOrName::default()),
            Self::Discriminant::Workspace => Self::Workspace(WorkspaceTarget::default()),
            Self::Discriminant::NoInitialFocus => Self::NoInitialFocus,
            Self::Discriminant::Pin => Self::Pin,
            Self::Discriminant::Unset => Self::Unset,
            Self::Discriminant::NoMaxSize => Self::NoMaxSize,
            Self::Discriminant::StayFocused => Self::StayFocused,
            Self::Discriminant::Group => Self::Group(vec![WindowGroupOption::default()]),
            Self::Discriminant::SuppressEvent => {
                Self::SuppressEvent([WindowEvent::default()].into_iter().collect())
            }
            Self::Discriminant::Content => Self::Content(ContentType::default()),
            Self::Discriminant::NoCloseFor => Self::NoCloseFor(0),
            Self::Discriminant::Animation => Self::Animation(AnimationStyle::default()),
            Self::Discriminant::BorderColor => Self::BorderColor(BorderColor::default()),
            Self::Discriminant::IdleIngibit => Self::IdleIngibit(IdleIngibitMode::default()),
            Self::Discriminant::Opacity => Self::Opacity(HyprOpacity::default()),
            Self::Discriminant::Tag => Self::Tag(TagToggleState::Toggle, "".to_string()),
            Self::Discriminant::MaxSize => Self::MaxSize(0, 0),
            Self::Discriminant::MinSize => Self::MinSize(0, 0),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Float => Self::Float,
            Self::Discriminant::Tile => Self::Tile,
            Self::Discriminant::Fullscreen => Self::Fullscreen,
            Self::Discriminant::Maximize => Self::Maximize,
            Self::Discriminant::PersistentSize => Self::PersistentSize,
            Self::Discriminant::FullscreenState => {
                let (internal, client) = str.split_once(' ').unwrap_or((str, ""));
                Self::FullscreenState(
                    internal.parse().unwrap_or_default(),
                    client.parse().unwrap_or_default(),
                )
            }
            Self::Discriminant::Move => Self::Move(str.parse().unwrap_or_default()),
            Self::Discriminant::Size => Self::Size(str.parse().unwrap_or_default()),
            Self::Discriminant::Center => Self::Center,
            Self::Discriminant::CenterWithRespectToMonitorReservedArea => {
                Self::CenterWithRespectToMonitorReservedArea
            }
            Self::Discriminant::Pseudo => Self::Pseudo,
            Self::Discriminant::Monitor => Self::Monitor(str.parse().unwrap_or_default()),
            Self::Discriminant::Workspace => Self::Workspace(str.parse().unwrap_or_default()),
            Self::Discriminant::NoInitialFocus => Self::NoInitialFocus,
            Self::Discriminant::Pin => Self::Pin,
            Self::Discriminant::Unset => Self::Unset,
            Self::Discriminant::NoMaxSize => Self::NoMaxSize,
            Self::Discriminant::StayFocused => Self::StayFocused,
            Self::Discriminant::Group => Self::Group(
                str.split(' ')
                    .map(|s| s.parse().unwrap_or_default())
                    .collect(),
            ),
            Self::Discriminant::SuppressEvent => Self::SuppressEvent(
                str.split(' ')
                    .map(|s| s.parse().unwrap_or_default())
                    .collect(),
            ),
            Self::Discriminant::Content => Self::Content(str.parse().unwrap_or_default()),
            Self::Discriminant::NoCloseFor => Self::NoCloseFor(str.parse().unwrap_or_default()),
            Self::Discriminant::Animation => Self::Animation(str.parse().unwrap_or_default()),
            Self::Discriminant::BorderColor => Self::BorderColor(str.parse().unwrap_or_default()),
            Self::Discriminant::IdleIngibit => Self::IdleIngibit(str.parse().unwrap_or_default()),
            Self::Discriminant::Opacity => Self::Opacity(str.parse().unwrap_or_default()),
            Self::Discriminant::Tag => {
                if let Some(stripped) = str.strip_prefix('+') {
                    Self::Tag(TagToggleState::Set, stripped.trim().to_string())
                } else if let Some(stripped) = str.strip_prefix('-') {
                    Self::Tag(TagToggleState::Unset, stripped.trim().to_string())
                } else {
                    Self::Tag(TagToggleState::Toggle, str.trim().to_string())
                }
            }
            Self::Discriminant::MaxSize => {
                let (width, height) = str.split_once(' ').unwrap_or((str, ""));
                Self::MaxSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                )
            }
            Self::Discriminant::MinSize => {
                let (width, height) = str.split_once(' ').unwrap_or((str, ""));
                Self::MinSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                )
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            WindowRule::Float => None,
            WindowRule::Tile => None,
            WindowRule::Fullscreen => None,
            WindowRule::Maximize => None,
            WindowRule::PersistentSize => None,
            WindowRule::FullscreenState(internal, client) => {
                Some(format!("{} {}", internal.to_num(), client.to_num()))
            }
            WindowRule::Move(coord) => Some(coord.to_string()),
            WindowRule::Size(size) => Some(size.to_string()),
            WindowRule::Center => None,
            WindowRule::CenterWithRespectToMonitorReservedArea => None,
            WindowRule::Pseudo => None,
            WindowRule::Monitor(target) => Some(target.to_string()),
            WindowRule::Workspace(target) => Some(target.to_string()),
            WindowRule::NoInitialFocus => None,
            WindowRule::Pin => None,
            WindowRule::Unset => None,
            WindowRule::NoMaxSize => None,
            WindowRule::StayFocused => None,
            WindowRule::Group(group) => Some(join_with_separator(group, " ")),
            WindowRule::SuppressEvent(events) => Some(join_with_separator(events, " ")),
            WindowRule::Content(content) => Some(content.to_string()),
            WindowRule::NoCloseFor(duration) => Some(duration.to_string()),
            WindowRule::Animation(animation) => Some(animation.to_string()),
            WindowRule::BorderColor(color) => Some(color.to_string()),
            WindowRule::IdleIngibit(mode) => Some(mode.to_string()),
            WindowRule::Opacity(opacity) => Some(opacity.to_string()),
            WindowRule::Tag(toggle_state, tag) => Some(match toggle_state {
                TagToggleState::Set => format!("+{}", tag),
                TagToggleState::Unset => format!("-{}", tag),
                TagToggleState::Toggle => tag.clone(),
            }),
            WindowRule::MaxSize(width, height) => Some(format!("{} {}", width, height)),
            WindowRule::MinSize(width, height) => Some(format!("{} {}", width, height)),
        }
    }

    fn custom_split(discriminant: Self::Discriminant) -> Option<fn(&str) -> Vec<&str>> {
        match discriminant {
            Self::Discriminant::Tag => Some(|s| {
                if let Some(stripped) = s.strip_prefix("+") {
                    vec!["+", stripped]
                } else if let Some(stripped) = s.strip_prefix("-") {
                    vec!["-", stripped]
                } else {
                    vec!["", s]
                }
            }),
            _ => None,
        }
    }
}

impl FromStr for WindowRule {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }
        let (part1, part2) = s.split_once(' ').unwrap_or((s, ""));
        match part1.trim().to_lowercase().as_str() {
            "float" => Ok(WindowRule::Float),
            "tile" => Ok(WindowRule::Tile),
            "fullscreen" => Ok(WindowRule::Fullscreen),
            "maximize" => Ok(WindowRule::Maximize),
            "persistent" => Ok(WindowRule::PersistentSize),
            "fullscreenstate" => {
                let (internal, client) = part2.split_once(' ').unwrap_or((part2, ""));
                Ok(WindowRule::FullscreenState(
                    FullscreenState::from_num(internal.parse().unwrap_or_default()),
                    FullscreenState::from_num(client.parse().unwrap_or_default()),
                ))
            }
            "move" => Ok(WindowRule::Move(part2.parse().unwrap_or_default())),
            "size" => Ok(WindowRule::Size(part2.parse().unwrap_or_default())),
            "center" => match part2.trim().to_lowercase().as_str() {
                "1" => Ok(WindowRule::CenterWithRespectToMonitorReservedArea),
                _ => Ok(WindowRule::Center),
            },
            "pseudo" => Ok(WindowRule::Pseudo),
            "monitor" => Ok(WindowRule::Monitor(part2.parse().unwrap_or_default())),
            "workspace" => Ok(WindowRule::Workspace(part2.parse().unwrap_or_default())),
            "noinitialfocus" => Ok(WindowRule::NoInitialFocus),
            "pin" => Ok(WindowRule::Pin),
            "unset" => Ok(WindowRule::Unset),
            "nomaxsize" => Ok(WindowRule::NoMaxSize),
            "stayfocused" => Ok(WindowRule::StayFocused),
            "group" => Ok(WindowRule::Group(
                part2
                    .split(' ')
                    .map(|s| WindowGroupOption::from_str(s).unwrap_or_default())
                    .collect(),
            )),
            "suppress" => Ok(WindowRule::SuppressEvent(
                part2
                    .split(' ')
                    .map(|s| WindowEvent::from_str(s).unwrap_or_default())
                    .collect(),
            )),
            "content" => Ok(WindowRule::Content(part2.parse().unwrap_or_default())),
            "noclosefor" => Ok(WindowRule::NoCloseFor(part2.parse().unwrap_or_default())),
            "animation" => Ok(WindowRule::Animation(part2.parse().unwrap_or_default())),
            "bordercolor" => Ok(WindowRule::BorderColor(part2.parse().unwrap_or_default())),
            "idleingibit" => Ok(WindowRule::IdleIngibit(part2.parse().unwrap_or_default())),
            "opacity" => Ok(WindowRule::Opacity(part2.parse().unwrap_or_default())),
            "tag" => {
                if let Some(stripped) = part2.strip_prefix("+") {
                    Ok(WindowRule::Tag(
                        TagToggleState::Set,
                        stripped.trim().to_string(),
                    ))
                } else if let Some(stripped) = part2.strip_prefix("-") {
                    Ok(WindowRule::Tag(
                        TagToggleState::Unset,
                        stripped.trim().to_string(),
                    ))
                } else {
                    Ok(WindowRule::Tag(
                        TagToggleState::Toggle,
                        part2.trim().to_string(),
                    ))
                }
            }
            "maxsize" => {
                let (width, height) = part2.split_once(' ').unwrap_or((part2, ""));
                Ok(WindowRule::MaxSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                ))
            }
            "minsize" => {
                let (width, height) = part2.split_once(' ').unwrap_or((part2, ""));
                Ok(WindowRule::MinSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                ))
            }
            _ => Err(()),
        }
    }
}

impl Display for WindowRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowRule::Float => write!(f, "float"),
            WindowRule::Tile => write!(f, "tile"),
            WindowRule::Fullscreen => write!(f, "fullscreen"),
            WindowRule::Maximize => write!(f, "maximize"),
            WindowRule::PersistentSize => write!(f, "persistent"),
            WindowRule::FullscreenState(internal, client) => write!(
                f,
                "fullscreenstate {} {}",
                internal.to_num(),
                client.to_num()
            ),
            WindowRule::Move(move_) => write!(f, "move {}", move_),
            WindowRule::Size(size) => write!(f, "size {}", size),
            WindowRule::Center => write!(f, "center"),
            WindowRule::CenterWithRespectToMonitorReservedArea => write!(f, "center 1"),
            WindowRule::Pseudo => write!(f, "pseudo"),
            WindowRule::Monitor(monitor) => write!(f, "monitor {}", monitor),
            WindowRule::Workspace(workspace) => write!(f, "workspace {}", workspace),
            WindowRule::NoInitialFocus => write!(f, "noinitialfocus"),
            WindowRule::Pin => write!(f, "pin"),
            WindowRule::Unset => write!(f, "unset"),
            WindowRule::NoMaxSize => write!(f, "nomaxsize"),
            WindowRule::StayFocused => write!(f, "stayfocused"),
            WindowRule::Group(group) => write!(f, "group {}", join_with_separator(group, " ")),
            WindowRule::SuppressEvent(suppress) => {
                write!(f, "suppress {}", join_with_separator(suppress, " "))
            }
            WindowRule::Content(content) => write!(f, "content {}", content),
            WindowRule::NoCloseFor(no_close_for) => write!(f, "noclosefor {}", no_close_for),
            WindowRule::Animation(animation) => write!(f, "animation {}", animation),
            WindowRule::BorderColor(border_color) => write!(f, "bordercolor {}", border_color),
            WindowRule::IdleIngibit(idle_ingibit) => write!(f, "idleingibit {}", idle_ingibit),
            WindowRule::Opacity(opacity) => write!(f, "opacity {}", opacity),
            WindowRule::Tag(TagToggleState::Set, tag) => write!(f, "tag +{}", tag),
            WindowRule::Tag(TagToggleState::Unset, tag) => write!(f, "tag -{}", tag),
            WindowRule::Tag(TagToggleState::Toggle, tag) => write!(f, "tag {}", tag),
            WindowRule::MaxSize(width, height) => write!(f, "maxsize {} {}", width, height),
            WindowRule::MinSize(width, height) => write!(f, "minsize {} {}", width, height),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum KeyState {
    #[default]
    Down,
    Repeat,
    Up,
}

impl FromStr for KeyState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "down" => Ok(KeyState::Down),
            "repeat" => Ok(KeyState::Repeat),
            "up" => Ok(KeyState::Up),
            _ => Err(()),
        }
    }
}

impl Display for KeyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyState::Down => write!(f, "down"),
            KeyState::Repeat => write!(f, "repeat"),
            KeyState::Up => write!(f, "up"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Default)]
pub enum DispatcherFullscreenState {
    Current,
    #[default]
    None,
    Maximize,
    Fullscreen,
    MaximizeAndFullscreen,
}

impl DispatcherFullscreenState {
    pub fn from_num(num: i8) -> Self {
        match num {
            -1 => DispatcherFullscreenState::Current,
            0 => DispatcherFullscreenState::None,
            1 => DispatcherFullscreenState::Maximize,
            2 => DispatcherFullscreenState::Fullscreen,
            3 => DispatcherFullscreenState::MaximizeAndFullscreen,
            _ => DispatcherFullscreenState::Current,
        }
    }

    pub fn to_num(self) -> i8 {
        match self {
            DispatcherFullscreenState::Current => -1,
            DispatcherFullscreenState::None => 0,
            DispatcherFullscreenState::Maximize => 1,
            DispatcherFullscreenState::Fullscreen => 2,
            DispatcherFullscreenState::MaximizeAndFullscreen => 3,
        }
    }
}

impl FromStr for DispatcherFullscreenState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_num(s.trim().parse().unwrap_or(-1)))
    }
}

impl Display for DispatcherFullscreenState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_num())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(MoveDirectionDiscriminant))]
pub enum MoveDirection {
    Direction(Direction),
    DirectionSilent(Direction),
    Monitor(MonitorTarget),
    MonitorSilent(MonitorTarget),
}

impl HasDiscriminant for MoveDirection {
    type Discriminant = MoveDirectionDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Direction => Self::Direction(Direction::default()),
            Self::Discriminant::DirectionSilent => Self::DirectionSilent(Direction::default()),
            Self::Discriminant::Monitor => Self::Monitor(MonitorTarget::default()),
            Self::Discriminant::MonitorSilent => Self::MonitorSilent(MonitorTarget::default()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Direction => Self::Direction(str.parse().unwrap_or_default()),
            Self::Discriminant::DirectionSilent => {
                Self::DirectionSilent(str.parse().unwrap_or_default())
            }
            Self::Discriminant::Monitor => Self::Monitor(str.parse().unwrap_or_default()),
            Self::Discriminant::MonitorSilent => {
                Self::MonitorSilent(str.parse().unwrap_or_default())
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            MoveDirection::Direction(direction) => Some(direction.to_string()),
            MoveDirection::DirectionSilent(direction) => Some(direction.to_string()),
            MoveDirection::Monitor(monitor) => Some(monitor.to_string()),
            MoveDirection::MonitorSilent(monitor) => Some(monitor.to_string()),
        }
    }
}

impl Default for MoveDirection {
    fn default() -> Self {
        MoveDirection::Direction(Direction::default())
    }
}

impl FromStr for MoveDirection {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        let (part1, part2) = s.rsplit_once(' ').unwrap_or((s, ""));

        if let Some(stripped) = part1.strip_prefix("mon:") {
            let monitor = MonitorTarget::from_str(stripped.trim()).unwrap_or_default();
            match part2 {
                "silent" => Ok(MoveDirection::MonitorSilent(monitor)),
                _ => Ok(MoveDirection::Monitor(monitor)),
            }
        } else {
            let direction = Direction::from_str(part1).unwrap_or_default();
            match part2 {
                "silent" => Ok(MoveDirection::DirectionSilent(direction)),
                _ => Ok(MoveDirection::Direction(direction)),
            }
        }
    }
}

impl Display for MoveDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveDirection::Direction(direction) => {
                write!(f, "{}", direction)
            }
            MoveDirection::DirectionSilent(direction) => {
                write!(f, "{} silent", direction)
            }
            MoveDirection::Monitor(monitor) => {
                write!(f, "mon:{}", monitor)
            }
            MoveDirection::MonitorSilent(monitor) => {
                write!(f, "mon:{} silent", monitor)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(SwapDirectionDiscriminant))]
pub enum SwapDirection {
    Direction(Direction),
    Window(WindowTarget),
}

impl HasDiscriminant for SwapDirection {
    type Discriminant = SwapDirectionDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Direction => Self::Direction(Direction::default()),
            Self::Discriminant::Window => Self::Window(WindowTarget::default()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Direction => Self::Direction(str.parse().unwrap_or_default()),
            Self::Discriminant::Window => Self::Window(str.parse().unwrap_or_default()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            SwapDirection::Direction(direction) => Some(direction.to_string()),
            SwapDirection::Window(window) => Some(window.to_string()),
        }
    }
}

impl Default for SwapDirection {
    fn default() -> Self {
        SwapDirection::Direction(Direction::default())
    }
}

impl FromStr for SwapDirection {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        if let Ok(direction) = Direction::from_str(s) {
            Ok(SwapDirection::Direction(direction))
        } else if let Ok(window) = WindowTarget::from_str(s) {
            Ok(SwapDirection::Window(window))
        } else {
            Err(())
        }
    }
}

impl Display for SwapDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwapDirection::Direction(direction) => {
                write!(f, "{}", direction)
            }
            SwapDirection::Window(window) => {
                write!(f, "{}", window)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CycleNext {
    pub is_prev: bool,
    pub is_tiled: bool,
    pub is_floating: bool,
    pub is_visible: bool,
    pub is_hist: bool,
}

impl FromStr for CycleNext {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        let parts = s.split_whitespace().collect::<Vec<_>>();

        let mut is_next = false;
        let mut is_prev = false;
        let mut is_tiled = false;
        let mut is_floating = false;
        let mut is_visible = false;
        let mut is_hist = false;

        for part in parts {
            match part {
                "next" => {
                    if !(is_next || is_prev) {
                        is_next = true;
                    }
                }
                "prev" => {
                    if !(is_next || is_prev) {
                        is_prev = true;
                    }
                }
                "tiled" => {
                    if !(is_tiled || is_floating) {
                        is_tiled = true;
                    }
                }
                "floating" => {
                    if !(is_tiled || is_floating) {
                        is_floating = true;
                    }
                }
                "visible" => {
                    if !(is_visible) {
                        is_visible = true;
                    }
                }
                "hist" => {
                    if !(is_hist) {
                        is_hist = true;
                    }
                }
                _ => {}
            }
        }

        Ok(CycleNext {
            is_prev,
            is_tiled,
            is_floating,
            is_visible,
            is_hist,
        })
    }
}

impl Display for CycleNext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        if self.is_visible {
            result.push_str("visible ");
        }

        if self.is_prev {
            result.push_str("prev ");
        } else {
            result.push_str("next ");
        }

        if self.is_tiled {
            result.push_str("tiled ");
        } else if self.is_floating {
            result.push_str("floating ");
        }

        if self.is_hist {
            result.push_str("hist ");
        }

        write!(f, "{}", result.trim())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum SwapNext {
    #[default]
    Next,
    Prev,
}

impl FromStr for SwapNext {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "prev" => Ok(SwapNext::Prev),
            _ => Ok(SwapNext::Next),
        }
    }
}

impl Display for SwapNext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwapNext::Next => write!(f, ""),
            SwapNext::Prev => write!(f, "prev"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(ChangeGroupActiveDiscriminant))]
pub enum ChangeGroupActive {
    Back,
    #[default]
    Forward,
    Index(u32),
}

impl HasDiscriminant for ChangeGroupActive {
    type Discriminant = ChangeGroupActiveDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Back => Self::Back,
            Self::Discriminant::Forward => Self::Forward,
            Self::Discriminant::Index => Self::Index(0),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Back => Self::Back,
            Self::Discriminant::Forward => Self::Forward,
            Self::Discriminant::Index => Self::Index(str.parse().unwrap_or_default()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            ChangeGroupActive::Back => None,
            ChangeGroupActive::Forward => None,
            ChangeGroupActive::Index(index) => Some(index.to_string()),
        }
    }
}

impl FromStr for ChangeGroupActive {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        match s {
            "b" => Ok(ChangeGroupActive::Back),
            "f" => Ok(ChangeGroupActive::Forward),
            index => Ok(ChangeGroupActive::Index(
                index.parse::<u32>().unwrap_or_default().saturating_sub(1),
            )),
        }
    }
}

impl Display for ChangeGroupActive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeGroupActive::Back => write!(f, "b"),
            ChangeGroupActive::Forward => write!(f, "f"),
            ChangeGroupActive::Index(index) => write!(f, "{}", index.saturating_add(1)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum SetPropToggleState {
    Off,
    On,
    #[default]
    Toggle,
    Unset,
}

impl FromStr for SetPropToggleState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        match parse_bool(&s.to_lowercase()) {
            Some(true) => Ok(SetPropToggleState::On),
            Some(false) => Ok(SetPropToggleState::Off),
            None => match s.to_lowercase().as_str() {
                "toggle" => Ok(SetPropToggleState::Toggle),
                "unset" => Ok(SetPropToggleState::Unset),
                _ => Err(()),
            },
        }
    }
}

impl Display for SetPropToggleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetPropToggleState::Off => write!(f, "0"),
            SetPropToggleState::On => write!(f, "1"),
            SetPropToggleState::Toggle => write!(f, "toggle"),
            SetPropToggleState::Unset => write!(f, "unset"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HyprGradient {
    pub colors: Vec<HyprColor>,
    pub angle: Option<Angle>,
}

impl Default for HyprGradient {
    fn default() -> Self {
        HyprGradient {
            colors: vec![HyprColor::default(), HyprColor::default()],
            angle: None,
        }
    }
}

impl FromStr for HyprGradient {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        let mut colors = Vec::new();

        let parts = s.split_whitespace().collect::<Vec<_>>();

        for part in parts {
            if let Ok(angle) = Angle::from_str(part) {
                return Ok(HyprGradient {
                    colors,
                    angle: Some(angle),
                });
            } else if let Ok(color) = HyprColor::from_str(part) {
                colors.push(color);
            }
        }

        if colors.is_empty() {
            colors.push(HyprColor::default());
            colors.push(HyprColor::default());
        } else if colors.len() == 1 {
            colors.push(colors[0]);
        }

        Ok(HyprGradient {
            colors,
            angle: None,
        })
    }
}

impl Display for HyprGradient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.angle {
            Some(angle) => write!(f, "{} {}", join_with_separator(&self.colors, " "), angle),
            None => write!(f, "{}", join_with_separator(&self.colors, " ")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(SetPropDiscriminant))]
pub enum SetProp {
    Alpha(f64),
    AlphaOverride(bool),
    AlphaInactive(f64),
    AlphaInactiveOverride(bool),
    AlphaFullscreen(f64),
    AlphaFullscreenOverride(bool),
    AnimationStyle(String),
    ActiveBorderColor(Option<HyprGradient>),
    InactiveBorderColor(Option<HyprGradient>),
    Animation(AnimationStyle),
    BorderColor(BorderColor),
    IdleIngibit(IdleIngibitMode),
    Opacity(HyprOpacity),
    Tag(TagToggleState, String),
    MaxSize(u32, u32),
    MinSize(u32, u32),
    BorderSize(u32),
    Rounding(u32),
    RoundingPower(f64),
    AllowsInput(SetPropToggleState),
    DimAround(SetPropToggleState),
    Decorate(SetPropToggleState),
    FocusOnActivate(SetPropToggleState),
    KeepAspectRatio(SetPropToggleState),
    NearestNeighbor(SetPropToggleState),
    NoAnim(SetPropToggleState),
    NoBlur(SetPropToggleState),
    NoBorder(SetPropToggleState),
    NoDim(SetPropToggleState),
    NoFocus(SetPropToggleState),
    NoFollowMouse(SetPropToggleState),
    NoMaxSize(SetPropToggleState),
    NoRounding(SetPropToggleState),
    NoShadow(SetPropToggleState),
    NoShortcutsInhibit(SetPropToggleState),
    Opaque(SetPropToggleState),
    ForceRGBX(SetPropToggleState),
    SyncFullscreen(SetPropToggleState),
    Immediate(SetPropToggleState),
    Xray(SetPropToggleState),
    RenderUnfocused,
    ScrollMouse(f64),
    ScrollTouchpad(f64),
    NoScreenShare(SetPropToggleState),
    NoVRR(SetPropToggleState),
}

impl HasDiscriminant for SetProp {
    type Discriminant = SetPropDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Alpha => Self::Alpha(1.0),
            Self::Discriminant::AlphaOverride => Self::AlphaOverride(false),
            Self::Discriminant::AlphaInactive => Self::AlphaInactive(1.0),
            Self::Discriminant::AlphaInactiveOverride => Self::AlphaInactiveOverride(false),
            Self::Discriminant::AlphaFullscreen => Self::AlphaFullscreen(1.0),
            Self::Discriminant::AlphaFullscreenOverride => Self::AlphaFullscreenOverride(false),
            Self::Discriminant::AnimationStyle => Self::AnimationStyle("".to_string()),
            Self::Discriminant::ActiveBorderColor => Self::ActiveBorderColor(None),
            Self::Discriminant::InactiveBorderColor => Self::InactiveBorderColor(None),
            Self::Discriminant::Animation => Self::Animation(AnimationStyle::default()),
            Self::Discriminant::BorderColor => Self::BorderColor(BorderColor::default()),
            Self::Discriminant::IdleIngibit => Self::IdleIngibit(IdleIngibitMode::default()),
            Self::Discriminant::Opacity => Self::Opacity(HyprOpacity::default()),
            Self::Discriminant::Tag => Self::Tag(TagToggleState::Toggle, "".to_string()),
            Self::Discriminant::MaxSize => Self::MaxSize(0, 0),
            Self::Discriminant::MinSize => Self::MinSize(0, 0),
            Self::Discriminant::BorderSize => Self::BorderSize(0),
            Self::Discriminant::Rounding => Self::Rounding(0),
            Self::Discriminant::RoundingPower => Self::RoundingPower(0.0),
            Self::Discriminant::AllowsInput => Self::AllowsInput(SetPropToggleState::default()),
            Self::Discriminant::DimAround => Self::DimAround(SetPropToggleState::default()),
            Self::Discriminant::Decorate => Self::Decorate(SetPropToggleState::default()),
            Self::Discriminant::FocusOnActivate => {
                Self::FocusOnActivate(SetPropToggleState::default())
            }
            Self::Discriminant::KeepAspectRatio => {
                Self::KeepAspectRatio(SetPropToggleState::default())
            }
            Self::Discriminant::NearestNeighbor => {
                Self::NearestNeighbor(SetPropToggleState::default())
            }
            Self::Discriminant::NoAnim => Self::NoAnim(SetPropToggleState::default()),
            Self::Discriminant::NoBlur => Self::NoBlur(SetPropToggleState::default()),
            Self::Discriminant::NoBorder => Self::NoBorder(SetPropToggleState::default()),
            Self::Discriminant::NoDim => Self::NoDim(SetPropToggleState::default()),
            Self::Discriminant::NoFocus => Self::NoFocus(SetPropToggleState::default()),
            Self::Discriminant::NoFollowMouse => Self::NoFollowMouse(SetPropToggleState::default()),
            Self::Discriminant::NoMaxSize => Self::NoMaxSize(SetPropToggleState::default()),
            Self::Discriminant::NoRounding => Self::NoRounding(SetPropToggleState::default()),
            Self::Discriminant::NoShadow => Self::NoShadow(SetPropToggleState::default()),
            Self::Discriminant::NoShortcutsInhibit => {
                Self::NoShortcutsInhibit(SetPropToggleState::default())
            }
            Self::Discriminant::Opaque => Self::Opaque(SetPropToggleState::default()),
            Self::Discriminant::ForceRGBX => Self::ForceRGBX(SetPropToggleState::default()),
            Self::Discriminant::SyncFullscreen => {
                Self::SyncFullscreen(SetPropToggleState::default())
            }
            Self::Discriminant::Immediate => Self::Immediate(SetPropToggleState::default()),
            Self::Discriminant::Xray => Self::Xray(SetPropToggleState::default()),
            Self::Discriminant::RenderUnfocused => Self::RenderUnfocused,
            Self::Discriminant::ScrollMouse => Self::ScrollMouse(0.0),
            Self::Discriminant::ScrollTouchpad => Self::ScrollTouchpad(0.0),
            Self::Discriminant::NoScreenShare => Self::NoScreenShare(SetPropToggleState::default()),
            Self::Discriminant::NoVRR => Self::NoVRR(SetPropToggleState::default()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Alpha => Self::Alpha(str.parse().unwrap_or_default()),
            Self::Discriminant::AlphaOverride => {
                Self::AlphaOverride(str.parse().unwrap_or_default())
            }
            Self::Discriminant::AlphaInactive => {
                Self::AlphaInactive(str.parse().unwrap_or_default())
            }
            Self::Discriminant::AlphaInactiveOverride => {
                Self::AlphaInactiveOverride(str.parse().unwrap_or_default())
            }
            Self::Discriminant::AlphaFullscreen => {
                Self::AlphaFullscreen(str.parse().unwrap_or_default())
            }
            Self::Discriminant::AlphaFullscreenOverride => {
                Self::AlphaFullscreenOverride(str.parse().unwrap_or_default())
            }
            Self::Discriminant::AnimationStyle => Self::AnimationStyle(str.to_string()),
            Self::Discriminant::ActiveBorderColor => {
                if str == "-1" {
                    Self::ActiveBorderColor(None)
                } else {
                    Self::ActiveBorderColor(Some(str.parse().unwrap_or_default()))
                }
            }
            Self::Discriminant::InactiveBorderColor => {
                if str == "-1" {
                    Self::InactiveBorderColor(None)
                } else {
                    Self::InactiveBorderColor(Some(str.parse().unwrap_or_default()))
                }
            }
            Self::Discriminant::Animation => Self::Animation(str.parse().unwrap_or_default()),
            Self::Discriminant::BorderColor => Self::BorderColor(str.parse().unwrap_or_default()),
            Self::Discriminant::IdleIngibit => Self::IdleIngibit(str.parse().unwrap_or_default()),
            Self::Discriminant::Opacity => Self::Opacity(str.parse().unwrap_or_default()),
            Self::Discriminant::Tag => {
                if let Some(stripped) = str.strip_prefix('+') {
                    Self::Tag(TagToggleState::Set, stripped.trim().to_string())
                } else if let Some(stripped) = str.strip_prefix('-') {
                    Self::Tag(TagToggleState::Unset, stripped.trim().to_string())
                } else {
                    Self::Tag(TagToggleState::Toggle, str.trim().to_string())
                }
            }
            Self::Discriminant::MaxSize => {
                let (width, height) = str.split_once(' ').unwrap_or((str, "0"));
                Self::MaxSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                )
            }
            Self::Discriminant::MinSize => {
                let (width, height) = str.split_once(' ').unwrap_or((str, "0"));
                Self::MinSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                )
            }
            Self::Discriminant::BorderSize => Self::BorderSize(str.parse().unwrap_or_default()),
            Self::Discriminant::Rounding => Self::Rounding(str.parse().unwrap_or_default()),
            Self::Discriminant::RoundingPower => {
                Self::RoundingPower(str.parse().unwrap_or_default())
            }
            Self::Discriminant::AllowsInput => Self::AllowsInput(str.parse().unwrap_or_default()),
            Self::Discriminant::DimAround => Self::DimAround(str.parse().unwrap_or_default()),
            Self::Discriminant::Decorate => Self::Decorate(str.parse().unwrap_or_default()),
            Self::Discriminant::FocusOnActivate => {
                Self::FocusOnActivate(str.parse().unwrap_or_default())
            }
            Self::Discriminant::KeepAspectRatio => {
                Self::KeepAspectRatio(str.parse().unwrap_or_default())
            }
            Self::Discriminant::NearestNeighbor => {
                Self::NearestNeighbor(str.parse().unwrap_or_default())
            }
            Self::Discriminant::NoAnim => Self::NoAnim(str.parse().unwrap_or_default()),
            Self::Discriminant::NoBlur => Self::NoBlur(str.parse().unwrap_or_default()),
            Self::Discriminant::NoBorder => Self::NoBorder(str.parse().unwrap_or_default()),
            Self::Discriminant::NoDim => Self::NoDim(str.parse().unwrap_or_default()),
            Self::Discriminant::NoFocus => Self::NoFocus(str.parse().unwrap_or_default()),
            Self::Discriminant::NoFollowMouse => {
                Self::NoFollowMouse(str.parse().unwrap_or_default())
            }
            Self::Discriminant::NoMaxSize => Self::NoMaxSize(str.parse().unwrap_or_default()),
            Self::Discriminant::NoRounding => Self::NoRounding(str.parse().unwrap_or_default()),
            Self::Discriminant::NoShadow => Self::NoShadow(str.parse().unwrap_or_default()),
            Self::Discriminant::NoShortcutsInhibit => {
                Self::NoShortcutsInhibit(str.parse().unwrap_or_default())
            }
            Self::Discriminant::Opaque => Self::Opaque(str.parse().unwrap_or_default()),
            Self::Discriminant::ForceRGBX => Self::ForceRGBX(str.parse().unwrap_or_default()),
            Self::Discriminant::SyncFullscreen => {
                Self::SyncFullscreen(str.parse().unwrap_or_default())
            }
            Self::Discriminant::Immediate => Self::Immediate(str.parse().unwrap_or_default()),
            Self::Discriminant::Xray => Self::Xray(str.parse().unwrap_or_default()),
            Self::Discriminant::RenderUnfocused => Self::RenderUnfocused,
            Self::Discriminant::ScrollMouse => Self::ScrollMouse(str.parse().unwrap_or_default()),
            Self::Discriminant::ScrollTouchpad => {
                Self::ScrollTouchpad(str.parse().unwrap_or_default())
            }
            Self::Discriminant::NoScreenShare => {
                Self::NoScreenShare(str.parse().unwrap_or_default())
            }
            Self::Discriminant::NoVRR => Self::NoVRR(str.parse().unwrap_or_default()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            SetProp::Alpha(alpha) => Some(alpha.to_string()),
            SetProp::AlphaOverride(value) => Some(value.to_string()),
            SetProp::AlphaInactive(alpha) => Some(alpha.to_string()),
            SetProp::AlphaInactiveOverride(value) => Some(value.to_string()),
            SetProp::AlphaFullscreen(alpha) => Some(alpha.to_string()),
            SetProp::AlphaFullscreenOverride(value) => Some(value.to_string()),
            SetProp::AnimationStyle(style) => Some(style.clone()),
            SetProp::ActiveBorderColor(None) => Some("-1".to_string()),
            SetProp::ActiveBorderColor(Some(color)) => Some(color.to_string()),
            SetProp::InactiveBorderColor(None) => Some("-1".to_string()),
            SetProp::InactiveBorderColor(Some(color)) => Some(color.to_string()),
            SetProp::Animation(animation) => Some(animation.to_string()),
            SetProp::BorderColor(border_color) => Some(border_color.to_string()),
            SetProp::IdleIngibit(mode) => Some(mode.to_string()),
            SetProp::Opacity(opacity) => Some(opacity.to_string()),
            SetProp::Tag(TagToggleState::Set, tag) => Some(format!("+{}", tag)),
            SetProp::Tag(TagToggleState::Unset, tag) => Some(format!("-{}", tag)),
            SetProp::Tag(TagToggleState::Toggle, tag) => Some(tag.clone()),
            SetProp::MaxSize(width, height) => Some(format!("{} {}", width, height)),
            SetProp::MinSize(width, height) => Some(format!("{} {}", width, height)),
            SetProp::BorderSize(size) => Some(size.to_string()),
            SetProp::Rounding(size) => Some(size.to_string()),
            SetProp::RoundingPower(power) => Some(power.to_string()),
            SetProp::AllowsInput(mode) => Some(mode.to_string()),
            SetProp::DimAround(mode) => Some(mode.to_string()),
            SetProp::Decorate(mode) => Some(mode.to_string()),
            SetProp::FocusOnActivate(mode) => Some(mode.to_string()),
            SetProp::KeepAspectRatio(mode) => Some(mode.to_string()),
            SetProp::NearestNeighbor(mode) => Some(mode.to_string()),
            SetProp::NoAnim(mode) => Some(mode.to_string()),
            SetProp::NoBlur(mode) => Some(mode.to_string()),
            SetProp::NoBorder(mode) => Some(mode.to_string()),
            SetProp::NoDim(mode) => Some(mode.to_string()),
            SetProp::NoFocus(mode) => Some(mode.to_string()),
            SetProp::NoFollowMouse(mode) => Some(mode.to_string()),
            SetProp::NoMaxSize(mode) => Some(mode.to_string()),
            SetProp::NoRounding(mode) => Some(mode.to_string()),
            SetProp::NoShadow(mode) => Some(mode.to_string()),
            SetProp::NoShortcutsInhibit(mode) => Some(mode.to_string()),
            SetProp::Opaque(mode) => Some(mode.to_string()),
            SetProp::ForceRGBX(mode) => Some(mode.to_string()),
            SetProp::SyncFullscreen(mode) => Some(mode.to_string()),
            SetProp::Immediate(mode) => Some(mode.to_string()),
            SetProp::Xray(mode) => Some(mode.to_string()),
            SetProp::RenderUnfocused => None,
            SetProp::ScrollMouse(speed) => Some(speed.to_string()),
            SetProp::ScrollTouchpad(speed) => Some(speed.to_string()),
            SetProp::NoScreenShare(mode) => Some(mode.to_string()),
            SetProp::NoVRR(mode) => Some(mode.to_string()),
        }
    }

    fn custom_split(discriminant: Self::Discriminant) -> Option<fn(&str) -> Vec<&str>> {
        match discriminant {
            Self::Discriminant::Tag => Some(|s| {
                if let Some(stripped) = s.strip_prefix("+") {
                    vec!["+", stripped]
                } else if let Some(stripped) = s.strip_prefix("-") {
                    vec!["-", stripped]
                } else {
                    vec!["", s]
                }
            }),
            _ => None,
        }
    }
}

impl Default for SetProp {
    fn default() -> Self {
        SetProp::Alpha(1.0)
    }
}

impl FromStr for SetProp {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(());
        }

        let parts = s.split_whitespace().collect::<Vec<_>>();

        match parts[0] {
            "alpha" => {
                let alpha = parts.get(1).unwrap_or(&"").parse().unwrap_or(1.0);
                Ok(SetProp::Alpha(alpha))
            }
            "alphaoverride" => Ok(SetProp::AlphaOverride(
                parse_bool(&parts.get(1).unwrap_or(&"").to_lowercase()).unwrap_or(false),
            )),
            "alphainactive" => {
                let alpha = parts.get(1).unwrap_or(&"").parse().unwrap_or(1.0);
                Ok(SetProp::AlphaInactive(alpha))
            }
            "alphainactiveoverride" => Ok(SetProp::AlphaInactiveOverride(
                parse_bool(&parts.get(1).unwrap_or(&"").to_lowercase()).unwrap_or(false),
            )),
            "alphafullscreen" => {
                let alpha = parts.get(1).unwrap_or(&"").parse().unwrap_or(1.0);
                Ok(SetProp::AlphaFullscreen(alpha))
            }
            "alphafullscreenoverride" => Ok(SetProp::AlphaFullscreenOverride(
                parse_bool(&parts.get(1).unwrap_or(&"").to_lowercase()).unwrap_or(false),
            )),
            "animationstyle" => Ok(SetProp::AnimationStyle(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "activebordercolor" => {
                if parts.get(1) == Some(&"-1") {
                    Ok(SetProp::ActiveBorderColor(None))
                } else {
                    Ok(SetProp::ActiveBorderColor(Some(
                        parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
                    )))
                }
            }
            "inactivebordercolor" => {
                if parts.get(1) == Some(&"-1") {
                    Ok(SetProp::InactiveBorderColor(None))
                } else {
                    Ok(SetProp::InactiveBorderColor(Some(
                        parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
                    )))
                }
            }
            "animation" => Ok(SetProp::Animation(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "bordercolor" => Ok(SetProp::BorderColor(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "idleingibit" => Ok(SetProp::IdleIngibit(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "opacity" => Ok(SetProp::Opacity(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "tag" => {
                let part2 = parts.get(1).unwrap_or(&"");
                if let Some(stripped) = part2.strip_prefix("+") {
                    Ok(SetProp::Tag(
                        TagToggleState::Set,
                        stripped.trim().to_string(),
                    ))
                } else if let Some(stripped) = part2.strip_prefix("-") {
                    Ok(SetProp::Tag(
                        TagToggleState::Unset,
                        stripped.trim().to_string(),
                    ))
                } else {
                    Ok(SetProp::Tag(
                        TagToggleState::Toggle,
                        part2.trim().to_string(),
                    ))
                }
            }
            "maxsize" => {
                let width = parts.get(1).unwrap_or(&"");
                let height = parts.get(2).unwrap_or(&"");
                Ok(SetProp::MaxSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                ))
            }
            "minsize" => {
                let width = parts.get(1).unwrap_or(&"");
                let height = parts.get(2).unwrap_or(&"");
                Ok(SetProp::MinSize(
                    width.parse().unwrap_or_default(),
                    height.parse().unwrap_or_default(),
                ))
            }
            "bordersize" => Ok(SetProp::BorderSize(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "rounding" => Ok(SetProp::Rounding(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "roundingpower" => Ok(SetProp::RoundingPower(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "allowsinput" => Ok(SetProp::AllowsInput(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "dimaround" => Ok(SetProp::DimAround(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "decorate" => Ok(SetProp::Decorate(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "focusonactivate" => Ok(SetProp::FocusOnActivate(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "keepaspectratio" => Ok(SetProp::KeepAspectRatio(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "nearestneighbor" => Ok(SetProp::NearestNeighbor(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "noanim" => Ok(SetProp::NoAnim(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "noblur" => Ok(SetProp::NoBlur(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "noborder" => Ok(SetProp::NoBorder(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "nodim" => Ok(SetProp::NoDim(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "nofocus" => Ok(SetProp::NoFocus(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "nofollowmouse" => Ok(SetProp::NoFollowMouse(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "nomaxsize" => Ok(SetProp::NoMaxSize(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "norounding" => Ok(SetProp::NoRounding(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "noshadow" => Ok(SetProp::NoShadow(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "noshortcutsinhibit" => Ok(SetProp::NoShortcutsInhibit(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "opaque" => Ok(SetProp::Opaque(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "forcergbx" => Ok(SetProp::ForceRGBX(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "syncfullscreen" => Ok(SetProp::SyncFullscreen(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "immediate" => Ok(SetProp::Immediate(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "xray" => Ok(SetProp::Xray(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "renderunfocused" => Ok(SetProp::RenderUnfocused),
            "scrollmouse" => Ok(SetProp::ScrollMouse(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "scrolltouchpad" => Ok(SetProp::ScrollTouchpad(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "noscreenshare" => Ok(SetProp::NoScreenShare(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            "novrr" => Ok(SetProp::NoVRR(
                parts.get(1).unwrap_or(&"").parse().unwrap_or_default(),
            )),
            _ => Err(()),
        }
    }
}

impl Display for SetProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetProp::Alpha(alpha) => write!(f, "alpha {}", alpha),
            SetProp::AlphaOverride(false) => write!(f, "alphaoverride 0"),
            SetProp::AlphaOverride(true) => write!(f, "alphaoverride 1"),
            SetProp::AlphaInactive(alpha) => write!(f, "alphainactive {}", alpha),
            SetProp::AlphaInactiveOverride(false) => write!(f, "alphainactiveoverride 0"),
            SetProp::AlphaInactiveOverride(true) => write!(f, "alphainactiveoverride 1"),
            SetProp::AlphaFullscreen(alpha) => write!(f, "alphafullscreen {}", alpha),
            SetProp::AlphaFullscreenOverride(false) => write!(f, "alphafullscreenoverride 0"),
            SetProp::AlphaFullscreenOverride(true) => write!(f, "alphafullscreenoverride 1"),
            SetProp::AnimationStyle(style) => write!(f, "animationstyle {}", style),
            SetProp::ActiveBorderColor(None) => write!(f, "activebordercolor -1"),
            SetProp::ActiveBorderColor(Some(color)) => write!(f, "activebordercolor {}", color),
            SetProp::InactiveBorderColor(None) => write!(f, "inactivebordercolor -1"),
            SetProp::InactiveBorderColor(Some(color)) => write!(f, "inactivebordercolor {}", color),
            SetProp::Animation(animation) => write!(f, "animation {}", animation),
            SetProp::BorderColor(border_color) => write!(f, "bordercolor {}", border_color),
            SetProp::IdleIngibit(mode) => write!(f, "idleingibit {}", mode),
            SetProp::Opacity(opacity) => write!(f, "opacity {}", opacity),
            SetProp::Tag(TagToggleState::Set, tag) => write!(f, "tag +{}", tag),
            SetProp::Tag(TagToggleState::Unset, tag) => write!(f, "tag -{}", tag),
            SetProp::Tag(TagToggleState::Toggle, tag) => write!(f, "tag {}", tag),
            SetProp::MaxSize(width, height) => write!(f, "maxsize {} {}", width, height),
            SetProp::MinSize(width, height) => write!(f, "minsize {} {}", width, height),
            SetProp::BorderSize(size) => write!(f, "bordersize {}", size),
            SetProp::Rounding(size) => write!(f, "rounding {}", size),
            SetProp::RoundingPower(power) => write!(f, "roundingpower {}", power),
            SetProp::AllowsInput(mode) => write!(f, "allowsinput {}", mode),
            SetProp::DimAround(mode) => write!(f, "dimaround {}", mode),
            SetProp::Decorate(mode) => write!(f, "decorate {}", mode),
            SetProp::FocusOnActivate(mode) => write!(f, "focusonactivate {}", mode),
            SetProp::KeepAspectRatio(mode) => write!(f, "keepaspectratio {}", mode),
            SetProp::NearestNeighbor(mode) => write!(f, "nearestneighbor {}", mode),
            SetProp::NoAnim(mode) => write!(f, "noanim {}", mode),
            SetProp::NoBlur(mode) => write!(f, "noblur {}", mode),
            SetProp::NoBorder(mode) => write!(f, "noborder {}", mode),
            SetProp::NoDim(mode) => write!(f, "nodim {}", mode),
            SetProp::NoFocus(mode) => write!(f, "nofocus {}", mode),
            SetProp::NoFollowMouse(mode) => write!(f, "nofollowmouse {}", mode),
            SetProp::NoMaxSize(mode) => write!(f, "nomaxsize {}", mode),
            SetProp::NoRounding(mode) => write!(f, "norounding {}", mode),
            SetProp::NoShadow(mode) => write!(f, "noshadow {}", mode),
            SetProp::NoShortcutsInhibit(mode) => write!(f, "noshortcutsinhibit {}", mode),
            SetProp::Opaque(mode) => write!(f, "opaque {}", mode),
            SetProp::ForceRGBX(mode) => write!(f, "forcergbx {}", mode),
            SetProp::SyncFullscreen(mode) => write!(f, "syncfullscreen {}", mode),
            SetProp::Immediate(mode) => write!(f, "immediate {}", mode),
            SetProp::Xray(mode) => write!(f, "xray {}", mode),
            SetProp::RenderUnfocused => write!(f, "renderunfocused"),
            SetProp::ScrollMouse(speed) => write!(f, "scrollmouse {}", speed),
            SetProp::ScrollTouchpad(speed) => write!(f, "scrolltouchpad {}", speed),
            SetProp::NoScreenShare(mode) => write!(f, "noscreenshare {}", mode),
            SetProp::NoVRR(mode) => write!(f, "novrr {}", mode),
        }
    }
}

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(DispatcherDiscriminant))]
pub enum Dispatcher {
    Exec(Vec<WindowRule>, String),
    Execr(String),
    Pass(WindowTarget),
    SendShortcut(HashSet<Modifier>, String, Option<WindowTarget>),
    SendKeyState(HashSet<Modifier>, String, KeyState, WindowTarget),
    KillActive,
    ForceKillActive,
    CloseWindow(WindowTarget),
    KillWindow(WindowTarget),
    Signal(String),
    SignalWindow(WindowTarget, String),
    Workspace(WorkspaceTarget),
    MoveToWorkspace(WorkspaceTarget, Option<WindowTarget>),
    MoveToWorkspaceSilent(WorkspaceTarget, Option<WindowTarget>),
    ToggleFloating(Option<WindowTarget>),
    SetFloating(Option<WindowTarget>),
    SetTiled(Option<WindowTarget>),
    Fullscreen(FullscreenMode),
    FullscreenState(DispatcherFullscreenState, DispatcherFullscreenState),
    Dpms(ToggleState, Option<String>),
    Pin(Option<WindowTarget>),
    MoveFocus(Direction),
    MoveWindow(MoveDirection),
    SwapWindow(SwapDirection),
    CenterWindow(bool),
    ResizeActive(ResizeParams),
    MoveActive(ResizeParams),
    ResizeWindowPixel(ResizeParams, WindowTarget),
    MoveWindowPixel(ResizeParams, WindowTarget),
    CycleNext(CycleNext),
    SwapNext(SwapNext),
    TagWindow(TagToggleState, String, Option<WindowTarget>),
    FocusWindow(WindowTarget),
    FocusMonitor(MonitorTarget),
    SplitRatio(FloatValue),
    MoveCursorToCorner(CursorCorner),
    MoveCursor(u32, u32),
    RenameWorkspace(u32, String),
    Exit,
    ForceRendererReload,
    MoveCurrentWorkspaceToMonitor(MonitorTarget),
    FocusWorkspaceOnCurrentMonitor(WorkspaceTarget),
    MoveWorkspaceToMonitor(WorkspaceTarget, MonitorTarget),
    SwapActiveWorkspaces(MonitorTarget, MonitorTarget),
    BringActiveToTop,
    AlterZOrder(ZHeight, Option<WindowTarget>),
    ToggleSpecialWorkspace(Option<String>),
    FocusUrgentOrLast,
    ToggleGroup,
    ChangeGroupActive(ChangeGroupActive),
    FocusCurrentOrLast,
    LockGroups(GroupLockAction),
    LockActiveGroup(GroupLockAction),
    MoveIntoGroup(Direction),
    MoveOutOfGroup(Option<WindowTarget>),
    MoveWindowOrGroup(Direction),
    MoveGroupWindow(bool),
    DenyWindowFromGroup(ToggleState),
    SetIgnoreGroupLock(ToggleState),
    Global(String),
    Event(String),
    SetProp(SetProp),
    ToggleSwallow,
}

impl HasDiscriminant for Dispatcher {
    type Discriminant = DispatcherDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Exec => Self::Exec(Vec::new(), "".to_string()),
            Self::Discriminant::Execr => Self::Execr("".to_string()),
            Self::Discriminant::Pass => Self::Pass(WindowTarget::default()),
            Self::Discriminant::SendShortcut => {
                Self::SendShortcut(HashSet::new(), "".to_string(), None)
            }
            Self::Discriminant::SendKeyState => Self::SendKeyState(
                HashSet::new(),
                "".to_string(),
                KeyState::default(),
                WindowTarget::default(),
            ),
            Self::Discriminant::KillActive => Self::KillActive,
            Self::Discriminant::ForceKillActive => Self::ForceKillActive,
            Self::Discriminant::CloseWindow => Self::CloseWindow(WindowTarget::default()),
            Self::Discriminant::KillWindow => Self::KillWindow(WindowTarget::default()),
            Self::Discriminant::Signal => Self::Signal("".to_string()),
            Self::Discriminant::SignalWindow => {
                Self::SignalWindow(WindowTarget::default(), "".to_string())
            }
            Self::Discriminant::Workspace => Self::Workspace(WorkspaceTarget::default()),
            Self::Discriminant::MoveToWorkspace => {
                Self::MoveToWorkspace(WorkspaceTarget::default(), None)
            }
            Self::Discriminant::MoveToWorkspaceSilent => {
                Self::MoveToWorkspaceSilent(WorkspaceTarget::default(), None)
            }
            Self::Discriminant::ToggleFloating => Self::ToggleFloating(None),
            Self::Discriminant::SetFloating => Self::SetFloating(None),
            Self::Discriminant::SetTiled => Self::SetTiled(None),
            Self::Discriminant::Fullscreen => Self::Fullscreen(FullscreenMode::default()),
            Self::Discriminant::FullscreenState => Self::FullscreenState(
                DispatcherFullscreenState::default(),
                DispatcherFullscreenState::default(),
            ),
            Self::Discriminant::Dpms => Self::Dpms(ToggleState::default(), None),
            Self::Discriminant::Pin => Self::Pin(None),
            Self::Discriminant::MoveFocus => Self::MoveFocus(Direction::default()),
            Self::Discriminant::MoveWindow => Self::MoveWindow(MoveDirection::default()),
            Self::Discriminant::SwapWindow => Self::SwapWindow(SwapDirection::default()),
            Self::Discriminant::CenterWindow => Self::CenterWindow(false),
            Self::Discriminant::ResizeActive => Self::ResizeActive(ResizeParams::default()),
            Self::Discriminant::MoveActive => Self::MoveActive(ResizeParams::default()),
            Self::Discriminant::ResizeWindowPixel => {
                Self::ResizeWindowPixel(ResizeParams::default(), WindowTarget::default())
            }
            Self::Discriminant::MoveWindowPixel => {
                Self::MoveWindowPixel(ResizeParams::default(), WindowTarget::default())
            }
            Self::Discriminant::CycleNext => Self::CycleNext(CycleNext::default()),
            Self::Discriminant::SwapNext => Self::SwapNext(SwapNext::default()),
            Self::Discriminant::TagWindow => {
                Self::TagWindow(TagToggleState::Toggle, "".to_string(), None)
            }
            Self::Discriminant::FocusWindow => Self::FocusWindow(WindowTarget::default()),
            Self::Discriminant::FocusMonitor => Self::FocusMonitor(MonitorTarget::default()),
            Self::Discriminant::SplitRatio => Self::SplitRatio(FloatValue::default()),
            Self::Discriminant::MoveCursorToCorner => {
                Self::MoveCursorToCorner(CursorCorner::default())
            }
            Self::Discriminant::MoveCursor => Self::MoveCursor(0, 0),
            Self::Discriminant::RenameWorkspace => Self::RenameWorkspace(1, "".to_string()),
            Self::Discriminant::Exit => Self::Exit,
            Self::Discriminant::ForceRendererReload => Self::ForceRendererReload,
            Self::Discriminant::MoveCurrentWorkspaceToMonitor => {
                Self::MoveCurrentWorkspaceToMonitor(MonitorTarget::default())
            }
            Self::Discriminant::FocusWorkspaceOnCurrentMonitor => {
                Self::FocusWorkspaceOnCurrentMonitor(WorkspaceTarget::default())
            }
            Self::Discriminant::MoveWorkspaceToMonitor => {
                Self::MoveWorkspaceToMonitor(WorkspaceTarget::default(), MonitorTarget::default())
            }
            Self::Discriminant::SwapActiveWorkspaces => {
                Self::SwapActiveWorkspaces(MonitorTarget::default(), MonitorTarget::default())
            }
            Self::Discriminant::BringActiveToTop => Self::BringActiveToTop,
            Self::Discriminant::AlterZOrder => Self::AlterZOrder(ZHeight::default(), None),
            Self::Discriminant::ToggleSpecialWorkspace => Self::ToggleSpecialWorkspace(None),
            Self::Discriminant::FocusUrgentOrLast => Self::FocusUrgentOrLast,
            Self::Discriminant::ToggleGroup => Self::ToggleGroup,
            Self::Discriminant::ChangeGroupActive => {
                Self::ChangeGroupActive(ChangeGroupActive::default())
            }
            Self::Discriminant::FocusCurrentOrLast => Self::FocusCurrentOrLast,
            Self::Discriminant::LockGroups => Self::LockGroups(GroupLockAction::default()),
            Self::Discriminant::LockActiveGroup => {
                Self::LockActiveGroup(GroupLockAction::default())
            }
            Self::Discriminant::MoveIntoGroup => Self::MoveIntoGroup(Direction::default()),
            Self::Discriminant::MoveOutOfGroup => Self::MoveOutOfGroup(None),
            Self::Discriminant::MoveWindowOrGroup => Self::MoveWindowOrGroup(Direction::default()),
            Self::Discriminant::MoveGroupWindow => Self::MoveGroupWindow(false),
            Self::Discriminant::DenyWindowFromGroup => {
                Self::DenyWindowFromGroup(ToggleState::default())
            }
            Self::Discriminant::SetIgnoreGroupLock => {
                Self::SetIgnoreGroupLock(ToggleState::default())
            }
            Self::Discriminant::Global => Self::Global("".to_string()),
            Self::Discriminant::Event => Self::Event("".to_string()),
            Self::Discriminant::SetProp => Self::SetProp(SetProp::default()),
            Self::Discriminant::ToggleSwallow => Self::ToggleSwallow,
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Exec => {
                if str.starts_with('[') {
                    let mut rules = Vec::new();
                    let mut rule = String::new();
                    let mut in_brackets = false;
                    let mut command = String::new();

                    for c in str.chars() {
                        if c == '[' {
                            in_brackets = true;
                        } else if c == ']' {
                            if !rule.trim().is_empty() {
                                rules.push(rule.parse().unwrap_or_default());
                                rule.clear();
                            }
                            in_brackets = false;
                        } else if c == ';' && in_brackets {
                            if !rule.trim().is_empty() {
                                rules.push(rule.parse().unwrap_or_default());
                                rule.clear();
                            }
                        } else if in_brackets {
                            rule.push(c);
                        } else {
                            command.push(c);
                        }
                    }

                    Self::Exec(rules, command.trim_start().to_string())
                } else {
                    Self::Exec(Vec::new(), str.to_string())
                }
            }
            Self::Discriminant::Execr => Self::Execr(str.to_string()),
            Self::Discriminant::Pass => Self::Pass(str.parse().unwrap_or_default()),
            Self::Discriminant::SendShortcut => {
                let parts: Vec<&str> = str.split(',').collect();
                if parts.len() == 1 {
                    Self::SendShortcut(parse_modifiers(parts[0]), String::new(), None)
                } else if parts.len() == 2 {
                    Self::SendShortcut(
                        parse_modifiers(parts[0]),
                        parts[1].parse().unwrap_or_default(),
                        None,
                    )
                } else {
                    Self::SendShortcut(
                        parse_modifiers(parts[0]),
                        parts[1].parse().unwrap_or_default(),
                        Some(WindowTarget::from_str(parts[2]).unwrap_or_default()),
                    )
                }
            }
            Self::Discriminant::SendKeyState => {
                let parts: Vec<&str> = str.split(',').collect();
                let mods = parse_modifiers(parts.first().unwrap_or(&""));
                let key = parts.get(1).unwrap_or(&"").to_string();
                let state = parts.get(2).unwrap_or(&"").parse().unwrap_or_default();
                let window_target = parts.get(3).unwrap_or(&"").parse().unwrap_or_default();
                Self::SendKeyState(mods, key, state, window_target)
            }
            Self::Discriminant::KillActive => Self::KillActive,
            Self::Discriminant::ForceKillActive => Self::ForceKillActive,
            Self::Discriminant::CloseWindow => Self::CloseWindow(str.parse().unwrap_or_default()),
            Self::Discriminant::KillWindow => Self::KillWindow(str.parse().unwrap_or_default()),
            Self::Discriminant::Signal => Self::Signal(str.to_string()),
            Self::Discriminant::SignalWindow => {
                let parts: Vec<&str> = str.split(',').collect();
                let window_target = parts.first().unwrap_or(&"").parse().unwrap_or_default();
                let signal = parts.get(1).unwrap_or(&"").to_string();
                Self::SignalWindow(window_target, signal)
            }
            Self::Discriminant::Workspace => Self::Workspace(str.parse().unwrap_or_default()),
            Self::Discriminant::MoveToWorkspace => {
                let parts: Vec<&str> = str.split(',').collect();
                if parts.len() == 1 {
                    Self::MoveToWorkspace(str.parse().unwrap_or_default(), None)
                } else {
                    let workspace_target = parts.first().unwrap_or(&"").parse().unwrap_or_default();
                    let window_target = parts.get(1).unwrap_or(&"").parse().unwrap_or_default();
                    Self::MoveToWorkspace(workspace_target, Some(window_target))
                }
            }
            Self::Discriminant::MoveToWorkspaceSilent => {
                let parts: Vec<&str> = str.split(',').collect();
                if parts.len() == 1 {
                    Self::MoveToWorkspaceSilent(str.parse().unwrap_or_default(), None)
                } else {
                    let workspace_target = parts.first().unwrap_or(&"").parse().unwrap_or_default();
                    let window_target = parts.get(1).unwrap_or(&"").parse().unwrap_or_default();
                    Self::MoveToWorkspaceSilent(workspace_target, Some(window_target))
                }
            }
            Self::Discriminant::ToggleFloating => {
                if str.is_empty() || str == "active" {
                    Self::ToggleFloating(None)
                } else {
                    Self::ToggleFloating(Some(str.parse().unwrap_or_default()))
                }
            }
            Self::Discriminant::SetFloating => {
                if str.is_empty() || str == "active" {
                    Self::SetFloating(None)
                } else {
                    Self::SetFloating(Some(str.parse().unwrap_or_default()))
                }
            }
            Self::Discriminant::SetTiled => {
                if str.is_empty() || str == "active" {
                    Self::SetTiled(None)
                } else {
                    Self::SetTiled(Some(str.parse().unwrap_or_default()))
                }
            }
            Self::Discriminant::Fullscreen => {
                Self::Fullscreen(FullscreenMode::from_num(str.parse().unwrap_or(0)))
            }
            Self::Discriminant::FullscreenState => {
                let (internal, client) = str.split_once(' ').unwrap_or((str, ""));
                let internal = internal.parse().unwrap_or(0);
                let client = client.parse().unwrap_or(0);
                Self::FullscreenState(
                    DispatcherFullscreenState::from_num(internal),
                    DispatcherFullscreenState::from_num(client),
                )
            }
            Self::Discriminant::Dpms => {
                let (state, monitor_name) = str.split_once(' ').unwrap_or((str, ""));
                let state = state.parse().unwrap_or_default();
                let monitor_name = match monitor_name {
                    "" => None,
                    name => Some(name.to_string()),
                };
                Self::Dpms(state, monitor_name)
            }
            Self::Discriminant::Pin => {
                if str.is_empty() || str == "active" {
                    Self::Pin(None)
                } else {
                    Self::Pin(Some(str.parse().unwrap_or_default()))
                }
            }
            Self::Discriminant::MoveFocus => Self::MoveFocus(str.parse().unwrap_or_default()),
            Self::Discriminant::MoveWindow => Self::MoveWindow(str.parse().unwrap_or_default()),
            Self::Discriminant::SwapWindow => Self::SwapWindow(str.parse().unwrap_or_default()),
            Self::Discriminant::CenterWindow => Self::CenterWindow(str.parse().unwrap_or_default()),
            Self::Discriminant::ResizeActive => Self::ResizeActive(str.parse().unwrap_or_default()),
            Self::Discriminant::MoveActive => Self::MoveActive(str.parse().unwrap_or_default()),
            Self::Discriminant::ResizeWindowPixel => {
                let (resize_params, window_target) = str.split_once(' ').unwrap_or((str, ""));
                let resize_params = ResizeParams::from_str(resize_params).unwrap_or_default();
                let window_target = window_target.parse().unwrap_or_default();
                Self::ResizeWindowPixel(resize_params, window_target)
            }
            Self::Discriminant::MoveWindowPixel => {
                let (resize_params, window_target) = str.split_once(' ').unwrap_or((str, ""));
                let resize_params = ResizeParams::from_str(resize_params).unwrap_or_default();
                let window_target = window_target.parse().unwrap_or_default();
                Self::MoveWindowPixel(resize_params, window_target)
            }
            Self::Discriminant::CycleNext => Self::CycleNext(str.parse().unwrap_or_default()),
            Self::Discriminant::SwapNext => Self::SwapNext(str.parse().unwrap_or_default()),
            Self::Discriminant::TagWindow => {
                let (tag, window_target) = str.split_once(' ').unwrap_or((str, ""));
                let tag_toggle_state = if tag.starts_with("+") {
                    TagToggleState::Set
                } else if tag.starts_with("-") {
                    TagToggleState::Unset
                } else {
                    TagToggleState::Toggle
                };
                let tag = tag
                    .trim_start_matches("+")
                    .trim_start_matches("-")
                    .to_string();
                let window_target = match window_target {
                    "" => None,
                    window_target => Some(window_target.parse().unwrap_or_default()),
                };
                Self::TagWindow(tag_toggle_state, tag, window_target)
            }
            Self::Discriminant::FocusWindow => Self::FocusWindow(str.parse().unwrap_or_default()),
            Self::Discriminant::FocusMonitor => Self::FocusMonitor(str.parse().unwrap_or_default()),
            Self::Discriminant::SplitRatio => Self::SplitRatio(str.parse().unwrap_or_default()),
            Self::Discriminant::MoveCursorToCorner => {
                Self::MoveCursorToCorner(CursorCorner::from_num(str.parse().unwrap_or_default()))
            }
            Self::Discriminant::MoveCursor => {
                let (x, y) = str.split_once(' ').unwrap_or((str, ""));
                let x = x.parse().unwrap_or_default();
                let y = y.parse().unwrap_or_default();
                Self::MoveCursor(x, y)
            }
            Self::Discriminant::RenameWorkspace => {
                let (workspace, name) = str.split_once(' ').unwrap_or((str, ""));
                let workspace_id = match workspace.parse().unwrap_or_default() {
                    0 => 1,
                    id => id,
                };
                let name = name.to_string();
                Self::RenameWorkspace(workspace_id, name)
            }
            Self::Discriminant::Exit => Self::Exit,
            Self::Discriminant::ForceRendererReload => Self::ForceRendererReload,
            Self::Discriminant::MoveCurrentWorkspaceToMonitor => {
                Self::MoveCurrentWorkspaceToMonitor(str.parse().unwrap_or_default())
            }
            Self::Discriminant::FocusWorkspaceOnCurrentMonitor => {
                Self::FocusWorkspaceOnCurrentMonitor(str.parse().unwrap_or_default())
            }
            Self::Discriminant::MoveWorkspaceToMonitor => {
                let (workspace_target, monitor_target) = str.split_once(' ').unwrap_or((str, ""));
                let workspace_target = workspace_target.parse().unwrap_or_default();
                let monitor_target = monitor_target.parse().unwrap_or_default();
                Self::MoveWorkspaceToMonitor(workspace_target, monitor_target)
            }
            Self::Discriminant::SwapActiveWorkspaces => {
                let (first_monitor, second_monitor) = str.split_once(' ').unwrap_or((str, ""));
                let first_monitor = first_monitor.parse().unwrap_or_default();
                let second_monitor = second_monitor.parse().unwrap_or_default();
                Self::SwapActiveWorkspaces(first_monitor, second_monitor)
            }
            Self::Discriminant::BringActiveToTop => Self::BringActiveToTop,
            Self::Discriminant::AlterZOrder => {
                let (zheight, window_target) = str.split_once(' ').unwrap_or((str, ""));
                let zheight = zheight.parse().unwrap_or_default();
                let window_target = match window_target {
                    "" => None,
                    window_target => Some(window_target.parse().unwrap_or_default()),
                };
                Self::AlterZOrder(zheight, window_target)
            }
            Self::Discriminant::ToggleSpecialWorkspace => match str {
                "" => Self::ToggleSpecialWorkspace(None),
                name => Self::ToggleSpecialWorkspace(Some(name.to_string())),
            },
            Self::Discriminant::FocusUrgentOrLast => Self::FocusUrgentOrLast,
            Self::Discriminant::ToggleGroup => Self::ToggleGroup,
            Self::Discriminant::ChangeGroupActive => {
                Self::ChangeGroupActive(str.parse().unwrap_or_default())
            }
            Self::Discriminant::FocusCurrentOrLast => Self::FocusCurrentOrLast,
            Self::Discriminant::LockGroups => Self::LockGroups(str.parse().unwrap_or_default()),
            Self::Discriminant::LockActiveGroup => {
                Self::LockActiveGroup(str.parse().unwrap_or_default())
            }
            Self::Discriminant::MoveIntoGroup => {
                Self::MoveIntoGroup(str.parse().unwrap_or_default())
            }
            Self::Discriminant::MoveOutOfGroup => {
                if str.is_empty() || str == "active" {
                    Self::MoveOutOfGroup(None)
                } else {
                    Self::MoveOutOfGroup(Some(str.parse().unwrap_or_default()))
                }
            }
            Self::Discriminant::MoveWindowOrGroup => {
                Self::MoveWindowOrGroup(str.parse().unwrap_or_default())
            }
            Self::Discriminant::MoveGroupWindow => Self::MoveGroupWindow(str == "b"),
            Self::Discriminant::DenyWindowFromGroup => {
                Self::DenyWindowFromGroup(str.parse().unwrap_or_default())
            }
            Self::Discriminant::SetIgnoreGroupLock => {
                Self::SetIgnoreGroupLock(str.parse().unwrap_or_default())
            }
            Self::Discriminant::Global => Self::Global(str.to_string()),
            Self::Discriminant::Event => Self::Event(str.to_string()),
            Self::Discriminant::SetProp => Self::SetProp(str.parse().unwrap_or_default()),
            Self::Discriminant::ToggleSwallow => Self::ToggleSwallow,
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            Dispatcher::Exec(window_rules, command) => {
                if window_rules.is_empty() {
                    Some(command.clone())
                } else {
                    Some(format!(
                        "[{}] {}",
                        join_with_separator(window_rules, "; "),
                        command
                    ))
                }
            }
            Dispatcher::Execr(command) => Some(command.clone()),
            Dispatcher::Pass(window_target) => Some(window_target.to_string()),
            Dispatcher::SendShortcut(modifiers, key, None) => {
                Some(format!("{} {}", join_with_separator(modifiers, "_"), key))
            }
            Dispatcher::SendShortcut(modifiers, key, Some(window_target)) => Some(format!(
                "{} {} {}",
                join_with_separator(modifiers, "_"),
                key,
                window_target
            )),
            Dispatcher::SendKeyState(modifiers, key, state, window_target) => Some(format!(
                "{} {} {} {}",
                join_with_separator(modifiers, "_"),
                key,
                state,
                window_target
            )),
            Dispatcher::KillActive => None,
            Dispatcher::ForceKillActive => None,
            Dispatcher::CloseWindow(window_target) => Some(window_target.to_string()),
            Dispatcher::KillWindow(window_target) => Some(window_target.to_string()),
            Dispatcher::Signal(signal) => Some(signal.clone()),
            Dispatcher::SignalWindow(window_target, signal) => {
                Some(format!("{} {}", window_target, signal))
            }
            Dispatcher::Workspace(workspace_target) => Some(workspace_target.to_string()),
            Dispatcher::MoveToWorkspace(workspace_target, None) => {
                Some(workspace_target.to_string())
            }
            Dispatcher::MoveToWorkspace(workspace_target, Some(window_target)) => {
                Some(format!("{} {}", workspace_target, window_target))
            }
            Dispatcher::MoveToWorkspaceSilent(workspace_target, None) => {
                Some(workspace_target.to_string())
            }
            Dispatcher::MoveToWorkspaceSilent(workspace_target, Some(window_target)) => {
                Some(format!("{} {}", workspace_target, window_target))
            }
            Dispatcher::ToggleFloating(None) => None,
            Dispatcher::ToggleFloating(Some(window_target)) => Some(window_target.to_string()),
            Dispatcher::SetFloating(None) => None,
            Dispatcher::SetFloating(Some(window_target)) => Some(window_target.to_string()),
            Dispatcher::SetTiled(None) => None,
            Dispatcher::SetTiled(Some(window_target)) => Some(window_target.to_string()),
            Dispatcher::Fullscreen(mode) => Some(mode.to_num().to_string()),
            Dispatcher::FullscreenState(internal, client) => {
                Some(format!("{} {}", internal.to_num(), client.to_num()))
            }
            Dispatcher::Dpms(state, None) => Some(state.to_string()),
            Dispatcher::Dpms(state, Some(name)) => Some(format!("{} {}", state, name)),
            Dispatcher::Pin(None) => None,
            Dispatcher::Pin(Some(window_target)) => Some(window_target.to_string()),
            Dispatcher::MoveFocus(direction) => Some(direction.to_string()),
            Dispatcher::MoveWindow(move_direction) => Some(move_direction.to_string()),
            Dispatcher::SwapWindow(swap_direction) => Some(swap_direction.to_string()),
            Dispatcher::CenterWindow(false) => None,
            Dispatcher::CenterWindow(true) => Some("1".to_string()),
            Dispatcher::ResizeActive(resize_params) => Some(resize_params.to_string()),
            Dispatcher::MoveActive(resize_params) => Some(resize_params.to_string()),
            Dispatcher::ResizeWindowPixel(resize_params, window_target) => {
                Some(format!("{} {}", resize_params, window_target))
            }
            Dispatcher::MoveWindowPixel(move_params, window_target) => {
                Some(format!("{} {}", move_params, window_target))
            }
            Dispatcher::CycleNext(cycle_next) => Some(cycle_next.to_string()),
            Dispatcher::SwapNext(swap_next) => Some(swap_next.to_string()),
            Dispatcher::TagWindow(TagToggleState::Toggle, tag, None) => Some(tag.clone()),
            Dispatcher::TagWindow(TagToggleState::Toggle, tag, Some(window_target)) => {
                Some(format!("{} {}", tag, window_target))
            }
            Dispatcher::TagWindow(TagToggleState::Set, tag, None) => Some(format!("+{}", tag)),
            Dispatcher::TagWindow(TagToggleState::Set, tag, Some(window_target)) => {
                Some(format!("+{} {}", tag, window_target))
            }
            Dispatcher::TagWindow(TagToggleState::Unset, tag, None) => Some(format!("-{}", tag)),
            Dispatcher::TagWindow(TagToggleState::Unset, tag, Some(window_target)) => {
                Some(format!("-{} {}", tag, window_target))
            }
            Dispatcher::FocusWindow(window_target) => Some(window_target.to_string()),
            Dispatcher::FocusMonitor(monitor_target) => Some(monitor_target.to_string()),
            Dispatcher::SplitRatio(float_value) => Some(float_value.to_string()),
            Dispatcher::MoveCursorToCorner(corner) => Some(corner.to_num().to_string()),
            Dispatcher::MoveCursor(x, y) => Some(format!("{} {}", x, y)),
            Dispatcher::RenameWorkspace(id, name) => Some(format!("{} {}", id, name)),
            Dispatcher::Exit => None,
            Dispatcher::ForceRendererReload => None,
            Dispatcher::MoveCurrentWorkspaceToMonitor(monitor_target) => {
                Some(monitor_target.to_string())
            }
            Dispatcher::FocusWorkspaceOnCurrentMonitor(workspace_target) => {
                Some(workspace_target.to_string())
            }
            Dispatcher::MoveWorkspaceToMonitor(workspace_target, monitor_target) => {
                Some(format!("{} {}", workspace_target, monitor_target))
            }
            Dispatcher::SwapActiveWorkspaces(first_monitor, second_monitor) => {
                Some(format!("{} {}", first_monitor, second_monitor))
            }
            Dispatcher::BringActiveToTop => None,
            Dispatcher::AlterZOrder(zheight, None) => Some(zheight.to_string()),
            Dispatcher::AlterZOrder(zheight, Some(window_target)) => {
                Some(format!("{} {}", zheight, window_target))
            }
            Dispatcher::ToggleSpecialWorkspace(None) => None,
            Dispatcher::ToggleSpecialWorkspace(Some(name)) => Some(name.clone()),
            Dispatcher::FocusUrgentOrLast => None,
            Dispatcher::ToggleGroup => None,
            Dispatcher::ChangeGroupActive(change_group_active) => {
                Some(change_group_active.to_string())
            }
            Dispatcher::FocusCurrentOrLast => None,
            Dispatcher::LockGroups(group_lock_action) => Some(group_lock_action.to_string()),
            Dispatcher::LockActiveGroup(group_lock_action) => Some(group_lock_action.to_string()),
            Dispatcher::MoveIntoGroup(direction) => Some(direction.to_string()),
            Dispatcher::MoveOutOfGroup(None) => None,
            Dispatcher::MoveOutOfGroup(Some(window_target)) => Some(window_target.to_string()),
            Dispatcher::MoveWindowOrGroup(direction) => Some(direction.to_string()),
            Dispatcher::MoveGroupWindow(true) => Some("b".to_string()),
            Dispatcher::MoveGroupWindow(false) => Some("f".to_string()),
            Dispatcher::DenyWindowFromGroup(toggle_state) => Some(toggle_state.to_string()),
            Dispatcher::SetIgnoreGroupLock(toggle_state) => Some(toggle_state.to_string()),
            Dispatcher::Global(name) => Some(name.clone()),
            Dispatcher::Event(event) => Some(event.clone()),
            Dispatcher::SetProp(set_prop) => Some(set_prop.to_string()),
            Dispatcher::ToggleSwallow => None,
        }
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Dispatcher::Exec(vec![], String::new())
    }
}

impl FromStr for Dispatcher {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        if s.is_empty() {
            return Err(());
        }

        let (dispatcher, params) = s.split_once(",").unwrap_or((&s, ""));

        let dispatcher = dispatcher.trim().to_lowercase();

        let params = params.trim();

        match dispatcher.as_str() {
            "exec" => {
                if params.starts_with("[") {
                    let mut rules = Vec::new();
                    let mut rule = String::new();
                    let mut in_brackets = false;
                    let mut command = String::new();

                    for c in params.chars() {
                        if c == '[' {
                            in_brackets = true;
                        } else if c == ']' {
                            if !rule.trim().is_empty() {
                                rules.push(rule.parse().unwrap_or_default());
                                rule.clear();
                            }
                            in_brackets = false;
                        } else if c == ';' && in_brackets {
                            if !rule.trim().is_empty() {
                                rules.push(rule.parse().unwrap_or_default());
                                rule.clear();
                            }
                        } else if in_brackets {
                            rule.push(c);
                        } else {
                            command.push(c);
                        }
                    }

                    Ok(Dispatcher::Exec(rules, command.trim_start().to_string()))
                } else {
                    Ok(Dispatcher::Exec(Vec::new(), params.to_string()))
                }
            }
            "execr" => Ok(Dispatcher::Execr(params.to_string())),
            "pass" => Ok(Dispatcher::Pass(params.parse().unwrap_or_default())),
            "sendshortcut" => {
                let parts: Vec<&str> = params.split(' ').collect();
                if parts.len() == 1 {
                    Ok(Dispatcher::SendShortcut(
                        parse_modifiers(parts[0]),
                        String::new(),
                        None,
                    ))
                } else if parts.len() == 2 {
                    Ok(Dispatcher::SendShortcut(
                        parse_modifiers(parts[0]),
                        parts[1].parse().unwrap_or_default(),
                        None,
                    ))
                } else {
                    Ok(Dispatcher::SendShortcut(
                        parse_modifiers(parts[0]),
                        parts[1].parse().unwrap_or_default(),
                        Some(WindowTarget::from_str(parts[2]).unwrap_or_default()),
                    ))
                }
            }
            "sendkeystate" => {
                let parts: Vec<&str> = params.split(' ').collect();

                let mods = parse_modifiers(parts.first().unwrap_or(&""));
                let key = parts.get(1).unwrap_or(&"").to_string();
                let state = parts.get(2).unwrap_or(&"").parse().unwrap_or_default();
                let window_target = parts.get(3).unwrap_or(&"").parse().unwrap_or_default();

                Ok(Dispatcher::SendKeyState(mods, key, state, window_target))
            }
            "killactive" => Ok(Dispatcher::KillActive),
            "forcekillactive" => Ok(Dispatcher::ForceKillActive),
            "closewindow" => Ok(Dispatcher::CloseWindow(params.parse().unwrap_or_default())),
            "killwindow" => Ok(Dispatcher::KillWindow(params.parse().unwrap_or_default())),
            "signal" => Ok(Dispatcher::Signal(params.to_string())),
            "signalwindow" => {
                let parts: Vec<&str> = params.split(' ').collect();

                let window_target = parts.first().unwrap_or(&"").parse().unwrap_or_default();
                let signal = parts.get(1).unwrap_or(&"").to_string();

                Ok(Dispatcher::SignalWindow(window_target, signal))
            }
            "workspace" => Ok(Dispatcher::Workspace(params.parse().unwrap_or_default())),
            "movetoworkspace" => {
                let parts: Vec<&str> = params.split(' ').collect();

                if parts.len() == 1 {
                    Ok(Dispatcher::MoveToWorkspace(
                        params.parse().unwrap_or_default(),
                        None,
                    ))
                } else {
                    let workspace_target = parts.first().unwrap_or(&"").parse().unwrap_or_default();
                    let window_target = parts.get(1).unwrap_or(&"").parse().unwrap_or_default();

                    Ok(Dispatcher::MoveToWorkspace(
                        workspace_target,
                        Some(window_target),
                    ))
                }
            }
            "movetoworkspacesilent" => {
                let parts: Vec<&str> = params.split(' ').collect();

                if parts.len() == 1 {
                    Ok(Dispatcher::MoveToWorkspaceSilent(
                        params.parse().unwrap_or_default(),
                        None,
                    ))
                } else {
                    let workspace_target = parts.first().unwrap_or(&"").parse().unwrap_or_default();
                    let window_target = parts.get(1).unwrap_or(&"").parse().unwrap_or_default();

                    Ok(Dispatcher::MoveToWorkspaceSilent(
                        workspace_target,
                        Some(window_target),
                    ))
                }
            }
            "togglefloating" => {
                if params.is_empty() || params == "active" {
                    Ok(Dispatcher::ToggleFloating(None))
                } else {
                    Ok(Dispatcher::ToggleFloating(Some(
                        params.parse().unwrap_or_default(),
                    )))
                }
            }
            "setfloating" => {
                if params.is_empty() || params == "active" {
                    Ok(Dispatcher::SetFloating(None))
                } else {
                    Ok(Dispatcher::SetFloating(Some(
                        params.parse().unwrap_or_default(),
                    )))
                }
            }
            "settiled" => {
                if params.is_empty() || params == "active" {
                    Ok(Dispatcher::SetTiled(None))
                } else {
                    Ok(Dispatcher::SetTiled(Some(
                        params.parse().unwrap_or_default(),
                    )))
                }
            }
            "fullscreen" => Ok(Dispatcher::Fullscreen(FullscreenMode::from_num(
                params.parse().unwrap_or(0),
            ))),
            "fullscreenstate" => {
                let (internal, client) = params.split_once(' ').unwrap_or((params, ""));
                let internal = internal.parse().unwrap_or(0);
                let client = client.parse().unwrap_or(0);

                Ok(Dispatcher::FullscreenState(
                    DispatcherFullscreenState::from_num(internal),
                    DispatcherFullscreenState::from_num(client),
                ))
            }
            "dpms" => {
                let (state, monitor_name) = params.split_once(' ').unwrap_or((params, ""));

                let state = state.parse().unwrap_or_default();
                let monitor_name = match monitor_name {
                    "" => None,
                    name => Some(name.to_string()),
                };

                Ok(Dispatcher::Dpms(state, monitor_name))
            }
            "pin" => {
                if params.is_empty() || params == "active" {
                    Ok(Dispatcher::Pin(None))
                } else {
                    Ok(Dispatcher::Pin(Some(params.parse().unwrap_or_default())))
                }
            }
            "movefocus" => Ok(Dispatcher::MoveFocus(params.parse().unwrap_or_default())),
            "movewindow" => Ok(Dispatcher::MoveWindow(params.parse().unwrap_or_default())),
            "swapwindow" => Ok(Dispatcher::SwapWindow(params.parse().unwrap_or_default())),
            "centerwindow" => Ok(Dispatcher::CenterWindow(params.parse().unwrap_or_default())),
            "resizeactive" => Ok(Dispatcher::ResizeActive(params.parse().unwrap_or_default())),
            "moveactive" => Ok(Dispatcher::MoveActive(params.parse().unwrap_or_default())),
            "resizewindowpixel" => {
                let (resize_params, window_target) = params.split_once(' ').unwrap_or((params, ""));

                let resize_params = ResizeParams::from_str(resize_params).unwrap_or_default();
                let window_target = window_target.parse().unwrap_or_default();

                Ok(Dispatcher::ResizeWindowPixel(resize_params, window_target))
            }
            "movewindowpixel" => {
                let (resize_params, window_target) = params.split_once(' ').unwrap_or((params, ""));

                let resize_params = ResizeParams::from_str(resize_params).unwrap_or_default();
                let window_target = window_target.parse().unwrap_or_default();

                Ok(Dispatcher::MoveWindowPixel(resize_params, window_target))
            }
            "cyclenext" => Ok(Dispatcher::CycleNext(params.parse().unwrap_or_default())),
            "swapnext" => Ok(Dispatcher::SwapNext(params.parse().unwrap_or_default())),
            "tagwindow" => {
                let (tag, window_target) = params.split_once(' ').unwrap_or((params, ""));

                let tag_toggle_state = if tag.starts_with("+") {
                    TagToggleState::Set
                } else if tag.starts_with("-") {
                    TagToggleState::Unset
                } else {
                    TagToggleState::Toggle
                };
                let tag = tag
                    .trim_start_matches("+")
                    .trim_start_matches("-")
                    .to_string();
                let window_target = match window_target {
                    "" => None,
                    window_target => Some(window_target.parse().unwrap_or_default()),
                };

                Ok(Dispatcher::TagWindow(tag_toggle_state, tag, window_target))
            }
            "focuswindow" => Ok(Dispatcher::FocusWindow(params.parse().unwrap_or_default())),
            "focusmonitor" => Ok(Dispatcher::FocusMonitor(params.parse().unwrap_or_default())),
            "splitratio" => Ok(Dispatcher::SplitRatio(params.parse().unwrap_or_default())),
            "movecursortocorner" => Ok(Dispatcher::MoveCursorToCorner(CursorCorner::from_num(
                params.parse().unwrap_or_default(),
            ))),
            "movecursor" => {
                let (x, y) = params.split_once(' ').unwrap_or((params, ""));

                let x = x.parse().unwrap_or_default();
                let y = y.parse().unwrap_or_default();

                Ok(Dispatcher::MoveCursor(x, y))
            }
            "renameworkspace" => {
                let (workspace, name) = params.split_once(' ').unwrap_or((params, ""));

                let workspace_id = match workspace.parse().unwrap_or_default() {
                    0 => 1,
                    id => id,
                };
                let name = name.to_string();

                Ok(Dispatcher::RenameWorkspace(workspace_id, name))
            }
            "exit" => Ok(Dispatcher::Exit),
            "forcerendererreload" => Ok(Dispatcher::ForceRendererReload),
            "movecurrentworkspacetomonitor" => Ok(Dispatcher::MoveCurrentWorkspaceToMonitor(
                params.parse().unwrap_or_default(),
            )),
            "focusworkspaceoncurrentmonitor" => Ok(Dispatcher::FocusWorkspaceOnCurrentMonitor(
                params.parse().unwrap_or_default(),
            )),
            "moveworkspacetomonitor" => {
                let (workspace_target, monitor_target) =
                    params.split_once(' ').unwrap_or((params, ""));

                let workspace_target = workspace_target.parse().unwrap_or_default();
                let monitor_target = monitor_target.parse().unwrap_or_default();

                Ok(Dispatcher::MoveWorkspaceToMonitor(
                    workspace_target,
                    monitor_target,
                ))
            }
            "swapactiveworkspaces" => {
                let (first_monitor, second_monitor) =
                    params.split_once(' ').unwrap_or((params, ""));

                let first_monitor = first_monitor.parse().unwrap_or_default();
                let second_monitor = second_monitor.parse().unwrap_or_default();

                Ok(Dispatcher::SwapActiveWorkspaces(
                    first_monitor,
                    second_monitor,
                ))
            }
            "bringactivetotop" => Ok(Dispatcher::BringActiveToTop),
            "alterzorder" => {
                let (zheight, window_target) = params.split_once(' ').unwrap_or((params, ""));

                let zheight = zheight.parse().unwrap_or_default();
                let window_target = match window_target {
                    "" => None,
                    window_target => Some(window_target.parse().unwrap_or_default()),
                };

                Ok(Dispatcher::AlterZOrder(zheight, window_target))
            }
            "togglespecialworkspace" => match params {
                "" => Ok(Dispatcher::ToggleSpecialWorkspace(None)),
                name => Ok(Dispatcher::ToggleSpecialWorkspace(Some(name.to_string()))),
            },
            "focusurgentorlast" => Ok(Dispatcher::FocusUrgentOrLast),
            "togglegroup" => Ok(Dispatcher::ToggleGroup),
            "changegroupactive" => Ok(Dispatcher::ChangeGroupActive(
                params.parse().unwrap_or_default(),
            )),
            "focuscurrentorlast" => Ok(Dispatcher::FocusCurrentOrLast),
            "lockgroups" => Ok(Dispatcher::LockGroups(params.parse().unwrap_or_default())),
            "lockactivegroup" => Ok(Dispatcher::LockActiveGroup(
                params.parse().unwrap_or_default(),
            )),
            "moveintogroup" => Ok(Dispatcher::MoveIntoGroup(
                params.parse().unwrap_or_default(),
            )),
            "moveoutofgroup" => {
                if params.is_empty() || params == "active" {
                    Ok(Dispatcher::MoveOutOfGroup(None))
                } else {
                    Ok(Dispatcher::MoveOutOfGroup(Some(
                        params.parse().unwrap_or_default(),
                    )))
                }
            }
            "movewindoworgroup" => Ok(Dispatcher::MoveWindowOrGroup(
                params.parse().unwrap_or_default(),
            )),
            "movegroupwindow" => Ok(Dispatcher::MoveGroupWindow(params == "b")),
            "denywindowfromgroup" => Ok(Dispatcher::DenyWindowFromGroup(
                params.parse().unwrap_or_default(),
            )),
            "setignoregrouplock" => Ok(Dispatcher::SetIgnoreGroupLock(
                params.parse().unwrap_or_default(),
            )),
            "global" => Ok(Dispatcher::Global(params.to_string())),
            "event" => Ok(Dispatcher::Event(params.to_string())),
            "setprop" => Ok(Dispatcher::SetProp(params.parse().unwrap_or_default())),
            "toggleswallow" => Ok(Dispatcher::ToggleSwallow),
            _ => Err(()),
        }
    }
}

impl Display for Dispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dispatcher::Exec(window_rules, command) => {
                if window_rules.is_empty() {
                    write!(f, "exec, {}", command)
                } else {
                    write!(
                        f,
                        "exec, [{}] {}",
                        join_with_separator(window_rules, "; "),
                        command.trim()
                    )
                }
            }
            Dispatcher::Execr(command) => write!(f, "execr, {}", command),
            Dispatcher::Pass(window_target) => write!(f, "pass, {}", window_target),
            Dispatcher::SendShortcut(modifiers, key, None) => {
                write!(
                    f,
                    "sendshortcut, {} {}",
                    join_with_separator(modifiers, "_"),
                    key
                )
            }
            Dispatcher::SendShortcut(modifiers, key, Some(window_target)) => write!(
                f,
                "sendshortcut, {} {} {}",
                join_with_separator(modifiers, "_"),
                key,
                window_target,
            ),
            Dispatcher::SendKeyState(modifiers, key, state, window_target) => write!(
                f,
                "sendkeystate, {} {} {} {}",
                join_with_separator(modifiers, "_"),
                key,
                state,
                window_target,
            ),
            Dispatcher::KillActive => write!(f, "killactive"),
            Dispatcher::ForceKillActive => write!(f, "forcekillactive"),
            Dispatcher::CloseWindow(window_target) => {
                write!(f, "killwindow, {}", window_target)
            }
            Dispatcher::KillWindow(window_target) => {
                write!(f, "killwindow, {}", window_target)
            }
            Dispatcher::Signal(signal) => write!(f, "signal, {}", signal),
            Dispatcher::SignalWindow(window_target, signal) => {
                write!(f, "killwindow, {} {}", window_target, signal)
            }
            Dispatcher::Workspace(workspace_target) => write!(f, "workspace, {}", workspace_target),
            Dispatcher::MoveToWorkspace(workspace_target, None) => {
                write!(f, "movetoworkspace, {}", workspace_target)
            }
            Dispatcher::MoveToWorkspace(workspace_target, Some(window_target)) => {
                write!(f, "movetoworkspace, {} {}", workspace_target, window_target,)
            }
            Dispatcher::MoveToWorkspaceSilent(workspace_target, None) => {
                write!(f, "movetoworkspace, {}", workspace_target)
            }
            Dispatcher::MoveToWorkspaceSilent(workspace_target, Some(window_target)) => {
                write!(f, "movetoworkspace, {} {}", workspace_target, window_target,)
            }
            Dispatcher::ToggleFloating(None) => write!(f, "togglefloating"),
            Dispatcher::ToggleFloating(Some(window_target)) => {
                write!(f, "togglefloating, {}", window_target,)
            }
            Dispatcher::SetFloating(None) => write!(f, "setfloating"),
            Dispatcher::SetFloating(Some(window_target)) => {
                write!(f, "setfloating, {}", window_target)
            }
            Dispatcher::SetTiled(None) => write!(f, "settiled"),
            Dispatcher::SetTiled(Some(window_target)) => write!(f, "settiled, {}", window_target),
            Dispatcher::Fullscreen(mode) => write!(f, "fullscreen, {}", mode.to_num()),
            Dispatcher::FullscreenState(internal, client) => {
                write!(f, "fullscreen, {} {}", internal.to_num(), client.to_num())
            }
            Dispatcher::Dpms(state, None) => {
                write!(f, "dpms, {}", state)
            }
            Dispatcher::Dpms(state, Some(name)) => {
                write!(f, "dpms, {} {}", state, name)
            }
            Dispatcher::Pin(None) => write!(f, "pin"),
            Dispatcher::Pin(Some(window_target)) => write!(f, "pin, {}", window_target),
            Dispatcher::MoveFocus(direction) => write!(f, "movefocus, {}", direction),
            Dispatcher::MoveWindow(move_direction) => write!(f, "movewindow, {}", move_direction),
            Dispatcher::SwapWindow(swap_direction) => write!(f, "swapwindow, {}", swap_direction),
            Dispatcher::CenterWindow(false) => write!(f, "centerwindow"),
            Dispatcher::CenterWindow(true) => write!(f, "centerwindow, 1"),
            Dispatcher::ResizeActive(resize_params) => write!(f, "resizeactive, {}", resize_params),
            Dispatcher::MoveActive(resize_params) => write!(f, "moveactive, {}", resize_params),
            Dispatcher::ResizeWindowPixel(resize_params, window_target) => {
                write!(f, "resizewindowpixel, {} {}", resize_params, window_target)
            }
            Dispatcher::MoveWindowPixel(move_params, window_target) => {
                write!(f, "movewindowpixel, {} {}", move_params, window_target)
            }
            Dispatcher::CycleNext(cycle_next) => write!(f, "cyclenext, {}", cycle_next),
            Dispatcher::SwapNext(swap_next) => write!(f, "swapnext, {}", swap_next),
            Dispatcher::TagWindow(TagToggleState::Toggle, tag, None) => {
                write!(f, "tagwindow, {}", tag)
            }
            Dispatcher::TagWindow(TagToggleState::Toggle, tag, Some(window_target)) => {
                write!(f, "tagwindow, {} {}", tag, window_target)
            }
            Dispatcher::TagWindow(TagToggleState::Set, tag, None) => {
                write!(f, "tagwindow, +{}", tag)
            }
            Dispatcher::TagWindow(TagToggleState::Set, tag, Some(window_target)) => {
                write!(f, "tagwindow, +{} {}", tag, window_target)
            }
            Dispatcher::TagWindow(TagToggleState::Unset, tag, None) => {
                write!(f, "tagwindow, -{}", tag)
            }
            Dispatcher::TagWindow(TagToggleState::Unset, tag, Some(window_target)) => {
                write!(f, "tagwindow, -{} {}", tag, window_target)
            }
            Dispatcher::FocusWindow(window_target) => write!(f, "focuswindow, {}", window_target),
            Dispatcher::FocusMonitor(monitor_target) => {
                write!(f, "focusmonitor, {}", monitor_target)
            }
            Dispatcher::SplitRatio(float_value) => write!(f, "splitratio, {}", float_value),
            Dispatcher::MoveCursorToCorner(corner) => {
                write!(f, "movecursortocorner, {}", corner.to_num())
            }
            Dispatcher::MoveCursor(x, y) => write!(f, "movecursor, {} {}", x, y),
            Dispatcher::RenameWorkspace(id, name) => write!(f, "renameworkspace, {} {}", id, name),
            Dispatcher::Exit => write!(f, "exit"),
            Dispatcher::ForceRendererReload => write!(f, "forcerendererreload"),
            Dispatcher::MoveCurrentWorkspaceToMonitor(monitor_target) => {
                write!(f, "movecurrentworkspacetomonitor, {}", monitor_target)
            }
            Dispatcher::FocusWorkspaceOnCurrentMonitor(workspace_target) => {
                write!(f, "focusworkspaceoncurrentmonitor, {}", workspace_target)
            }
            Dispatcher::MoveWorkspaceToMonitor(workspace_target, monitor_target) => {
                write!(
                    f,
                    "moveworkspacetomonitor, {} {}",
                    workspace_target, monitor_target
                )
            }
            Dispatcher::SwapActiveWorkspaces(first_monitor, second_monitor) => {
                write!(
                    f,
                    "swapactiveworkspaces, {} {}",
                    first_monitor, second_monitor
                )
            }
            Dispatcher::BringActiveToTop => write!(f, "bringactivetotop"),
            Dispatcher::AlterZOrder(zheight, None) => {
                write!(f, "alterzorder, {}", zheight)
            }
            Dispatcher::AlterZOrder(zheight, Some(window_target)) => {
                write!(f, "alterzorder, {} {}", zheight, window_target)
            }
            Dispatcher::ToggleSpecialWorkspace(None) => write!(f, "togglespecialworkspace"),
            Dispatcher::ToggleSpecialWorkspace(Some(name)) => {
                write!(f, "togglespecialworkspace, {}", name)
            }
            Dispatcher::FocusUrgentOrLast => write!(f, "focusurgentorlast"),
            Dispatcher::ToggleGroup => write!(f, "togglegroup"),
            Dispatcher::ChangeGroupActive(change_group_active) => {
                write!(f, "changegroupactive, {}", change_group_active)
            }
            Dispatcher::FocusCurrentOrLast => write!(f, "focuscurrentorlast"),
            Dispatcher::LockGroups(group_lock_action) => {
                write!(f, "lockgroups, {}", group_lock_action)
            }
            Dispatcher::LockActiveGroup(group_lock_action) => {
                write!(f, "lockactivegroup, {}", group_lock_action)
            }
            Dispatcher::MoveIntoGroup(direction) => {
                write!(f, "moveintogroup, {}", direction)
            }
            Dispatcher::MoveOutOfGroup(None) => {
                write!(f, "moveoutofgroup")
            }
            Dispatcher::MoveOutOfGroup(Some(window_target)) => {
                write!(f, "moveoutofgroup, {}", window_target)
            }
            Dispatcher::MoveWindowOrGroup(direction) => {
                write!(f, "movewindoworgroup, {}", direction)
            }
            Dispatcher::MoveGroupWindow(true) => {
                write!(f, "movegroupwindow, b")
            }
            Dispatcher::MoveGroupWindow(false) => {
                write!(f, "movegroupwindow, f")
            }
            Dispatcher::DenyWindowFromGroup(toggle_state) => {
                write!(f, "denywindowfromgroup, {}", toggle_state)
            }
            Dispatcher::SetIgnoreGroupLock(toggle_state) => {
                write!(f, "setignoregrouplock, {}", toggle_state)
            }
            Dispatcher::Global(name) => write!(f, "global, {}", name),
            Dispatcher::Event(event) => write!(f, "event, {}", event),
            Dispatcher::SetProp(set_prop) => write!(f, "setprop, {}", set_prop),
            Dispatcher::ToggleSwallow => write!(f, "toggleswallow"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BindRight {
    pub mods: HashSet<Modifier>,
    pub key: String,
    pub dispatcher: Dispatcher,
    pub description: Option<String>,
}

impl FromStr for BindRight {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
        if parts.is_empty() {
            return Err(t!("utils.bind_parse_error", count = parts.len()).into());
        }

        let mods_str = &parts[0];
        let mods = if mods_str.is_empty() {
            HashSet::new()
        } else {
            parse_modifiers(mods_str)
        };

        let key = parts.get(1).unwrap_or(&String::new()).clone();

        let dispatcher_str = match parts.get(2) {
            Some(_) => parts[2..].join(", "),
            None => String::new(),
        };

        let dispatcher = dispatcher_str.parse().unwrap_or(Dispatcher::default());

        Ok(BindRight {
            mods,
            key,
            dispatcher,
            description: None,
        })
    }
}

impl Display for BindRight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = format!(
            "{}, {}{}, {}",
            join_with_separator(&self.mods, "_"),
            self.key,
            if let Some(desc) = &self.description {
                format!(", {}", desc)
            } else {
                "".to_string()
            },
            self.dispatcher,
        );
        write!(f, "{}", result)
    }
}

fn parse_bind_with_description(bind_right_str: &str) -> BindRight {
    let parts: Vec<String> = bind_right_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    if parts.is_empty() {
        return BindRight {
            mods: HashSet::new(),
            key: String::new(),
            dispatcher: Dispatcher::default(),
            description: Some(String::new()),
        };
    }

    let mods_str = &parts[0];
    let mods = if mods_str.is_empty() {
        HashSet::new()
    } else {
        parse_modifiers(mods_str)
    };

    let key = parts.get(1).unwrap_or(&String::new()).clone();

    let description = parts.get(2).unwrap_or(&String::new()).clone();

    let dispatcher_str = match parts.get(3) {
        Some(_) => parts[3..].join(", "),
        None => String::new(),
    };

    let dispatcher = dispatcher_str.parse().unwrap_or(Dispatcher::default());

    BindRight {
        mods,
        key,
        dispatcher,
        description: Some(description),
    }
}

pub fn parse_bind_right(has_description: bool, bind_right_str: &str) -> BindRight {
    if has_description {
        parse_bind_with_description(bind_right_str)
    } else {
        BindRight::from_str(bind_right_str).unwrap_or(BindRight {
            mods: HashSet::new(),
            key: String::new(),
            dispatcher: Dispatcher::default(),
            description: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UnbindRight {
    pub mods: HashSet<Modifier>,
    pub key: String,
}

impl FromStr for UnbindRight {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
        if parts.is_empty() {
            return Err(t!("utils.unbind_parse_error", count = parts.len()).into());
        }

        let mods_str = &parts[0];
        let mods = if mods_str.is_empty() {
            HashSet::new()
        } else {
            parse_modifiers(mods_str)
        };

        let key = parts.get(1).unwrap_or(&String::new()).clone();

        Ok(UnbindRight { mods, key })
    }
}

impl Display for UnbindRight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = format!("{}, {}", join_with_separator(&self.mods, "_"), self.key,);
        write!(f, "{}", result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum GestureDirection {
    #[default]
    Swipe,
    Horizontal,
    Vertical,
    Left,
    Right,
    Up,
    Down,
    Pinch,
    PinchIn,
    PinchOut,
}

impl FromStr for GestureDirection {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "swipe" => Ok(GestureDirection::Swipe),
            "horizontal" => Ok(GestureDirection::Horizontal),
            "vertical" => Ok(GestureDirection::Vertical),
            "left" => Ok(GestureDirection::Left),
            "right" => Ok(GestureDirection::Right),
            "up" => Ok(GestureDirection::Up),
            "down" => Ok(GestureDirection::Down),
            "pinch" => Ok(GestureDirection::Pinch),
            "pinchin" => Ok(GestureDirection::PinchIn),
            "pinchout" => Ok(GestureDirection::PinchOut),
            _ => Err(()),
        }
    }
}

impl Display for GestureDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GestureDirection::Swipe => write!(f, "swipe"),
            GestureDirection::Horizontal => write!(f, "horizontal"),
            GestureDirection::Vertical => write!(f, "vertical"),
            GestureDirection::Left => write!(f, "left"),
            GestureDirection::Right => write!(f, "right"),
            GestureDirection::Up => write!(f, "up"),
            GestureDirection::Down => write!(f, "down"),
            GestureDirection::Pinch => write!(f, "pinch"),
            GestureDirection::PinchIn => write!(f, "pinchin"),
            GestureDirection::PinchOut => write!(f, "pinchout"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum GestureFullscreen {
    #[default]
    Fullscreen,
    Maximize,
}

impl FromStr for GestureFullscreen {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "" | "fullscreen" => Ok(GestureFullscreen::Fullscreen),
            "maximize" => Ok(GestureFullscreen::Maximize),
            _ => Err(()),
        }
    }
}

impl Display for GestureFullscreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GestureFullscreen::Fullscreen => write!(f, ""),
            GestureFullscreen::Maximize => write!(f, "maximize"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum GestureFloating {
    #[default]
    Toggle,
    Float,
    Tile,
}

impl FromStr for GestureFloating {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "" | "toggle" => Ok(GestureFloating::Toggle),
            "float" => Ok(GestureFloating::Float),
            "tile" => Ok(GestureFloating::Tile),
            _ => Err(()),
        }
    }
}

impl Display for GestureFloating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GestureFloating::Toggle => write!(f, ""),
            GestureFloating::Float => write!(f, "float"),
            GestureFloating::Tile => write!(f, "tile"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(GestureActionDiscriminant))]
pub enum GestureAction {
    Dispatcher(Dispatcher),
    Workspace,
    Move,
    Resize,
    Special(String),
    Close,
    Fullscreen(GestureFullscreen),
    Float(GestureFloating),
}

impl HasDiscriminant for GestureAction {
    type Discriminant = GestureActionDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            GestureActionDiscriminant::Dispatcher => {
                GestureAction::Dispatcher(Dispatcher::default())
            }
            GestureActionDiscriminant::Workspace => GestureAction::Workspace,
            GestureActionDiscriminant::Move => GestureAction::Move,
            GestureActionDiscriminant::Resize => GestureAction::Resize,
            GestureActionDiscriminant::Special => GestureAction::Special(String::default()),
            GestureActionDiscriminant::Close => GestureAction::Close,
            GestureActionDiscriminant::Fullscreen => {
                GestureAction::Fullscreen(GestureFullscreen::Fullscreen)
            }
            GestureActionDiscriminant::Float => GestureAction::Float(GestureFloating::default()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            GestureActionDiscriminant::Dispatcher => {
                GestureAction::Dispatcher(Dispatcher::from_str(str).unwrap_or_default())
            }
            GestureActionDiscriminant::Workspace => GestureAction::Workspace,
            GestureActionDiscriminant::Move => GestureAction::Move,
            GestureActionDiscriminant::Resize => GestureAction::Resize,
            GestureActionDiscriminant::Special => GestureAction::Special(str.to_string()),
            GestureActionDiscriminant::Close => GestureAction::Close,
            GestureActionDiscriminant::Fullscreen => {
                GestureAction::Fullscreen(GestureFullscreen::from_str(str).unwrap_or_default())
            }
            GestureActionDiscriminant::Float => {
                GestureAction::Float(GestureFloating::from_str(str).unwrap_or_default())
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            GestureAction::Dispatcher(dispatcher) => Some(dispatcher.to_string()),
            GestureAction::Workspace => None,
            GestureAction::Move => None,
            GestureAction::Resize => None,
            GestureAction::Special(special) => Some(special.to_string()),
            GestureAction::Close => None,
            GestureAction::Fullscreen(fullscreen) => Some(fullscreen.to_string()),
            GestureAction::Float(floating) => Some(floating.to_string()),
        }
    }
}

impl Default for GestureAction {
    fn default() -> Self {
        GestureAction::Dispatcher(Dispatcher::default())
    }
}

impl FromStr for GestureAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (action, arguments) = s.split_once(',').unwrap_or((s, ""));
        let action = action.trim();
        let arguments = arguments.trim();

        match action {
            "dispatcher" => Ok(GestureAction::Dispatcher(
                Dispatcher::from_str(arguments).unwrap_or_default(),
            )),
            "workspace" => Ok(GestureAction::Workspace),
            "move" => Ok(GestureAction::Move),
            "resize" => Ok(GestureAction::Resize),
            "special" => Ok(GestureAction::Special(arguments.to_string())),
            "close" => Ok(GestureAction::Close),
            "fullscreen" => Ok(GestureAction::Fullscreen(
                GestureFullscreen::from_str(arguments).unwrap_or_default(),
            )),
            "float" => Ok(GestureAction::Float(
                GestureFloating::from_str(arguments).unwrap_or_default(),
            )),
            _ => Err(()),
        }
    }
}

impl Display for GestureAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GestureAction::Dispatcher(dispatcher) => write!(f, "dispatcher, {}", dispatcher),
            GestureAction::Workspace => write!(f, "workspace"),
            GestureAction::Move => write!(f, "move"),
            GestureAction::Resize => write!(f, "resize"),
            GestureAction::Special(special) => write!(f, "special, {}", special),
            GestureAction::Close => write!(f, "close"),
            GestureAction::Fullscreen(fullscreen) => write!(f, "fullscreen, {}", fullscreen),
            GestureAction::Float(floating) => write!(f, "float, {}", floating),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gesture {
    pub finger_count: u32,
    pub direction: GestureDirection,
    pub action: GestureAction,
    pub anim_speed: Option<f64>,
    pub mods: Option<HashSet<Modifier>>,
}

impl Default for Gesture {
    fn default() -> Self {
        Gesture {
            finger_count: 3,
            direction: GestureDirection::Swipe,
            action: GestureAction::default(),
            anim_speed: None,
            mods: None,
        }
    }
}

impl FromStr for Gesture {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();

        let finger_count = match parts.first().unwrap_or(&"").trim().parse::<u32>() {
            Ok(0) | Ok(1) | Ok(2) | Ok(3) => 3,
            Ok(finger_count) => finger_count,
            _ => 3,
        };

        let direction = parts
            .get(1)
            .unwrap_or(&"")
            .parse::<GestureDirection>()
            .unwrap_or_default();

        let mut action = GestureAction::default();

        let mut anim_speed = None;

        let mut mods = None;

        for (i, part) in parts.iter().enumerate().skip(2) {
            if let Some(stripped) = part.trim().strip_prefix("mod:") {
                mods = Some(parse_modifiers(stripped));
            } else if let Some(stripped) = part.trim().strip_prefix("scale:")
                && let Ok(speed) = stripped.parse::<f64>()
            {
                anim_speed = Some(speed);
            } else if let Ok(gesture_action) =
                GestureAction::from_str(&format!("{}, {}", part, parts.get(i + 1).unwrap_or(&"")))
            {
                action = gesture_action;
            }
        }

        Ok(Gesture {
            finger_count,
            direction,
            action,
            anim_speed,
            mods,
        })
    }
}

impl Display for Gesture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.anim_speed, &self.mods) {
            (Some(speed), Some(mods)) => write!(
                f,
                "{}, {}, mod:{}, scale:{}, {}",
                self.finger_count,
                self.direction,
                join_with_separator(mods, "_"),
                speed,
                self.action
            ),
            (Some(speed), None) => write!(
                f,
                "{}, {}, scale:{}, {}",
                self.finger_count, self.direction, speed, self.action
            ),
            (None, Some(mods)) => write!(
                f,
                "{}, {}, mod:{}, {}",
                self.finger_count,
                self.direction,
                join_with_separator(mods, "_"),
                self.action
            ),
            (None, None) => write!(
                f,
                "{}, {}, {}",
                self.finger_count, self.direction, self.action
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum WindowRuleFullscreenState {
    #[default]
    Any,
    None,
    Maximize,
    Fullscreen,
    MaximizeAndFullscreen,
}

impl FromStr for WindowRuleFullscreenState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let first_byte = match s.trim_start().as_bytes().first() {
            Some(byte) => byte,
            None => return Err(()),
        };
        match *first_byte {
            b'*' => Ok(WindowRuleFullscreenState::Any),
            b'0' => Ok(WindowRuleFullscreenState::None),
            b'1' => Ok(WindowRuleFullscreenState::Maximize),
            b'2' => Ok(WindowRuleFullscreenState::Fullscreen),
            b'3' => Ok(WindowRuleFullscreenState::MaximizeAndFullscreen),
            _ => Err(()),
        }
    }
}

impl Display for WindowRuleFullscreenState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowRuleFullscreenState::Any => write!(f, "*"),
            WindowRuleFullscreenState::None => write!(f, "0"),
            WindowRuleFullscreenState::Maximize => write!(f, "1"),
            WindowRuleFullscreenState::Fullscreen => write!(f, "2"),
            WindowRuleFullscreenState::MaximizeAndFullscreen => write!(f, "3"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(IdOrNameOrWorkspaceSelectorDiscriminant))]
pub enum IdOrNameOrWorkspaceSelector {
    Id(u32),
    Name(String),
    WorkspaceSelector(Vec<WorkspaceSelector>),
}

impl HasDiscriminant for IdOrNameOrWorkspaceSelector {
    type Discriminant = IdOrNameOrWorkspaceSelectorDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Id => Self::Id(1),
            Self::Discriminant::Name => Self::Name("".to_string()),
            Self::Discriminant::WorkspaceSelector => Self::WorkspaceSelector(Vec::new()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Id => Self::Id(str.parse().unwrap_or_default()),
            Self::Discriminant::Name => Self::Name(str.to_string()),
            Self::Discriminant::WorkspaceSelector => {
                Self::WorkspaceSelector(parse_workspace_selector(str))
            }
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            IdOrNameOrWorkspaceSelector::Id(id) => Some(id.to_string()),
            IdOrNameOrWorkspaceSelector::Name(name) => Some(name.clone()),
            IdOrNameOrWorkspaceSelector::WorkspaceSelector(workspace_selector) => {
                Some(join_with_separator(workspace_selector, ""))
            }
        }
    }
}

impl Default for IdOrNameOrWorkspaceSelector {
    fn default() -> Self {
        IdOrNameOrWorkspaceSelector::Id(1)
    }
}

impl FromStr for IdOrNameOrWorkspaceSelector {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if let Some(name) = s.strip_prefix("name:") {
            Ok(IdOrNameOrWorkspaceSelector::Name(name.to_string()))
        } else if let Ok(id) = s.parse::<u32>() {
            Ok(IdOrNameOrWorkspaceSelector::Id(id))
        } else {
            Ok(IdOrNameOrWorkspaceSelector::WorkspaceSelector(
                parse_workspace_selector(s),
            ))
        }
    }
}

impl Display for IdOrNameOrWorkspaceSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdOrNameOrWorkspaceSelector::Id(id) => write!(f, "{id}"),
            IdOrNameOrWorkspaceSelector::Name(name) => write!(f, "name:{name}"),
            IdOrNameOrWorkspaceSelector::WorkspaceSelector(workspace_selector) => {
                write!(f, "{}", join_with_separator(workspace_selector, ""))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(WindowRuleParameterDiscriminant))]
pub enum WindowRuleParameter {
    Class(String),
    Title(String),
    InitialClass(String),
    InitialTitle(String),
    Tag(String),
    Xwayland,
    NotXwayland,
    Floating,
    NotFloating,
    Fullscreen,
    NotFullscreen,
    Pinned,
    NotPinned,
    Focus,
    NotFocus,
    Group,
    NotGroup,
    FullscreenState(WindowRuleFullscreenState, WindowRuleFullscreenState),
    Workspace(IdOrName),
    OnWorkspace(IdOrNameOrWorkspaceSelector),
    Content(ContentType),
    XdgTag(String),
}

impl HasDiscriminant for WindowRuleParameter {
    type Discriminant = WindowRuleParameterDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            Self::Discriminant::Class => Self::Class(String::new()),
            Self::Discriminant::Title => Self::Title(String::new()),
            Self::Discriminant::InitialClass => Self::InitialClass(String::new()),
            Self::Discriminant::InitialTitle => Self::InitialTitle(String::new()),
            Self::Discriminant::Tag => Self::Tag(String::new()),
            Self::Discriminant::Xwayland => Self::Xwayland,
            Self::Discriminant::NotXwayland => Self::NotXwayland,
            Self::Discriminant::Floating => Self::Floating,
            Self::Discriminant::NotFloating => Self::NotFloating,
            Self::Discriminant::Fullscreen => Self::Fullscreen,
            Self::Discriminant::NotFullscreen => Self::NotFullscreen,
            Self::Discriminant::Pinned => Self::Pinned,
            Self::Discriminant::NotPinned => Self::NotPinned,
            Self::Discriminant::Focus => Self::Focus,
            Self::Discriminant::NotFocus => Self::NotFocus,
            Self::Discriminant::Group => Self::Group,
            Self::Discriminant::NotGroup => Self::NotGroup,
            Self::Discriminant::FullscreenState => Self::FullscreenState(
                WindowRuleFullscreenState::default(),
                WindowRuleFullscreenState::default(),
            ),
            Self::Discriminant::Workspace => Self::Workspace(IdOrName::default()),
            Self::Discriminant::OnWorkspace => {
                Self::OnWorkspace(IdOrNameOrWorkspaceSelector::default())
            }
            Self::Discriminant::Content => Self::Content(ContentType::default()),
            Self::Discriminant::XdgTag => Self::XdgTag(String::new()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            Self::Discriminant::Class => Self::Class(str.to_string()),
            Self::Discriminant::Title => Self::Title(str.to_string()),
            Self::Discriminant::InitialClass => Self::InitialClass(str.to_string()),
            Self::Discriminant::InitialTitle => Self::InitialTitle(str.to_string()),
            Self::Discriminant::Tag => Self::Tag(str.to_string()),
            Self::Discriminant::Xwayland => Self::Xwayland,
            Self::Discriminant::NotXwayland => Self::NotXwayland,
            Self::Discriminant::Floating => Self::Floating,
            Self::Discriminant::NotFloating => Self::NotFloating,
            Self::Discriminant::Fullscreen => Self::Fullscreen,
            Self::Discriminant::NotFullscreen => Self::NotFullscreen,
            Self::Discriminant::Pinned => Self::Pinned,
            Self::Discriminant::NotPinned => Self::NotPinned,
            Self::Discriminant::Focus => Self::Focus,
            Self::Discriminant::NotFocus => Self::NotFocus,
            Self::Discriminant::Group => Self::Group,
            Self::Discriminant::NotGroup => Self::NotGroup,
            Self::Discriminant::FullscreenState => {
                let (state1, state2) = str.split_once(' ').unwrap_or((str, ""));
                Self::FullscreenState(
                    state1.parse().unwrap_or_default(),
                    state2.parse().unwrap_or_default(),
                )
            }
            Self::Discriminant::Workspace => Self::Workspace(str.parse().unwrap_or_default()),
            Self::Discriminant::OnWorkspace => Self::OnWorkspace(str.parse().unwrap_or_default()),
            Self::Discriminant::Content => Self::Content(str.parse().unwrap_or_default()),
            Self::Discriminant::XdgTag => Self::XdgTag(str.to_string()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            WindowRuleParameter::Class(class) => Some(class.clone()),
            WindowRuleParameter::Title(title) => Some(title.clone()),
            WindowRuleParameter::InitialClass(initial_class) => Some(initial_class.clone()),
            WindowRuleParameter::InitialTitle(initial_title) => Some(initial_title.clone()),
            WindowRuleParameter::Tag(tag) => Some(tag.clone()),
            WindowRuleParameter::Xwayland => Some("1".to_string()),
            WindowRuleParameter::NotXwayland => Some("0".to_string()),
            WindowRuleParameter::Floating => Some("1".to_string()),
            WindowRuleParameter::NotFloating => Some("0".to_string()),
            WindowRuleParameter::Fullscreen => Some("1".to_string()),
            WindowRuleParameter::NotFullscreen => Some("0".to_string()),
            WindowRuleParameter::Pinned => Some("1".to_string()),
            WindowRuleParameter::NotPinned => Some("0".to_string()),
            WindowRuleParameter::Focus => Some("1".to_string()),
            WindowRuleParameter::NotFocus => Some("0".to_string()),
            WindowRuleParameter::Group => Some("1".to_string()),
            WindowRuleParameter::NotGroup => Some("0".to_string()),
            WindowRuleParameter::FullscreenState(state1, state2) => {
                Some(format!("{} {}", state1, state2))
            }
            WindowRuleParameter::Workspace(workspace) => Some(workspace.to_string()),
            WindowRuleParameter::OnWorkspace(workspace_selector) => {
                Some(workspace_selector.to_string())
            }
            WindowRuleParameter::Content(content_type) => Some(content_type.to_string()),
            WindowRuleParameter::XdgTag(tag) => Some(tag.clone()),
        }
    }
}

impl Default for WindowRuleParameter {
    fn default() -> Self {
        Self::Class(String::new())
    }
}

impl FromStr for WindowRuleParameter {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (part1, part2) = s.split_once(':').unwrap_or((s, ""));

        match part1.trim() {
            "class" => Ok(WindowRuleParameter::Class(part2.to_string())),
            "title" => Ok(WindowRuleParameter::Title(part2.to_string())),
            "initialClass" => Ok(WindowRuleParameter::InitialClass(part2.to_string())),
            "initialTitle" => Ok(WindowRuleParameter::InitialTitle(part2.to_string())),
            "tag" => Ok(WindowRuleParameter::Tag(part2.to_string())),
            "xwayland" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleParameter::Xwayland),
                Some(false) => Ok(WindowRuleParameter::NotXwayland),
                None => Ok(WindowRuleParameter::Xwayland),
            },
            "floating" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleParameter::Floating),
                Some(false) => Ok(WindowRuleParameter::NotFloating),
                None => Ok(WindowRuleParameter::Floating),
            },
            "fullscreen" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleParameter::Fullscreen),
                Some(false) => Ok(WindowRuleParameter::NotFullscreen),
                None => Ok(WindowRuleParameter::Fullscreen),
            },
            "pinned" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleParameter::Pinned),
                Some(false) => Ok(WindowRuleParameter::NotPinned),
                None => Ok(WindowRuleParameter::Pinned),
            },
            "focus" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleParameter::Focus),
                Some(false) => Ok(WindowRuleParameter::NotFocus),
                None => Ok(WindowRuleParameter::Focus),
            },
            "group" => match parse_bool(part2) {
                Some(true) => Ok(WindowRuleParameter::Group),
                Some(false) => Ok(WindowRuleParameter::NotGroup),
                None => Ok(WindowRuleParameter::Group),
            },
            "fullscreenState" => {
                let (state1, state2) = part2.split_once(' ').unwrap_or((part2, ""));
                Ok(WindowRuleParameter::FullscreenState(
                    state1.parse().unwrap_or_default(),
                    state2.parse().unwrap_or_default(),
                ))
            }
            "workspace" => Ok(WindowRuleParameter::Workspace(
                part2.parse().unwrap_or_default(),
            )),
            "onworkspace" => Ok(WindowRuleParameter::OnWorkspace(
                part2.parse().unwrap_or_default(),
            )),
            "content" => Ok(WindowRuleParameter::Content(
                part2.parse().unwrap_or_default(),
            )),
            "xdgtag" => Ok(WindowRuleParameter::XdgTag(part2.to_string())),
            _ => Err(()),
        }
    }
}

impl Display for WindowRuleParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowRuleParameter::Class(class) => write!(f, "class:{}", class),
            WindowRuleParameter::Title(title) => write!(f, "title:{}", title),
            WindowRuleParameter::InitialClass(initial_class) => {
                write!(f, "initialClass:{}", initial_class)
            }
            WindowRuleParameter::InitialTitle(initial_title) => {
                write!(f, "initialTitle:{}", initial_title)
            }
            WindowRuleParameter::Tag(tag) => write!(f, "tag:{}", tag),
            WindowRuleParameter::Xwayland => write!(f, "xwayland:1"),
            WindowRuleParameter::NotXwayland => write!(f, "xwayland:0"),
            WindowRuleParameter::Floating => write!(f, "floating:1"),
            WindowRuleParameter::NotFloating => write!(f, "floating:0"),
            WindowRuleParameter::Fullscreen => write!(f, "fullscreen:1"),
            WindowRuleParameter::NotFullscreen => write!(f, "fullscreen:0"),
            WindowRuleParameter::Pinned => write!(f, "pinned:1"),
            WindowRuleParameter::NotPinned => write!(f, "pinned:0"),
            WindowRuleParameter::Focus => write!(f, "focus:1"),
            WindowRuleParameter::NotFocus => write!(f, "focus:0"),
            WindowRuleParameter::Group => write!(f, "group:1"),
            WindowRuleParameter::NotGroup => write!(f, "group:0"),
            WindowRuleParameter::FullscreenState(state1, state2) => {
                write!(f, "fullscreenState:{} {}", state1, state2)
            }
            WindowRuleParameter::Workspace(workspace) => write!(f, "workspace:{}", workspace),
            WindowRuleParameter::OnWorkspace(workspace_selector) => {
                write!(f, "onworkspace:{}", workspace_selector)
            }
            WindowRuleParameter::Content(content_type) => write!(f, "content:{}", content_type),
            WindowRuleParameter::XdgTag(tag) => write!(f, "xdgtag:{}", tag),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WindowRuleWithParameters {
    pub rule: WindowRule,
    pub parameters: Vec<WindowRuleParameter>,
}

impl FromStr for WindowRuleWithParameters {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
        if parts.is_empty() {
            return Err(());
        }

        let rule: WindowRule = parts[0].parse().unwrap_or_default();

        let parameters: Vec<WindowRuleParameter> = parts
            .iter()
            .skip(1)
            .filter_map(|s| s.parse::<WindowRuleParameter>().ok())
            .collect();

        Ok(WindowRuleWithParameters { rule, parameters })
    }
}

impl Display for WindowRuleWithParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}",
            self.rule,
            join_with_separator(&self.parameters, ", ")
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum OnOrOffOrUnset {
    #[default]
    Unset,
    On,
    Off,
}

impl FromStr for OnOrOffOrUnset {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_bool(s) {
            Some(true) => Ok(OnOrOffOrUnset::On),
            Some(false) => Ok(OnOrOffOrUnset::Off),
            None => Ok(OnOrOffOrUnset::Unset),
        }
    }
}

impl Display for OnOrOffOrUnset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OnOrOffOrUnset::Unset => write!(f, "unset"),
            OnOrOffOrUnset::On => write!(f, "1"),
            OnOrOffOrUnset::Off => write!(f, "0"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, EnumDiscriminants, Default)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(LayerRuleDiscriminant))]
pub enum LayerRule {
    #[default]
    Unset,
    NoAnim,
    Blur,
    BlurPopups,
    IgnoreAlpha(f64),
    IgnoreZero,
    DimAround,
    Xray(OnOrOffOrUnset),
    Animation(AnimationStyle),
    Order(i32),
    AboveLock,
    AboveLockInteractable,
}

impl HasDiscriminant for LayerRule {
    type Discriminant = LayerRuleDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            LayerRuleDiscriminant::Unset => LayerRule::Unset,
            LayerRuleDiscriminant::NoAnim => LayerRule::NoAnim,
            LayerRuleDiscriminant::Blur => LayerRule::Blur,
            LayerRuleDiscriminant::BlurPopups => LayerRule::BlurPopups,
            LayerRuleDiscriminant::IgnoreAlpha => LayerRule::IgnoreAlpha(0.0),
            LayerRuleDiscriminant::IgnoreZero => LayerRule::IgnoreZero,
            LayerRuleDiscriminant::DimAround => LayerRule::DimAround,
            LayerRuleDiscriminant::Xray => LayerRule::Xray(OnOrOffOrUnset::default()),
            LayerRuleDiscriminant::Animation => LayerRule::Animation(AnimationStyle::default()),
            LayerRuleDiscriminant::Order => LayerRule::Order(0),
            LayerRuleDiscriminant::AboveLock => LayerRule::AboveLock,
            LayerRuleDiscriminant::AboveLockInteractable => LayerRule::AboveLockInteractable,
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            LayerRuleDiscriminant::Unset => LayerRule::Unset,
            LayerRuleDiscriminant::NoAnim => LayerRule::NoAnim,
            LayerRuleDiscriminant::Blur => LayerRule::Blur,
            LayerRuleDiscriminant::BlurPopups => LayerRule::BlurPopups,
            LayerRuleDiscriminant::IgnoreAlpha => {
                LayerRule::IgnoreAlpha(str.parse().unwrap_or(0.0))
            }
            LayerRuleDiscriminant::IgnoreZero => LayerRule::IgnoreZero,
            LayerRuleDiscriminant::DimAround => LayerRule::DimAround,
            LayerRuleDiscriminant::Xray => {
                LayerRule::Xray(OnOrOffOrUnset::from_str(str).unwrap_or_default())
            }
            LayerRuleDiscriminant::Animation => {
                LayerRule::Animation(AnimationStyle::from_str(str).unwrap_or_default())
            }
            LayerRuleDiscriminant::Order => LayerRule::Order(str.parse().unwrap_or(0)),
            LayerRuleDiscriminant::AboveLock => LayerRule::AboveLock,
            LayerRuleDiscriminant::AboveLockInteractable => LayerRule::AboveLockInteractable,
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            LayerRule::Unset => None,
            LayerRule::NoAnim => None,
            LayerRule::Blur => None,
            LayerRule::BlurPopups => None,
            LayerRule::IgnoreAlpha(value) => Some(format!("{}", value)),
            LayerRule::IgnoreZero => None,
            LayerRule::DimAround => None,
            LayerRule::Xray(value) => Some(value.to_string()),
            LayerRule::Animation(value) => Some(value.to_string()),
            LayerRule::Order(value) => Some(value.to_string()),
            LayerRule::AboveLock => None,
            LayerRule::AboveLockInteractable => None,
        }
    }
}

impl FromStr for LayerRule {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        let (discriminant, str) = s.split_once(' ').unwrap_or((s, ""));
        let discriminant = discriminant.trim();
        let str = str.trim();

        match discriminant.to_lowercase().as_str() {
            "unset" => Ok(LayerRule::Unset),
            "noanim" => Ok(LayerRule::NoAnim),
            "blur" => Ok(LayerRule::Blur),
            "blurpopups" => Ok(LayerRule::BlurPopups),
            "ignorealpha" => Ok(LayerRule::IgnoreAlpha(
                str.parse().unwrap_or(0.0f64).clamp(0.0, 1.0),
            )),
            "ignorezero" => Ok(LayerRule::IgnoreZero),
            "dimaround" => Ok(LayerRule::DimAround),
            "xray" => Ok(LayerRule::Xray(
                OnOrOffOrUnset::from_str(str).unwrap_or_default(),
            )),
            "animation" => Ok(LayerRule::Animation(str.parse().unwrap_or_default())),
            "order" => Ok(LayerRule::Order(str.parse().unwrap_or(0))),
            "abovelock" => {
                if let Some(true) = parse_bool(str) {
                    Ok(LayerRule::AboveLockInteractable)
                } else {
                    Ok(LayerRule::AboveLock)
                }
            }
            _ => Err(()),
        }
    }
}

impl Display for LayerRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerRule::Unset => write!(f, "unset"),
            LayerRule::NoAnim => write!(f, "noanim"),
            LayerRule::Blur => write!(f, "blur"),
            LayerRule::BlurPopups => write!(f, "blurpopups"),
            LayerRule::IgnoreAlpha(value) => write!(f, "ignorealpha {}", value),
            LayerRule::IgnoreZero => write!(f, "ignorezero"),
            LayerRule::DimAround => write!(f, "dimaround"),
            LayerRule::Xray(value) => write!(f, "xray {}", value),
            LayerRule::Animation(value) => write!(f, "animation {}", value),
            LayerRule::Order(value) => write!(f, "order {}", value),
            LayerRule::AboveLock => write!(f, "abovelock"),
            LayerRule::AboveLockInteractable => write!(f, "abovelock true"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(NamespaceOrAddressDiscriminant))]
pub enum NamespaceOrAddress {
    Namespace(String),
    Address(String),
}

impl HasDiscriminant for NamespaceOrAddress {
    type Discriminant = NamespaceOrAddressDiscriminant;

    fn to_discriminant(&self) -> Self::Discriminant {
        self.into()
    }

    fn from_discriminant(discriminant: Self::Discriminant) -> Self {
        match discriminant {
            NamespaceOrAddressDiscriminant::Namespace => {
                NamespaceOrAddress::Namespace("".to_string())
            }
            NamespaceOrAddressDiscriminant::Address => NamespaceOrAddress::Address("".to_string()),
        }
    }

    fn from_discriminant_and_str(discriminant: Self::Discriminant, str: &str) -> Self {
        match discriminant {
            NamespaceOrAddressDiscriminant::Namespace => {
                NamespaceOrAddress::Namespace(str.to_string())
            }
            NamespaceOrAddressDiscriminant::Address => NamespaceOrAddress::Address(str.to_string()),
        }
    }

    fn to_str_without_discriminant(&self) -> Option<String> {
        match self {
            NamespaceOrAddress::Namespace(namespace) => Some(namespace.to_string()),
            NamespaceOrAddress::Address(address) => Some(address.to_string()),
        }
    }
}

impl Default for NamespaceOrAddress {
    fn default() -> Self {
        NamespaceOrAddress::Namespace("".to_string())
    }
}

impl FromStr for NamespaceOrAddress {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if let Some(address) = s.strip_prefix("address:0x") {
            Ok(NamespaceOrAddress::Address(address.to_string()))
        } else {
            Ok(NamespaceOrAddress::Namespace(s.to_string()))
        }
    }
}

impl Display for NamespaceOrAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NamespaceOrAddress::Namespace(namespace) => write!(f, "{}", namespace),
            NamespaceOrAddress::Address(address) => write!(f, "address:0x{}", address),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayerRuleWithParameter {
    pub rule: LayerRule,
    pub namespace_or_address: NamespaceOrAddress,
}

impl FromStr for LayerRuleWithParameter {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (rule_str, namespace_or_address_str) = s.split_once(',').unwrap_or((s, ""));

        let rule = rule_str.parse().unwrap_or_default();

        let namespace_or_address = namespace_or_address_str.parse().unwrap_or_default();

        Ok(LayerRuleWithParameter {
            rule,
            namespace_or_address,
        })
    }
}

impl Display for LayerRuleWithParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.rule, self.namespace_or_address)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExecWithRules {
    pub rules: Vec<WindowRule>,
    pub command: String,
}

impl FromStr for ExecWithRules {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let str = s.trim_start();

        if str.starts_with('[') {
            let mut rules = Vec::new();
            let mut rule = String::new();
            let mut in_brackets = false;
            let mut command = String::new();

            for c in str.chars() {
                if c == '[' {
                    in_brackets = true;
                } else if c == ']' {
                    if !rule.trim().is_empty() {
                        rules.push(rule.parse().unwrap_or_default());
                        rule.clear();
                    }
                    in_brackets = false;
                } else if c == ';' && in_brackets {
                    if !rule.trim().is_empty() {
                        rules.push(rule.parse().unwrap_or_default());
                        rule.clear();
                    }
                } else if in_brackets {
                    rule.push(c);
                } else {
                    command.push(c);
                }
            }

            Ok(Self {
                rules,
                command: command.trim_start().to_string(),
            })
        } else {
            Ok(Self {
                rules: Vec::new(),
                command: str.to_string(),
            })
        }
    }
}

impl Display for ExecWithRules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.rules.is_empty() {
            true => write!(f, "{}", self.command),
            false => write!(
                f,
                "[{}] {}",
                join_with_separator(&self.rules, "; "),
                self.command
            ),
        }
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_int(value: &str) -> Option<i32> {
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

/// 9007199254740992.0
pub const MAX_SAFE_INTEGER_F64: f64 = (1u64 << 53) as f64; // 2^53
/// -9007199254740992.0
pub const MIN_SAFE_INTEGER_F64: f64 = -MAX_SAFE_INTEGER_F64; // -2^53
/// 140737488355328
pub const MAX_SAFE_STEP_0_01_F64: f64 = (1u64 << 47) as f64; // 2^47
/// -140737488355328
pub const MIN_SAFE_STEP_0_01_F64: f64 = -MAX_SAFE_STEP_0_01_F64; // -2^47
