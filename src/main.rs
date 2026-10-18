use ezcmd::cli::{self, Action};
use ezcmd::content::{all_tools, CATEGORIES};
use ezcmd::matcher::{self, SearchResult};
use ezcmd::os::Os;
use ezcmd::prompt;
use ezcmd::render::{self, RenderCtx};
use ezcmd::term;
use std::io::{self, IsTerminal, Write};

fn print_usage_header<W: Write>(w: &mut W, ctx: &RenderCtx) -> io::Result<()> {
    render::usage_guide(w, ctx)
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
    let is_color = term::color_enabled(is_plain);
    let width = term::width();
    let render_ctx = RenderCtx {
        color: is_color,
        plain: is_plain,
        width,
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
        Action::Query(query) => {
            let os = Os::current();
            let search_res = matcher::search(
                all_tools(),
                os,
                &query,
                matcher::SearchOpts {
                    tool_tokens: true,
                    ..Default::default()
                },
            );

            match search_res {
                SearchResult::ToolOnly(tool) => {
                    render::toc_tool(out, tool, os, &render_ctx)?;
                    Ok(0)
                }
                SearchResult::Single(hit) => {
                    render::render_section(out, hit.tool, hit.section, os, &render_ctx)?;
                    Ok(0)
                }
                SearchResult::Ambiguous(hits) => {
                    let is_interactive = io::stdin().is_terminal()
                        && io::stdout().is_terminal()
                        && !render_ctx.plain;

                    if is_interactive {
                        writeln!(out, "Multiple matches for \"{}\":", query)?;
                        let mut stdin = io::stdin().lock();
                        match prompt::choose(&mut stdin, out, &hits)? {
                            Some(idx) => {
                                writeln!(out)?;
                                render::render_section(
                                    out,
                                    hits[idx].tool,
                                    hits[idx].section,
                                    os,
                                    &render_ctx,
                                )?;
                                Ok(0)
                            }
                            None => Ok(1), // User aborted or invalid input
                        }
                    } else {
                        // Non-interactive or piped: output best match and exit 0
                        render::render_section(
                            out,
                            hits[0].tool,
                            hits[0].section,
                            os,
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
                            query,
                            term::RESET
                        )?;
                    } else {
                        writeln!(out, "No match for \"{}\"", query)?;
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
        }
        Action::Help(request) => {
            let target_os = request.os.unwrap_or_else(Os::current);
            if let Some(tool_name) = request.tool {
                if let Some(tool) = all_tools().find(|tool| tool.slug == tool_name) {
                    if !tool.available_on(target_os) {
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
                            writeln!(out, "Multiple matches for \"{}\":", sec_query)?;
                            let mut stdin = io::stdin().lock();
                            match prompt::choose(&mut stdin, out, &hits)? {
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
