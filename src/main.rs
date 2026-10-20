use ezcmd::cli::{self, Action};
use ezcmd::content::{all_tools, CATEGORIES};
use ezcmd::context::Context;
use ezcmd::history::History;
use ezcmd::matcher::{self, SearchResult};
use ezcmd::os::Os;
use ezcmd::prompt;
use ezcmd::recommend::{self, Candidate, Decision, ScoredCandidate};
use ezcmd::render::{self, RenderCtx};
use ezcmd::shell::Shell;
use ezcmd::term;
use std::io::{self, IsTerminal, Write};

fn print_usage_header<W: Write>(w: &mut W, ctx: &RenderCtx) -> io::Result<()> {
    render::usage_guide(w, ctx)
}

fn render_candidate<W: Write>(
    out: &mut W,
    candidate: ScoredCandidate,
    frame: &ezcmd::recommend::Frame,
    context: &Context,
    os: Os,
    render_ctx: &RenderCtx,
    brief: bool,
) -> io::Result<i32> {
    match candidate.candidate {
        Candidate::Recipe(recipe) => {
            render::render_recipe(out, recipe, frame, context, os, render_ctx, brief)?
        }
        Candidate::LegacyCommand {
            tool,
            section,
            command,
        } => render::render_legacy_command(
            out, tool, section, command, frame, context, os, render_ctx,
        )?,
        Candidate::Workflow { tool, section } => {
            render::render_workflow(out, tool, section, frame, context, os, render_ctx)?
        }
    }
    Ok(0)
}

fn selected_history_id(candidate: &ScoredCandidate) -> String {
    candidate.candidate.id()
}

fn run<W: Write>(args: &[String], out: &mut W) -> io::Result<i32> {
    let invocation = match cli::parse(args) {
        Ok(inv) => inv,
        Err(err) => {
            eprintln!("error: {err}");
            match err {
                cli::CliError::StrayWord(word) => eprintln!("Try: ezcmd --help -{word}"),
                cli::CliError::SelectorAfterText(_) => {
                    eprintln!("Put -<os> and -<tool> before the section text")
                }
                cli::CliError::TwoTools { .. } => eprintln!("Use one tool at a time"),
                _ => eprintln!("Try: ezcmd --help"),
            }
            return Ok(2);
        }
    };

    let is_plain = invocation.plain;
    let shell = invocation.shell.unwrap_or_else(Shell::detect);
    let is_color = term::color_enabled(is_plain);
    let width = term::width();
    let render_ctx = RenderCtx {
        color: is_color,
        plain: is_plain,
        width,
        shell,
    };

    match invocation.action {
        Action::Usage => {
            print_usage_header(out, &render_ctx)?;
            writeln!(out)?;
            writeln!(out, "Run 'ezcmd --help' to browse all tools.")?;
            Ok(0)
        }
        Action::Install => ezcmd::welcome::run_install(out, render_ctx.color),
        Action::Welcome => {
            ezcmd::welcome::print_welcome(out, render_ctx.color)?;
            Ok(0)
        }
        Action::Uninstall { yes } => ezcmd::uninstall::run_uninstall(out, yes),
        Action::Version => {
            writeln!(out, "ezcmd {}", env!("CARGO_PKG_VERSION"))?;
            Ok(0)
        }
        Action::Update => ezcmd::update::run(out),
        Action::Forget => {
            if History::forget() {
                writeln!(out, "ezcmd history cleared.")?;
                Ok(0)
            } else {
                writeln!(out, "ezcmd history is disabled or unavailable.")?;
                Ok(1)
            }
        }
        Action::Query(query) => {
            let os = Os::current();
            let words = matcher::tokenize(&query);
            if words.len() == 1 {
                if let Some(tool) = all_tools().find(|tool| tool.matches_flag(&words[0])) {
                    render::toc_tool(out, tool, os, &render_ctx)?;
                    return Ok(0);
                }
            }
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let context = Context::detect(&cwd);
            let mut history = History::load();
            let (frame, decision) =
                recommend::decide_with_history(&query, os, shell, &context, &history);
            if invocation.explain {
                write!(out, "{}", recommend::explain(&frame, &decision))?;
            }
            match decision {
                Decision::Clear(candidate) => {
                    let history_id = selected_history_id(&candidate);
                    let result = render_candidate(
                        out,
                        candidate,
                        &frame,
                        &context,
                        os,
                        &render_ctx,
                        invocation.brief,
                    );
                    if result.as_ref().is_ok_and(|code| *code == 0) {
                        history.record(&history_id);
                    }
                    result
                }
                Decision::Ambiguous(candidates) => {
                    write_multiple_matches_heading(out, &query, render_ctx.color)?;
                    for (index, candidate) in candidates.iter().enumerate() {
                        write_recommendation_choice(out, index + 1, candidate, render_ctx.color)?;
                    }
                    let interactive = io::stdin().is_terminal()
                        && io::stdout().is_terminal()
                        && !render_ctx.plain;
                    let selected = if interactive {
                        if render_ctx.color {
                            write!(
                                out,
                                "{}Choose{} {}1-{}{}: ",
                                term::GREEN_BOLD,
                                term::RESET,
                                term::YELLOW,
                                candidates.len(),
                                term::RESET
                            )?;
                        } else {
                            write!(out, "Choose 1-{}: ", candidates.len())?;
                        }
                        out.flush()?;
                        let mut answer = String::new();
                        let _ = io::stdin().read_line(&mut answer);
                        answer
                            .trim()
                            .parse::<usize>()
                            .ok()
                            .and_then(|index| candidates.get(index.saturating_sub(1)))
                            .cloned()
                            .unwrap_or_else(|| candidates[0].clone())
                    } else {
                        candidates
                            .iter()
                            .find(|candidate| !candidate.candidate.danger_primary())
                            .cloned()
                            .unwrap_or_else(|| candidates[0].clone())
                    };
                    let history_id = selected_history_id(&selected);
                    let result = render_candidate(
                        out,
                        selected,
                        &frame,
                        &context,
                        os,
                        &render_ctx,
                        invocation.brief,
                    );
                    if result.as_ref().is_ok_and(|code| *code == 0) {
                        history.record(&history_id);
                    }
                    result
                }
                Decision::NoMatch(closest) => {
                    writeln!(out, "No match for \"{}\".", query)?;
                    if !closest.is_empty() {
                        writeln!(out, "Closest sections:")?;
                        for (index, candidate) in closest.iter().enumerate() {
                            writeln!(out, "  {}. {}", index + 1, candidate.candidate.title())?;
                        }
                    }
                    writeln!(out, "Try: ezcmd --help")?;
                    Ok(1)
                }
                Decision::Blocked(reason) => {
                    if render_ctx.color {
                        writeln!(
                            out,
                            "{}[!!] WARNING: Request blocked:{} {reason}",
                            term::WARNING_RED,
                            term::RESET
                        )?;
                        writeln!(
                            out,
                            "{}ezcmd never fills in system or home paths for deleting commands.{}",
                            term::WARNING_RED,
                            term::RESET
                        )?;
                        writeln!(
                            out,
                            "{}Use a precise, non-system project path if you meant a specific file or folder.{}",
                            term::WARNING_RED,
                            term::RESET
                        )?;
                    } else {
                        writeln!(out, "Request blocked: {reason}")?;
                        writeln!(
                            out,
                            "ezcmd never fills in system or home paths for deleting commands."
                        )?;
                        writeln!(
                            out,
                            "Use a precise, non-system project path if you meant a specific file or folder."
                        )?;
                    }
                    Ok(1)
                }
            }
        }
        Action::Help(request) => {
            let target_os = request.os.unwrap_or_else(Os::current);
            if let Some(tool_name) = request.tool {
                if let Some(tool) = all_tools().find(|tool| tool.slug == tool_name) {
                    if !tool.available_for(target_os, shell) {
                        let supported = if tool.platforms.is_empty() {
                            Os::ALL
                                .iter()
                                .map(|os| os.display())
                                .collect::<Vec<_>>()
                                .join(", ")
                        } else {
                            tool.platforms
                                .iter()
                                .map(|os| os.display())
                                .collect::<Vec<_>>()
                                .join(", ")
                        };
                        eprintln!(
                            "{} is not available on {}. It works on: {}.",
                            tool.name,
                            target_os.display(),
                            supported
                        );
                        return Ok(1);
                    }
                }
            }

            // If a specific section query is requested: `--help -<tool> -<section>`
            if let Some(sec_query) = request.section {
                let search_res = matcher::search(
                    all_tools(),
                    target_os,
                    &sec_query,
                    matcher::SearchOpts {
                        only_tool: request.tool,
                        shell,
                        tool_tokens: false,
                    },
                );

                match search_res {
                    SearchResult::ToolOnly(tool) => {
                        render::toc_tool(out, tool, target_os, &render_ctx)?;
                        Ok(0)
                    }
                    SearchResult::Single(hit) => {
                        render::render_section(out, hit.tool, hit.section, target_os, &render_ctx)?;
                        Ok(0)
                    }
                    SearchResult::Ambiguous(hits) => {
                        let is_interactive = io::stdin().is_terminal()
                            && io::stdout().is_terminal()
                            && !render_ctx.plain;

                        if is_interactive {
                            write_multiple_matches_heading(out, &sec_query, render_ctx.color)?;
                            let mut stdin = io::stdin().lock();
                            match prompt::choose(&mut stdin, out, &hits, render_ctx.color)? {
                                Some(idx) => {
                                    writeln!(out)?;
                                    render::render_section(
                                        out,
                                        hits[idx].tool,
                                        hits[idx].section,
                                        target_os,
                                        &render_ctx,
                                    )?;
                                    Ok(0)
                                }
                                None => Ok(1),
                            }
                        } else {
                            render::render_section(
                                out,
                                hits[0].tool,
                                hits[0].section,
                                target_os,
                                &render_ctx,
                            )?;
                            Ok(0)
                        }
                    }
                    SearchResult::NoMatch(closest) => {
                        if render_ctx.color {
                            writeln!(
                                out,
                                "{}No match for \"{}\"{}",
                                term::RED_BOLD,
                                sec_query,
                                term::RESET
                            )?;
                        } else {
                            writeln!(out, "No match for \"{}\"", sec_query)?;
                        }
                        if !closest.is_empty() {
                            writeln!(out, "\nClosest matches:")?;
                            for h in closest {
                                writeln!(out, "  - {}: {}", h.tool.name, h.section.title)?;
                            }
                        }
                        Ok(1)
                    }
                }
            } else {
                if let Some(tool_name) = request.tool {
                    if let Some(tool) =
                        all_tools().find(|tool| tool.slug.eq_ignore_ascii_case(tool_name))
                    {
                        render::toc_tool(out, tool, target_os, &render_ctx)?;
                    }
                } else {
                    render::usage_guide(out, &render_ctx)?;
                    writeln!(out)?;
                    render::toc_overview(out, CATEGORIES, target_os, &render_ctx)?;
                }
                Ok(0)
            }
        }
    }
}

fn write_multiple_matches_heading<W: Write>(
    out: &mut W,
    query: &str,
    color: bool,
) -> io::Result<()> {
    if color {
        writeln!(
            out,
            "{}Multiple matches{} for {}\"{}\"{}:",
            term::CYAN_BOLD,
            term::RESET,
            term::YELLOW,
            query,
            term::RESET
        )
    } else {
        writeln!(out, "Multiple matches for \"{query}\":")
    }
}

/// Make an ambiguity menu scannable at a glance: every candidate row receives
/// a different color. Plain output deliberately keeps the stable textual form
/// for scripts and terminals without ANSI support.
fn write_recommendation_choice<W: Write>(
    out: &mut W,
    index: usize,
    candidate: &ScoredCandidate,
    color: bool,
) -> io::Result<()> {
    let title = candidate.candidate.title();
    if !color {
        return writeln!(out, "  [{index}] {title}");
    }
    writeln!(
        out,
        "  {}[{}] {}{}",
        term::choice_color(index - 1),
        index,
        title,
        term::RESET
    )
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    let exit_code = match run(&args, &mut out) {
        Ok(code) => code,
        Err(err) => {
            if err.kind() == io::ErrorKind::BrokenPipe {
                std::process::exit(0);
            }
            eprintln!("ezcmd error: {err}");
            1
        }
    };

    if let Err(err) = out.flush() {
        if err.kind() == io::ErrorKind::BrokenPipe {
            std::process::exit(0);
        }
        eprintln!("error flushing output: {err}");
        std::process::exit(1);
    }

    std::process::exit(exit_code);
}
