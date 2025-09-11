use std::collections::HashSet;
use std::error::Error;
use std::fs;
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
pub const BACKUP_SUFFIX: &str = "-bak";
