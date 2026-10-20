use crate::context::Context;
use crate::model::{Category, Command, Section, Tool};
use crate::model::{Recipe, RecipeKind, Risk};
use crate::os::Os;
use crate::recommend::Frame;
use crate::shell::Shell;
use crate::term;
use crate::tuning;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy)]
pub struct RenderCtx {
    pub color: bool,
    pub plain: bool,
    pub width: usize,
    pub shell: Shell,
}

impl Default for RenderCtx {
    fn default() -> Self {
        Self {
            color: false,
            plain: false,
            width: 80,
            shell: Shell::Cmd,
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
        ("ezcmd --forget", "Clear local recommendation history", None),
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
        .filter(|tool| tool.available_for(os, ctx.shell))
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
            .filter(|tool| tool.available_for(os, ctx.shell))
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
                .filter(|section| section.available_for(os, ctx.shell))
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
                .any(|section| section.available_for(os, ctx.shell))
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
            .filter(|section| section.available_for(os, ctx.shell))
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
    if !section.available_for(os, ctx.shell) {
        return Ok(());
    }

    let prepared: Vec<PreparedCommand> = section
        .visible_commands_for(os, ctx.shell)
        .filter_map(|command| {
            command
                .resolve_for(os, ctx.shell)
                .map(|resolved| PreparedCommand {
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
        if !section.available_for(os, ctx.shell) {
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

/// Render a focused data recipe.  The established section renderer above is
/// intentionally left alone for `--help` and legacy workflow recipes.
pub fn render_recipe<W: Write>(
    w: &mut W,
    recipe: &Recipe,
    frame: &Frame,
    context: &Context,
    os: Os,
    ctx: &RenderCtx,
    brief: bool,
) -> io::Result<()> {
    let replace = |value: &str| substitute(value, frame, context);
    let mut lines: Vec<(String, String)> = Vec::new();
    let mut selected_single = None;
    match recipe.kind {
        RecipeKind::Compose => {
            let Some(command) = compose_command(recipe, frame, context, os, ctx.shell) else {
                return Ok(());
            };
            lines.push((
                command,
                recipe
                    .base
                    .map(|base| base.note.to_string())
                    .unwrap_or_default(),
            ));
        }
        RecipeKind::Single => {
            let mut chosen = recipe.primary;
            for variant in recipe.variants {
                if !variant.matches.is_empty()
                    && variant
                        .matches
                        .iter()
                        .any(|term| query_has(&frame.tokens, term))
                {
                    chosen = Some(*variant);
                    break;
                }
            }
            selected_single = chosen;
            if let Some(command) = chosen.and_then(|command| {
                command
                    .resolve_for(os, ctx.shell)
                    .map(|text| (command, replace(text)))
            }) {
                lines.push((
                    format!("{}{}", risk_marker(command.0.risk, ctx.plain), command.1),
                    command.0.note.to_string(),
                ));
            }
        }
    }
    if ctx.plain {
        writeln!(w, "# ---{}---", recipe.title)?;
        for (command, note) in &lines {
            writeln!(w, "{} --- {}", command, note)?;
        }
        if let Some(command) = selected_single.filter(|command| command.risk == Risk::Danger) {
            writeln!(
                w,
                "DANGER: {}",
                command.warn.unwrap_or("This command can lose data.")
            )?;
            if let Some(pre) = command.pre {
                writeln!(w, "preview first: {}", replace(pre))?;
            }
        }
        if frame.denied_path && recipe.action == Some("delete") {
            writeln!(
                w,
                "ezcmd never fills in system or home paths for deleting commands."
            )?;
        }
        for notice in compose_conflict_notices(recipe, frame) {
            writeln!(w, "Note: {notice}")?;
        }
        if !brief && show_compose_alternatives(recipe, frame) {
            render_compose_alternatives_plain(w, recipe, frame, context, os, ctx.shell)?;
        }
        if !brief && recipe.kind == RecipeKind::Single {
            render_variants_plain(w, recipe, frame, context, selected_single, os, ctx)?;
        }
        if !brief && !is_specific_compose_request(recipe, frame) {
            render_extras_plain(w, recipe, frame, context)?;
        }
        return Ok(());
    }
    let mut output_lines: Vec<String> = lines
        .iter()
        .map(|(command, note)| format!("  {command}  --- {note}"))
        .collect();
    if let Some(command) = selected_single.filter(|command| command.risk == Risk::Danger) {
        output_lines.push(format!(
            "     DANGER: {}",
            command.warn.unwrap_or("This command can lose data.")
        ));
        if let Some(pre) = command.pre {
            output_lines.push(format!("     preview first: {}", replace(pre)));
        }
    }
    if !brief && show_compose_alternatives(recipe, frame) {
        output_lines.push("".into());
        output_lines.push("  Alternatives".into());
        for (command, note) in compose_alternatives(recipe, frame, context, os, ctx.shell) {
            output_lines.push(format!("  {command}  --- {note}"));
        }
    }
    if !brief && recipe.kind == RecipeKind::Single {
        let variants: Vec<_> = variants_in_display_order(recipe)
            .into_iter()
            .filter(|variant| Some(**variant) != selected_single)
            .filter(|variant| variant.resolve_for(os, ctx.shell).is_some())
            .take(tuning::MAX_VARIANTS)
            .collect();
        if !variants.is_empty() {
            output_lines.push("".into());
            output_lines.push("  Alternatives".into());
            for variant in variants {
                let command = variant
                    .resolve_for(os, ctx.shell)
                    .expect("filtered to commands available for this platform");
                if let Some(pre) = variant.pre {
                    output_lines.push(format!("  preview first: {}", replace(pre)));
                }
                let marker = risk_marker(variant.risk, false);
                output_lines.push(format!(
                    "  {marker}{}  --- {}",
                    replace(command),
                    variant.when.unwrap_or(variant.note)
                ));
                if variant.risk == Risk::Danger {
                    output_lines.push(format!(
                        "     DANGER: {}",
                        variant.warn.unwrap_or("This command can lose data.")
                    ));
                }
            }
        }
    }
    if frame.denied_path && recipe.action == Some("delete") {
        output_lines
            .push("  ezcmd never fills in system or home paths for deleting commands.".into());
    }
    for notice in compose_conflict_notices(recipe, frame) {
        output_lines.push(format!("  Note: {notice}"));
    }
    if !brief && !is_specific_compose_request(recipe, frame) {
        append_extras(&mut output_lines, recipe, frame, context);
    }
    render_box(w, recipe.title, &output_lines, ctx)
}

/// Compose exactly one command from the base and the options selected in the
/// frame. It is shared by the main answer and the broad-query alternatives so
/// the same group/default rules apply to both.
fn compose_command(
    recipe: &Recipe,
    frame: &Frame,
    context: &Context,
    os: Os,
    shell: Shell,
) -> Option<String> {
    let base = recipe
        .base
        .and_then(|command| command.resolve_for(os, shell))?;
    let mut command = substitute(base, frame, context);
    let mut selected_groups: Vec<&str> = Vec::new();
    for option in recipe.options {
        if frame.options.iter().any(|matched| matched == option.id) {
            if let Some(group) = option.group {
                let latest = recipe
                    .options
                    .iter()
                    .filter(|candidate| candidate.group == Some(group))
                    .filter_map(|candidate| {
                        frame
                            .options
                            .iter()
                            .position(|selected| selected == candidate.id)
                            .map(|position| (position, candidate.id))
                    })
                    .max_by_key(|(position, _)| *position)
                    .map(|(_, id)| id);
                if latest != Some(option.id) || selected_groups.contains(&group) {
                    continue;
                }
                selected_groups.push(group);
            }
            command.push(' ');
            command.push_str(option.add);
        }
    }
    // A required default represents the normal choice for an exclusive group.
    // Apply it only when the user selected no member of that group.
    for option in recipe.options {
        let Some(group) = option.group else {
            continue;
        };
        if option.required_default
            && !recipe.options.iter().any(|candidate| {
                candidate.group == Some(group)
                    && frame
                        .options
                        .iter()
                        .any(|selected| selected == candidate.id)
            })
            && !selected_groups.contains(&group)
        {
            command.push(' ');
            command.push_str(option.add);
            selected_groups.push(group);
        }
    }
    Some(command)
}

/// Explicit options make a compose query specific. The resolved command is
/// then the complete answer, rather than a menu of alternatives that the user
/// has already ruled out.
fn is_specific_compose_request(recipe: &Recipe, frame: &Frame) -> bool {
    recipe.kind == RecipeKind::Compose && !frame.options.is_empty()
}

fn show_compose_alternatives(recipe: &Recipe, frame: &Frame) -> bool {
    recipe.kind == RecipeKind::Compose
        && !is_specific_compose_request(recipe, frame)
        && recipe.options.iter().any(|option| !option.suggest)
}

/// For a broad compose query, show a few common option forms alongside the
/// plain base command. Suggested options remain compact hints, preserving the
/// product's cap while exposing the meaningful alternatives a newcomer needs.
fn compose_alternatives(
    recipe: &Recipe,
    frame: &Frame,
    context: &Context,
    os: Os,
    shell: Shell,
) -> Vec<(String, String)> {
    recipe
        .options
        .iter()
        .filter(|option| !option.suggest)
        .take(tuning::MAX_VARIANTS)
        .filter_map(|option| {
            let mut alternative = frame.clone();
            alternative.options = vec![option.id.to_string()];
            compose_command(recipe, &alternative, context, os, shell)
                .map(|command| (command, format!("Add {}", option.add)))
        })
        .collect()
}

fn render_compose_alternatives_plain<W: Write>(
    w: &mut W,
    recipe: &Recipe,
    frame: &Frame,
    context: &Context,
    os: Os,
    shell: Shell,
) -> io::Result<()> {
    let alternatives = compose_alternatives(recipe, frame, context, os, shell);
    if alternatives.is_empty() {
        return Ok(());
    }
    writeln!(w)?;
    writeln!(w, "Alternatives:")?;
    for (command, note) in alternatives {
        writeln!(w, "{command} --- {note}")?;
    }
    Ok(())
}

/// Legacy sections are the migration form of a workflow recipe.  This keeps
/// their complete ordered sequence, while applying context pruning and the
/// same visible risk markers used by focused recipes.
pub fn render_workflow<W: Write>(
    w: &mut W,
    tool: &Tool,
    section: &Section,
    frame: &Frame,
    context: &Context,
    os: Os,
    ctx: &RenderCtx,
) -> io::Result<()> {
    let mut entries: Vec<(Option<String>, String)> = Vec::new();
    for command in section.visible_commands_for(os, ctx.shell) {
        let Some(text) = command.resolve_for(os, ctx.shell) else {
            continue;
        };
        let text = substitute(text, frame, context);
        if let Some(note) = context.skip_note(&text) {
            entries.push((None, note.to_string()));
            continue;
        }
        let risk = classify_risk(&text);
        let marker = risk_marker(risk, ctx.plain);
        entries.push((
            Some(format!("{}{}", marker, text)),
            command.note.to_string(),
        ));
        if risk == Risk::Danger {
            entries.push((
                None,
                "DANGER: review the target carefully; this command can permanently lose data."
                    .into(),
            ));
        }
    }
    let title = format!("{}: {}", tool.name, section.title);
    if ctx.plain {
        writeln!(w, "# ---{}---", title)?;
        for (command, note) in entries {
            match command {
                Some(command) => writeln!(w, "{} --- {}", command, note)?,
                None => writeln!(w, "{}", note)?,
            };
        }
        return Ok(());
    }
    let lines = entries
        .into_iter()
        .map(|(command, note)| {
            command
                .map(|command| format!("  {}  --- {}", command, note))
                .unwrap_or_else(|| format!("  {}", note))
        })
        .collect::<Vec<_>>();
    render_box(w, &title, &lines, ctx)
}

/// Render one command selected from the existing catalog.  Legacy data is
/// still used for the help browser and workflows, but a normal question must
/// not make the user sift through every command in its section.
#[allow(clippy::too_many_arguments)] // mirrors the existing recipe/workflow renderer inputs
pub fn render_legacy_command<W: Write>(
    w: &mut W,
    tool: &Tool,
    section: &Section,
    command: &Command,
    frame: &Frame,
    context: &Context,
    os: Os,
    ctx: &RenderCtx,
) -> io::Result<()> {
    let Some(raw) = command.resolve_for(os, ctx.shell) else {
        return Ok(());
    };
    let rendered = substitute(raw, frame, context);
    let risk = classify_risk(&rendered);
    let title = format!("{}: {}", tool.name, section.title);
    let marker = risk_marker(risk, ctx.plain);
    if ctx.plain {
        writeln!(w, "# ---{}---", title)?;
        writeln!(w, "{}{} --- {}", marker, rendered, command.note)?;
        if risk == Risk::Danger {
            writeln!(
                w,
                "DANGER: review the target carefully; this command can permanently lose data."
            )?;
        }
        return Ok(());
    }
    let mut lines = vec![format!("  {marker}{rendered}  --- {}", command.note)];
    if risk == Risk::Danger {
        lines.push(
            "     DANGER: review the target carefully; this command can permanently lose data."
                .into(),
        );
    }
    render_box(w, &title, &lines, ctx)
}

fn classify_risk(command: &str) -> Risk {
    let lower = command.to_ascii_lowercase();
    if lower.starts_with("dd ")
        || crate::content::safety_policy()
            .dangerous_patterns
            .iter()
            // `dd ` is checked above only at the command start. Treating it
            // as an arbitrary substring would classify `pnpm add` as danger.
            .any(|pattern| pattern != &"dd " && lower.contains(pattern))
    {
        Risk::Danger
    } else if ["git restore", "git reset", "rm ", "chmod -r", "chown -r"]
        .iter()
        .any(|pattern| lower.contains(pattern))
    {
        Risk::Caution
    } else {
        Risk::Safe
    }
}

fn query_has(tokens: &[String], term: &str) -> bool {
    tokenize_for_render(term).iter().all(|word| {
        tokens
            .iter()
            .any(|token| crate::matcher::token_match_score(token, word) >= 0.70)
    })
}
fn tokenize_for_render(value: &str) -> Vec<String> {
    value
        .split(|char: char| !char.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

fn substitute(value: &str, frame: &Frame, context: &Context) -> String {
    let slot = |kind: &str| {
        frame
            .slots
            .iter()
            .find(|(slot_kind, _)| slot_kind == kind)
            .map(|(_, value)| value.as_str())
    };
    let supplied_name = slot("name");
    let project_name = supplied_name.unwrap_or("<project-name>");
    let name = supplied_name.unwrap_or("<name>");
    // A generic “named …” can name a project, folder, file, or branch.  It
    // may only fill path/branch placeholders if it passes that stricter slot
    // type's validation; this keeps `/`, `~`, and injection text visible as
    // placeholders rather than turning them into command arguments.
    let generic_path = supplied_name.filter(|name| crate::recommend::valid_slot("path", name));
    let generic_branch = supplied_name.filter(|name| crate::recommend::valid_slot("branch", name));
    let path = slot("path").or(generic_path);
    let branch = slot("branch").or(generic_branch).unwrap_or("<branch>");
    let message = slot("message").unwrap_or("<message>");
    let url = slot("url").unwrap_or("<repo-url>");
    value
        .replace("{pkg.exec}", context.package_exec())
        .replace("{pkg.run}", context.package_run())
        .replace("{pkg.install}", context.package_install())
        .replace("<project-name>", project_name)
        .replace("<name>", name)
        .replace("<folder>", path.unwrap_or("<folder>"))
        .replace("<file>", path.unwrap_or("<file>"))
        .replace("<branch>", branch)
        .replace("<message>", message)
        .replace("<repo-url>", url)
        .replace("<url>", url)
}

fn risk_marker(risk: Risk, plain: bool) -> &'static str {
    match (risk, plain) {
        (Risk::Safe, _) => "",
        (Risk::Caution, true) => "CAUTION: ",
        (Risk::Caution, false) => "[!] ",
        (Risk::Danger, true) => "DANGER: ",
        (Risk::Danger, false) => "[!!] ",
    }
}

fn render_variants_plain<W: Write>(
    w: &mut W,
    recipe: &Recipe,
    frame: &Frame,
    context: &Context,
    selected: Option<crate::model::RecipeCommand>,
    os: Os,
    ctx: &RenderCtx,
) -> io::Result<()> {
    let mut shown = 0;
    for variant in variants_in_display_order(recipe) {
        if Some(*variant) == selected {
            continue;
        }
        if let Some(command) = variant.resolve_for(os, ctx.shell) {
            if shown == tuning::MAX_VARIANTS {
                break;
            }
            if let Some(pre) = variant.pre {
                writeln!(w, "preview first: {}", substitute(pre, frame, context))?;
            }
            writeln!(
                w,
                "{}{} --- {}",
                risk_marker(variant.risk, true),
                substitute(command, frame, context),
                variant.when.unwrap_or(variant.note)
            )?;
            if variant.risk == Risk::Danger {
                writeln!(
                    w,
                    "DANGER: {}",
                    variant.warn.unwrap_or("This command can lose data.")
                )?;
            }
            shown += 1;
        }
    }
    let _ = frame;
    Ok(())
}

/// Data order is meaningful for variants of equal risk, but a later catalog
/// edit must never put a destructive alternative ahead of a safer one.
fn variants_in_display_order(recipe: &Recipe) -> Vec<&crate::model::RecipeCommand> {
    let mut variants: Vec<_> = recipe.variants.iter().collect();
    variants.sort_by_key(|variant| variant.risk);
    variants
}

/// A query can name two options from one mutually-exclusive compose group.
/// The resolver intentionally uses the later mention; make that decision
/// visible instead of silently throwing one requested option away.
fn compose_conflict_notices(recipe: &Recipe, frame: &Frame) -> Vec<String> {
    if recipe.kind != RecipeKind::Compose {
        return Vec::new();
    }

    let mut notices = Vec::new();
    let mut seen_groups = Vec::new();
    for option in recipe.options {
        let Some(group) = option.group else {
            continue;
        };
        if seen_groups.contains(&group) {
            continue;
        }
        seen_groups.push(group);

        let requested: Vec<&crate::model::RecipeOption> = recipe
            .options
            .iter()
            .filter(|candidate| candidate.group == Some(group))
            .filter(|candidate| {
                frame
                    .options
                    .iter()
                    .any(|selected| selected == candidate.id)
            })
            .collect();
        if requested.len() < 2 {
            continue;
        }
        let selected = requested
            .iter()
            .filter_map(|candidate| {
                frame
                    .options
                    .iter()
                    .position(|choice| choice == candidate.id)
                    .map(|position| (position, candidate.add))
            })
            .max_by_key(|(position, _)| *position)
            .map(|(_, add)| add)
            .expect("a requested option must have a query position");
        notices.push(format!(
            "conflicting {group} choices requested; using {selected}."
        ));
    }
    notices
}

fn render_extras_plain<W: Write>(
    w: &mut W,
    recipe: &Recipe,
    frame: &Frame,
    context: &Context,
) -> io::Result<()> {
    let mut extras = Vec::new();
    append_extras(&mut extras, recipe, frame, context);
    for extra in extras {
        writeln!(w, "{}", extra.trim())?;
    }
    Ok(())
}

fn append_extras(lines: &mut Vec<String>, recipe: &Recipe, frame: &Frame, context: &Context) {
    let hints: Vec<&str> = recipe
        .options
        .iter()
        .filter(|option| {
            option.suggest && !frame.options.iter().any(|selected| selected == option.id)
        })
        .map(|option| option.add)
        .take(tuning::MAX_OPTION_HINTS)
        .collect();
    if !hints.is_empty() {
        lines.push(format!("  You can also add:  {}", hints.join("  ")));
    }
    if !recipe.next.is_empty() {
        lines.push(format!(
            "  next:  {}",
            recipe
                .next
                .iter()
                .map(|next| substitute(next, frame, context))
                .collect::<Vec<_>>()
                .join(" && ")
        ));
    }
}

fn render_box<W: Write>(
    w: &mut W,
    title: &str,
    lines: &[String],
    ctx: &RenderCtx,
) -> io::Result<()> {
    let content_width = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
        .max(58);
    let width = (content_width + 2).min(ctx.width.max(60));
    let fill = width.saturating_sub(title.chars().count() + 9);
    if ctx.color {
        writeln!(
            w,
            "{} ___[ {}{}{} {}]{}|{}",
            term::DIM,
            term::CYAN_BOLD,
            title,
            term::RESET,
            term::DIM,
            "_".repeat(fill),
            term::RESET
        )?;
        writeln!(
            w,
            "{}|{}{}|{}",
            term::DIM,
            " ".repeat(width.saturating_sub(2)),
            term::DIM,
            term::RESET
        )?;
    } else {
        writeln!(w, " ___[ {} ]{}|", title, "_".repeat(fill))?;
        writeln!(w, "|{}|", " ".repeat(width.saturating_sub(2)))?;
    }
    for line in lines {
        let padding = " ".repeat(width.saturating_sub(2 + line.chars().count()));
        if ctx.color {
            writeln!(
                w,
                "{}|{}{}{}{}|{}",
                term::DIM,
                term::RESET,
                style_box_line(line),
                padding,
                term::DIM,
                term::RESET
            )?;
        } else {
            writeln!(w, "|{}{}|", line, padding)?;
        }
    }
    if ctx.color {
        writeln!(
            w,
            "{}|{}|{}",
            term::DIM,
            "_".repeat(width.saturating_sub(2)),
            term::RESET
        )
    } else {
        writeln!(w, "|{}|", "_".repeat(width.saturating_sub(2)))
    }
}

/// Style focused-answer lines without letting ANSI escape bytes affect layout
/// calculations. `render_box` measures raw text first, then calls this at the
/// final write boundary.
fn style_box_line(line: &str) -> String {
    if let Some((before, note)) = line.split_once("  --- ") {
        return format!(
            "{}  {}--- {}{}",
            style_command_fragment(before),
            term::DIM,
            note,
            term::RESET
        );
    }
    if let Some(command) = line.strip_prefix("  next:  ") {
        return format!(
            "  {}next:{}  {}",
            term::DIM,
            term::RESET,
            colorize_command(command)
        );
    }
    if let Some(hints) = line.strip_prefix("  You can also add:  ") {
        return format!(
            "  {}You can also add:{}  {}",
            term::DIM,
            term::RESET,
            colorize_command(hints)
        );
    }
    if line.trim() == "Alternatives" {
        return format!("  {}Alternatives{}", term::MAGENTA_BOLD, term::RESET);
    }
    if line.trim_start().starts_with("DANGER:") {
        return format!("{}{}{}", term::WARNING_RED, line, term::RESET);
    }
    if line.trim_start().starts_with("Note:") {
        return format!("{}{}{}", term::YELLOW, line, term::RESET);
    }
    if let Some(command) = line.trim_start().strip_prefix("preview first: ") {
        let indent = &line[..line.len() - line.trim_start().len()];
        return format!(
            "{}{}preview first:{} {}",
            indent,
            term::DIM,
            term::RESET,
            colorize_command(command)
        );
    }
    line.to_string()
}

fn style_command_fragment(fragment: &str) -> String {
    let indent_len = fragment.len() - fragment.trim_start().len();
    let indent = &fragment[..indent_len];
    let mut command = &fragment[indent_len..];
    let marker = if let Some(rest) = command.strip_prefix("[!!] ") {
        command = rest;
        format!("{}[!!] {}{}", term::WARNING_RED, term::RESET, "")
    } else if let Some(rest) = command.strip_prefix("[!] ") {
        command = rest;
        format!("{}[!] {}{}", term::WARNING_RED, term::RESET, "")
    } else {
        String::new()
    };
    format!("{indent}{marker}{}", colorize_command(command))
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
    fn caution_marker_is_red_in_colored_output() {
        let styled = style_command_fragment("  [!] rm <file>");
        assert!(styled.starts_with(&format!("  {}[!]", term::WARNING_RED)));
    }

    #[test]
    fn focused_answers_apply_command_placeholder_and_frame_colors() {
        let recipe = crate::content::all_recipes()
            .iter()
            .find(|recipe| recipe.id == "next.create")
            .expect("Next.js recipe");
        let frame = crate::recommend::parse("make nextjs app with tailwindcss");
        let context = Context {
            git_repo: false,
            has_remote_origin: false,
            package_manager: "npm",
            ecosystems: vec![],
        };
        let mut output = Vec::new();
        render_recipe(
            &mut output,
            recipe,
            &frame,
            &context,
            Os::Linux,
            &RenderCtx {
                color: true,
                plain: false,
                width: 100,
                shell: Shell::Cmd,
            },
            false,
        )
        .expect("render focused answer");
        let output = String::from_utf8(output).expect("UTF-8 output");
        assert!(output.contains(term::CYAN_BOLD));
        assert!(output.contains(term::CMD_COLOR));
        assert!(output.contains(term::YELLOW));
        assert!(output.contains(term::DIM));
        assert!(output.contains("--- Scaffolds a new Next.js project"));
    }

    #[test]
    fn test_render_plain_mode() {
        static CMD: [Command; 1] = [Command {
            note: "Initialize repo",
            cmd: Some("git init"),
            unix: None,
            windows: None,
            powershell: None,
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
            shell: Shell::Cmd,
        };
        let mut buf = Vec::new();
        render_section(&mut buf, &tool, &sec, Os::Linux, &ctx).expect("render");
        let output = String::from_utf8(buf).expect("utf8");
        assert_eq!(output, "# ---Git: Init---\ngit init --- Initialize repo\n");
    }
}
