use crate::utils::markdown_to_pango;
use gtk::{Align, Box, Frame, Grid, Label, Orientation, pango::WrapMode, prelude::*};
use rust_i18n::{locale, t};

#[derive(Debug)]
enum ContentBlock {
    Text(String),
    Code {
        language: String,
        content: String,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Callout {
        callout_type: String,
        content: Vec<ContentBlock>,
    },
}

pub fn create_guide(name: &str) -> Box {
    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_start(15)
        .margin_end(15)
        .margin_top(10)
        .margin_bottom(10)
        .build();

    for block in get_content(name) {
        match block {
            ContentBlock::Text(text) => {
                let text_label = create_text_label(&text);
                main_box.append(&text_label);
                if text.contains("<span size=\"large\" weight=\"bold\">") {
                    let frame = Frame::builder().build();
                    main_box.append(&frame);
                }
            }
            ContentBlock::Code { language, content } => {
                let code_frame = create_code_frame(&language, &content);
                main_box.append(&code_frame);
            }
            ContentBlock::Table { headers, rows } => {
                let table_frame = create_table_frame(&headers, &rows);
                main_box.append(&table_frame);
            }
            ContentBlock::Callout {
                callout_type,
                content,
            } => {
                let callout_frame = create_callout_frame(&callout_type, &content);
                main_box.append(&callout_frame);
            }
        }
    }

    main_box
}

fn get_content(name: &str) -> Vec<ContentBlock> {
    let current_locale = locale().to_string();
    let content = match name {
        "Dispatchers" => match current_locale.as_str() {
            "ru" => include_str!("../guides/ru/Dispatchers.md"),
            "zh-CN" => include_str!("../guides/zh-CN/Dispatchers.md"),
            _ => include_str!("../guides/en/Dispatchers.md"),
        },
        "Dwindle-Layout" => match current_locale.as_str() {
            "ru" => include_str!("../guides/ru/Dwindle-Layout.md"),
            "zh-CN" => include_str!("../guides/zh-CN/Dwindle-Layout.md"),
            _ => include_str!("../guides/en/Dwindle-Layout.md"),
        },
        "Master-Layout" => match current_locale.as_str() {
            "ru" => include_str!("../guides/ru/Master-Layout.md"),
            "zh-CN" => include_str!("../guides/zh-CN/Master-Layout.md"),
            _ => include_str!("../guides/en/Master-Layout.md"),
        },
        "Monitors" => match current_locale.as_str() {
            "ru" => include_str!("../guides/ru/Monitors.md"),
            "zh-CN" => include_str!("../guides/zh-CN/Monitors.md"),
            _ => include_str!("../guides/en/Monitors.md"),
        },
        "Workspace-Rules" => match current_locale.as_str() {
            "ru" => include_str!("../guides/ru/Workspace-Rules.md"),
            "zh-CN" => include_str!("../guides/zh-CN/Workspace-Rules.md"),
            _ => include_str!("../guides/en/Workspace-Rules.md"),
        },
        "Animations" => match current_locale.as_str() {
            "ru" => include_str!("../guides/ru/Animations.md"),
            "zh-CN" => include_str!("../guides/zh-CN/Animations.md"),
            _ => include_str!("../guides/en/Animations.md"),
        },
        "Binds" => match current_locale.as_str() {
            "ru" => include_str!("../guides/ru/Binds.md"),
            "zh-CN" => include_str!("../guides/zh-CN/Binds.md"),
            _ => include_str!("../guides/en/Binds.md"),
        },
        "Gestures" => match current_locale.as_str() {
            "ru" => include_str!("../guides/ru/Gestures.md"),
            "zh-CN" => include_str!("../guides/zh-CN/Gestures.md"),
            _ => include_str!("../guides/en/Gestures.md"),
        },
        "Window-Rules" => match current_locale.as_str() {
            "ru" => include_str!("../guides/ru/Window-Rules.md"),
            "zh-CN" => include_str!("../guides/zh-CN/Window-Rules.md"),
            _ => include_str!("../guides/en/Window-Rules.md"),
        },
        "Layer-Rules" => match current_locale.as_str() {
            "ru" => include_str!("../guides/ru/Layer-Rules.md"),
            "zh-CN" => include_str!("../guides/zh-CN/Layer-Rules.md"),
            _ => include_str!("../guides/en/Layer-Rules.md"),
        },
        "Execs" => match current_locale.as_str() {
            "ru" => include_str!("../guides/ru/Execs.md"),
            "zh-CN" => include_str!("../guides/zh-CN/Execs.md"),
            _ => include_str!("../guides/en/Execs.md"),
        },
        "Environment-variables" => match current_locale.as_str() {
            "ru" => include_str!("../guides/ru/Environment-variables.md"),
            "zh-CN" => include_str!("../guides/zh-CN/Environment-variables.md"),
            _ => include_str!("../guides/en/Environment-variables.md"),
        },
        name => panic!("Invalid content name: {name}"),
    };
    let lines: Vec<&str> = content.lines().collect();
    parse_lines(&lines, name)
}

fn parse_lines(lines: &[&str], guide_name: &str) -> Vec<ContentBlock> {
    let mut blocks = Vec::new();
    let mut current_text = String::new();
    let mut in_code_block = false;
    let mut code_block_lang = String::new();
    let mut code_lines = Vec::new();
    let mut table_lines = Vec::new();
    let mut in_callout = false;
    let mut callout_type = String::new();
    let mut callout_lines = Vec::<&str>::new();

    for line in lines {
        if line.starts_with("weight:") || line.starts_with("title:") || line == &"---" {
            continue;
        }

        if in_callout {
            if let Some(callout_line) = line.strip_prefix("> ") {
                callout_lines.push(callout_line);
                continue;
            } else if *line == ">" {
                callout_lines.push("");
                continue;
            } else {
                let callout_blocks = parse_lines(&callout_lines, guide_name);
                blocks.push(ContentBlock::Callout {
                    callout_type: callout_type.clone(),
                    content: callout_blocks,
                });
                callout_type.clear();
                callout_lines.clear();
                in_callout = false;
            }
        }

        if line.trim_start().starts_with("> [!")
            && let (Some(bang_idx), Some(close_bracket_idx)) = (line.find('!'), line.find(']'))
        {
            callout_type = line[bang_idx + 1..close_bracket_idx].trim().to_lowercase();

            in_callout = true;
            continue;
        }

        if let Some(stripped) = line.trim_start().strip_prefix("> ") {
            callout_type = String::new();

            in_callout = true;
            callout_lines.push(stripped);
            continue;
        }

        if let Some(stripped) = line.strip_prefix("```") {
            if in_code_block {
                if !code_lines.is_empty() {
                    blocks.push(ContentBlock::Code {
                        language: code_block_lang.clone(),
                        content: code_lines.join("\n"),
                    });
                }
                code_block_lang.clear();
                code_lines.clear();
            } else {
                code_block_lang = stripped.trim().to_string();
            }
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            code_lines.push(line.to_string());
            continue;
        }

        if line.starts_with("|") {
            table_lines.push(line.to_string());
            continue;
        } else if !table_lines.is_empty() {
            if let Some(table) = parse_table(&table_lines, guide_name) {
                blocks.push(ContentBlock::Table {
                    headers: table.0,
                    rows: table.1,
                });
            }
            table_lines.clear();
        }

        if !line.trim().is_empty() {
            let processed_line = markdown_to_pango(line, guide_name);
            current_text.push_str(&processed_line);
            current_text.push('\n');
        } else if !current_text.is_empty() {
            blocks.push(ContentBlock::Text(
                current_text.trim_end_matches('\n').to_string(),
            ));
            current_text.clear();
        }
    }

    if !current_text.is_empty() {
        blocks.push(ContentBlock::Text(current_text));
    }

    if in_code_block && !code_lines.is_empty() {
        blocks.push(ContentBlock::Code {
            language: code_block_lang,
            content: code_lines.join("\n"),
        });
    }

    if !table_lines.is_empty()
        && let Some(table) = parse_table(&table_lines, guide_name)
    {
        blocks.push(ContentBlock::Table {
            headers: table.0,
            rows: table.1,
        });
    }

    if in_callout && !callout_lines.is_empty() {
        let callout_blocks = parse_lines(&callout_lines, guide_name);
        blocks.push(ContentBlock::Callout {
            callout_type: callout_type.clone(),
            content: callout_blocks,
        });
    }

    blocks
}

fn parse_table(lines: &[String], guide_name: &str) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    if lines.len() < 2 {
        return None;
    }

    let header_line = lines[0].as_str();
    let separator_line = lines[1].as_str();

    let headers: Vec<String> = header_line
        .trim_matches('|')
        .split('|')
        .map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                trimmed.to_string()
            } else {
                markdown_to_pango(trimmed, guide_name)
            }
        })
        .filter(|s| !s.is_empty())
        .collect();

    let separators: Vec<&str> = separator_line
        .trim_matches('|')
        .split('|')
        .map(|s| s.trim())
        .collect();

    if headers.len() != separators.len() {
        return None;
    }

    let mut rows = Vec::new();
    for line in &lines[2..] {
        let cells: Vec<String> = line
            .trim_matches('|')
            .split('|')
            .map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    trimmed.to_string()
                } else {
                    markdown_to_pango(trimmed, guide_name)
                }
            })
            .filter(|s| !s.is_empty())
            .collect();

        if cells.len() == headers.len() {
            rows.push(cells);
        }
    }

    Some((headers, rows))
}

fn create_text_label(text: &str) -> Label {
    let label = Label::builder()
        .use_markup(true)
        .wrap(true)
        .wrap_mode(WrapMode::Word)
        .selectable(true)
        .halign(Align::Start)
        .build();

    label.set_markup(text);

    label
}

fn create_code_frame(language: &str, content: &str) -> Frame {
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    let header = Label::builder()
        .label(language)
        .selectable(true)
        .halign(Align::Start)
        .margin_start(30)
        .margin_end(25)
        .margin_top(2)
        .margin_bottom(0)
        .build();

    vbox.append(&header);

    let frame = Frame::builder().build();
    vbox.append(&frame);

    let code_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .margin_start(20)
        .margin_end(20)
        .margin_top(5)
        .margin_bottom(20)
        .build();

    let code_label = Label::builder()
        .label(content)
        .wrap(true)
        .wrap_mode(WrapMode::Char)
        .selectable(true)
        .halign(Align::Start)
        .valign(Align::Center)
        .xalign(0.0)
        .build();
    code_box.append(&code_label);

    vbox.append(&code_box);

    let frame = Frame::builder().margin_start(15).margin_end(15).build();
    frame.set_child(Some(&vbox));

    frame
}

fn create_table_grid(headers: &[String], rows: &[Vec<String>]) -> Grid {
    let grid = Grid::builder()
        .row_spacing(2)
        .column_spacing(2)
        .margin_start(10)
        .margin_end(10)
        .margin_top(5)
        .margin_bottom(5)
        .build();

    for (col, header) in headers.iter().enumerate() {
        let label = Label::builder()
            .use_markup(true)
            .label(header)
            .wrap(true)
            .wrap_mode(WrapMode::Word)
            .selectable(true)
            .halign(Align::Center)
            .valign(Align::Center)
            .margin_start(5)
            .margin_end(5)
            .margin_top(3)
            .margin_bottom(3)
            .build();

        grid.attach(&label, col as i32, 0, 1, 1);
    }

    for (row_idx, row) in rows.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            let label = Label::builder()
                .use_markup(true)
                .label(cell)
                .wrap(true)
                .wrap_mode(WrapMode::Word)
                .selectable(true)
                .halign(Align::Start)
                .valign(Align::Center)
                .xalign(0.0)
                .margin_start(5)
                .margin_end(5)
                .margin_top(3)
                .margin_bottom(3)
                .build();

            if col_idx == 0 {
                label.set_max_width_chars(15);
                label.set_wrap(true);
                label.set_wrap_mode(WrapMode::Char);
            }

            grid.attach(&label, col_idx as i32, (row_idx + 1) as i32, 1, 1);
        }
    }

    grid
}

fn create_table_frame(headers: &[String], rows: &[Vec<String>]) -> Frame {
    let grid = create_table_grid(headers, rows);

    let inner_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .margin_start(10)
        .margin_end(10)
        .margin_top(5)
        .margin_bottom(5)
        .build();

    inner_box.append(&grid);

    let frame = Frame::builder().margin_start(15).margin_end(15).build();

    frame.set_child(Some(&inner_box));

    frame
}

fn create_callout_frame(callout_type: &str, content_blocks: &[ContentBlock]) -> Frame {
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(5)
        .margin_start(20)
        .margin_end(20)
        .margin_top(5)
        .margin_bottom(20)
        .build();

    let (title, style_class) = match callout_type.to_lowercase().as_str() {
        "info" | "note" | "tip" | "important" => {
            (&format!("<b>{}</b>", t!("guides.information")), "info")
        }
        "warning" | "caution" => (&format!("<b>{}</b>", t!("guides.warning")), "warning"),
        "error" | "danger" => (&format!("<b>{}</b>", t!("guides.error")), "error"),
        _ => (&format!("<b>{}</b>", callout_type), "card"),
    };

    let header = Label::builder()
        .use_markup(true)
        .selectable(true)
        .halign(Align::Start)
        .margin_start(10)
        .margin_end(25)
        .margin_top(2)
        .margin_bottom(0)
        .build();
    header.set_markup(title);

    if !title.is_empty() {
        vbox.append(&header);
    }

    for block in content_blocks {
        match block {
            ContentBlock::Text(text) => {
                let text_label = create_text_label(text);
                vbox.append(&text_label);
                if text.contains("<span size=\"large\" weight=\"bold\">") {
                    let frame = Frame::builder().build();
                    vbox.append(&frame);
                }
            }
            ContentBlock::Code { language, content } => {
                let code_frame = create_code_frame(language, content);
                vbox.append(&code_frame);
            }
            ContentBlock::Table { headers, rows } => {
                let table_frame = create_table_frame(headers, rows);
                vbox.append(&table_frame);
            }
            ContentBlock::Callout {
                callout_type,
                content,
            } => {
                let nested_callout = create_callout_frame(callout_type, content);
                vbox.append(&nested_callout);
            }
        }
    }

    let frame = Frame::builder().margin_start(15).margin_end(15).build();

    frame.add_css_class(style_class);

    frame.set_child(Some(&vbox));

    frame
}
