use crate::model::{Category, Section, Tool};
use crate::os::Os;
use crate::term;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy)]
pub struct RenderCtx {
    pub color: bool,
    pub plain: bool,
    pub width: usize,
}

impl Default for RenderCtx {
    fn default() -> Self {
        Self {
            color: false,
            plain: false,
            width: 80,
        }
    }
}

/// Helper to color command text in #96d0fb, highlighting placeholders `<...>` in yellow.
fn colorize_command(cmd_text: &str) -> String {
    let mut out = String::new();
    let mut in_placeholder = false;
    let mut in_cmd = false;

    for c in cmd_text.chars() {
        if c == '<' {
            if in_cmd {
                out.push_str(term::RESET);
                in_cmd = false;
            }
            out.push_str(term::YELLOW);
            out.push('<');
            in_placeholder = true;
        } else if c == '>' {
            out.push('>');
            out.push_str(term::RESET);
            in_placeholder = false;
        } else if in_placeholder {
            out.push(c);
        } else {
            if !in_cmd {
                out.push_str(term::CMD_COLOR);
                in_cmd = true;
            }
            out.push(c);
        }
    }
    if in_cmd || in_placeholder {
        out.push_str(term::RESET);
    }
    out
}

struct PreparedCommand {
    cmd_text: String,
    note_text: String,
}

pub fn usage_guide<W: Write>(out: &mut W, ctx: &RenderCtx) -> io::Result<()> {
    writeln!(out, "ezcmd {}", env!("CARGO_PKG_VERSION"))?;
    writeln!(out, "Find the right command, in order.\n")?;
    if ctx.color {
        writeln!(out, "{}HOW TO USE{}", term::CYAN_BOLD, term::RESET)?;
    } else {
        writeln!(out, "HOW TO USE")?;
    }

    let rows = [
        ("ezcmd <query>", "Search for a task or a command", None),
        (
            "ezcmd --help -<tool>",
            "List a tool's sections",
            Some("ezcmd --help -docker"),
        ),
        (
            "ezcmd --help -<tool> -<section>",
            "Show a section's commands",
            Some("ezcmd --help -docker -logs"),
        ),
        (
            "ezcmd --help -<section>",
            "Search every tool",
            Some("ezcmd --help -initialize"),
        ),
        (
            "ezcmd --help -<os>",
            "Browse as another OS",
            Some("ezcmd --help -windows"),
        ),
        ("ezcmd update", "Install the latest release", None),
    ];
    for (command, description, example) in rows {
        if ctx.width >= 80 {
            write!(out, "  {command:<36} {description:<30}")?;
            if let Some(example) = example {
                if ctx.color {
                    write!(out, "{}{}{}", term::DIM, example, term::RESET)?;
                } else {
                    write!(out, "{example}")?;
                }
            }
            writeln!(out)?;
        } else {
            writeln!(out, "  {command:<36} {description}")?;
        }
    }
    Ok(())
}

pub fn toc_overview<W: Write>(
    out: &mut W,
    categories: &[Category],
    os: Os,
    ctx: &RenderCtx,
) -> io::Result<()> {
    let visible_tools: Vec<&Tool> = categories
        .iter()
        .flat_map(|category| category.tools.iter())
        .filter(|tool| tool.available_on(os))
        .collect();
    let slug_width = visible_tools
        .iter()
        .map(|tool| tool.slug.chars().count())
        .max()
        .unwrap_or(0);
    let name_width = visible_tools
        .iter()
        .map(|tool| tool.name.chars().count())
        .max()
        .unwrap_or(0);

    let mut first_category = true;
    for category in categories {
        let tools: Vec<&Tool> = category
            .tools
            .iter()
            .filter(|tool| tool.available_on(os))
            .collect();
        if tools.is_empty() {
            continue;
        }
        if !first_category {
            writeln!(out)?;
        }
        if ctx.color {
            writeln!(out, "{}{}{}", term::CYAN_BOLD, category.name, term::RESET)?;
        } else {
            writeln!(out, "{}", category.name)?;
        }
        for tool in tools {
            let count = tool
                .sections()
                .filter(|section| section.available_on(os))
                .count();
            if ctx.color {
                write!(out, "  {}-{}{}", term::BOLD, tool.slug, term::RESET)?;
            } else {
                write!(out, "  -{}", tool.slug)?;
            }
            let slug_padding = " ".repeat(slug_width.saturating_sub(tool.slug.chars().count()) + 2);
            let name_padding = " ".repeat(name_width.saturating_sub(tool.name.chars().count()) + 2);
            write!(out, "{slug_padding}{}{name_padding}", tool.name)?;
            if ctx.color {
                writeln!(out, "{}{} sections{}", term::DIM, count, term::RESET)?;
            } else {
                writeln!(out, "{count} sections")?;
            }
        }
        first_category = false;
    }
    Ok(())
}

pub fn toc_tool<W: Write>(out: &mut W, tool: &Tool, os: Os, ctx: &RenderCtx) -> io::Result<()> {
    if ctx.color {
        writeln!(
            out,
            "{}{}{}  (-{})",
            term::CYAN_BOLD,
            tool.name,
            term::RESET,
            tool.slug
        )?;
    } else {
        writeln!(out, "{}  (-{})", tool.name, tool.slug)?;
    }
    if !tool.aliases.is_empty() {
        writeln!(out, "Also matches: {}", tool.aliases.join(", "))?;
    }
    for note in tool.notes {
        writeln!(out, "Note: {note}")?;
    }

    let groups: Vec<_> = tool
        .groups
        .iter()
        .filter(|group| {
            group
                .sections
                .iter()
                .any(|section| section.available_on(os))
        })
        .collect();
    let show_group_titles = groups.len() > 1;
    for (group_index, group) in groups.iter().enumerate() {
        writeln!(out)?;
        if show_group_titles {
            if ctx.color {
                writeln!(out, "{}{}{}", term::BOLD, group.title, term::RESET)?;
            } else {
                writeln!(out, "{}", group.title)?;
            }
        }
        for section in group
            .sections
            .iter()
            .filter(|section| section.available_on(os))
        {
            if ctx.color {
                writeln!(out, "  {}-{} {}", term::DIM, term::RESET, section.title)?;
            } else {
                writeln!(out, "  -{}", section.title)?;
            }
        }
        if group_index + 1 == groups.len() {
            writeln!(out)?;
        }
    }
    writeln!(
        out,
        "View a section:  ezcmd --help -{} -<section>",
        tool.slug
    )?;
    writeln!(
        out,
        "The section name can be shortened or misspelled. It is fuzzy-matched."
    )?;
    Ok(())
}

/// Render a single section for the given OS.
pub fn render_section<W: Write>(
    w: &mut W,
    tool: &Tool,
    section: &Section,
    os: Os,
    ctx: &RenderCtx,
) -> io::Result<()> {
    // If no command resolves for the OS, omit the whole section (Section 9.4)
    if !section.available_on(os) {
        return Ok(());
    }

    let prepared: Vec<PreparedCommand> = section
        .visible_commands(os)
        .filter_map(|command| {
            command.resolve(os).map(|resolved| PreparedCommand {
                cmd_text: resolved.to_string(),
                note_text: command.note.to_string(),
            })
        })
        .collect();

    let title = format!("{}: {}", tool.name, section.title);

    // Plain mode (--plain): `# ---Tool: Section---` then `command --- note`
    if ctx.plain {
        writeln!(w, "# ---{}---", title)?;
        for p in prepared {
            writeln!(w, "{} --- {}", p.cmd_text, p.note_text)?;
        }
        return Ok(());
    }

    // Determine cmd_w = longest command column text, capped at 48
    let max_cmd_len = prepared
        .iter()
        .map(|p| p.cmd_text.chars().count())
        .max()
        .unwrap_or(0);
    let cmd_w = max_cmd_len.min(48);

    // Longest note
    let longest_note = prepared
        .iter()
        .map(|p| p.note_text.chars().count())
        .max()
        .unwrap_or(0);

    // Wide layout if `3 + cmd_w + 2 + 4 + longest_note <= term_width`
    let is_wide = (3 + cmd_w + 6 + longest_note) <= ctx.width;

    // Calculate widest line
    let title_line_len = 8 + title.chars().count();
    let mut widest_line = title_line_len;

    if is_wide {
        for p in &prepared {
            let line_len = 3 + cmd_w + 6 + p.note_text.chars().count();
            if line_len > widest_line {
                widest_line = line_len;
            }
        }
    } else {
        for p in &prepared {
            let cmd_len = 3 + p.cmd_text.chars().count();
            let note_len = 11 + p.note_text.chars().count();
            if cmd_len > widest_line {
                widest_line = cmd_len;
            }
            if note_len > widest_line {
                widest_line = note_len;
            }
        }
    }

    // Frame width = clamp(widest_line + 2, 60, max(term_width, 60))
    // Frame width = clamp(widest_line + 2, 60, max(term_width, 60))
    let max_width = ctx.width.max(60);
    let frame_width = (widest_line + 2).clamp(60, max_width);

    // Top rule: ` ___[ ` + title + ` ]` + `_` to fill width, ending with `|`
    let prefix = " ___[ ";
    let suffix = " ]";
    let fixed_title_len = prefix.chars().count() + title.chars().count() + suffix.chars().count();
    let trailing_underscores = frame_width.saturating_sub(fixed_title_len + 1);

    if ctx.color {
        writeln!(
            w,
            " {}___[{} {}{}{} {}]{}{}|{}",
            term::DIM,
            term::RESET,
            term::CYAN_BOLD,
            title,
            term::RESET,
            term::DIM,
            "_".repeat(trailing_underscores),
            term::DIM,
            term::RESET
        )?;
        writeln!(
            w,
            "{}|{}{}|{}",
            term::DIM,
            " ".repeat(frame_width.saturating_sub(2)),
            term::DIM,
            term::RESET
        )?;
    } else {
        writeln!(w, " ___[ {} ]{}|", title, "_".repeat(trailing_underscores))?;
        writeln!(w, "|{}|", " ".repeat(frame_width.saturating_sub(2)))?;
    }

    // Content lines
    for p in prepared {
        if is_wide {
            let cmd_len = p.cmd_text.chars().count();
            let pad_len = cmd_w.saturating_sub(cmd_len);
            let pad = " ".repeat(pad_len);
            let line_content_len = 3 + cmd_w + 6 + p.note_text.chars().count();
            let right_pad = (frame_width.saturating_sub(1)).saturating_sub(line_content_len);

            if ctx.color {
                let colored_cmd = format!("{}{}", colorize_command(&p.cmd_text), pad);

                writeln!(
                    w,
                    "{}|{}  {}  {}---{} {}{}{}|{}",
                    term::DIM,
                    term::RESET,
                    colored_cmd,
                    term::DIM,
                    term::RESET,
                    p.note_text,
                    " ".repeat(right_pad),
                    term::DIM,
                    term::RESET
                )?;
            } else {
                writeln!(
                    w,
                    "|  {}{}  --- {}{}|",
                    p.cmd_text,
                    pad,
                    p.note_text,
                    " ".repeat(right_pad)
                )?;
            }
        } else {
            // Narrow layout
            let cmd_len = p.cmd_text.chars().count();
            let cmd_right_pad = (frame_width.saturating_sub(1)).saturating_sub(3 + cmd_len);

            let note_len = p.note_text.chars().count();
            let note_right_pad = (frame_width.saturating_sub(1)).saturating_sub(11 + note_len);

            if ctx.color {
                let colored_cmd = colorize_command(&p.cmd_text);

                writeln!(
                    w,
                    "{}|{}  {}{}{}|{}",
                    term::DIM,
                    term::RESET,
                    colored_cmd,
                    " ".repeat(cmd_right_pad),
                    term::DIM,
                    term::RESET
                )?;
                writeln!(
                    w,
                    "{}|      ---{} {}{}{}|{}",
                    term::DIM,
                    term::RESET,
                    p.note_text,
                    " ".repeat(note_right_pad),
                    term::DIM,
                    term::RESET
                )?;
            } else {
                writeln!(w, "|  {}{}|", p.cmd_text, " ".repeat(cmd_right_pad))?;
                writeln!(
                    w,
                    "|      --- {}{}|",
                    p.note_text,
                    " ".repeat(note_right_pad)
                )?;
            }
        }
    }

    // Bottom rule: `|` + `_` to fill width, ending with `|`
    let bottom_underscores = "_".repeat(frame_width.saturating_sub(2));
    if ctx.color {
        writeln!(w, "{}|{}|{}", term::DIM, bottom_underscores, term::RESET)?;
    } else {
        writeln!(w, "|{}|", bottom_underscores)?;
    }

    Ok(())
}

/// Render multiple sections, separated by a blank line.
pub fn render_sections<W: Write>(
    w: &mut W,
    tool: &Tool,
    sections: &[&Section],
    os: Os,
    ctx: &RenderCtx,
) -> io::Result<()> {
    let mut first = true;
    for section in sections {
        if !section.available_on(os) {
            continue;
        }
        if !first {
            writeln!(w)?;
        }
        render_section(w, tool, section, os, ctx)?;
        first = false;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Command;

    #[test]
    fn test_colorize_command() {
        let plain = "git clone <url>";
        let colored = colorize_command(plain);
        assert!(colored.contains(term::CMD_COLOR));
        assert!(colored.contains(term::YELLOW));
        assert!(colored.contains("<url>"));
    }

    #[test]
    fn test_render_plain_mode() {
        static CMD: [Command; 1] = [Command {
            note: "Initialize repo",
            cmd: Some("git init"),
            unix: None,
            windows: None,
            linux: None,
            macos: None,
        }];
        let sec = Section {
            title: "Init",
            aliases: &["init"],
            commands: &CMD,
        };
        let tool = Tool {
            slug: "git",
            name: "Git",
            category: "Test",
            aliases: &[],
            flag_aliases: &[],
            platforms: &[],
            notes: &[],
            groups: &[],
        };
        let ctx = RenderCtx {
            color: false,
            plain: true,
            width: 80,
        };
        let mut buf = Vec::new();
        render_section(&mut buf, &tool, &sec, Os::Linux, &ctx).expect("render");
        let output = String::from_utf8(buf).expect("utf8");
        assert_eq!(output, "# ---Git: Init---\ngit init --- Initialize repo\n");
    }
}
