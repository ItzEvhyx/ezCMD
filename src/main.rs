use ezcmd::cli::{self, Action};
use ezcmd::content::all_tools;
use ezcmd::matcher::{self, SearchResult};
use ezcmd::os::Os;
use ezcmd::prompt;
use ezcmd::render::{self, RenderCtx};
use ezcmd::term;
use std::io::{self, IsTerminal, Write};

fn print_usage_header<W: Write>(w: &mut W, ctx: &RenderCtx) -> io::Result<()> {
    writeln!(w, "ezcmd {}", env!("CARGO_PKG_VERSION"))?;
    writeln!(w, "Find the right command, in order.\n")?;

    if ctx.color {
        writeln!(w, "{}USAGE:{}", term::CYAN_BOLD, term::RESET)?;
    } else {
        writeln!(w, "USAGE:")?;
    }

    writeln!(
        w,
        "  ezcmd <query>                          Find a section or command"
    )?;
    writeln!(
        w,
        "  ezcmd --help                           Show every section (this OS)"
    )?;
    writeln!(
        w,
        "  ezcmd --help -<tool>                   Show one tool's sections"
    )?;
    writeln!(
        w,
        "  ezcmd --help -<tool> -<section>        Show one section"
    )?;
    writeln!(
        w,
        "  ezcmd --help -<os>                     Show everything for another OS"
    )?;
    writeln!(
        w,
        "  ezcmd --install                        Install ezcmd to PATH"
    )?;
    writeln!(
        w,
        "  ezcmd --uninstall                      Uninstall ezcmd from machine"
    )?;
    writeln!(
        w,
        "  ezcmd --welcome                        Show welcome banner & quickstart"
    )?;
    Ok(())
}

fn run<W: Write>(args: &[String], out: &mut W) -> io::Result<i32> {
    let invocation = match cli::parse(args) {
        Ok(inv) => inv,
        Err(err) => {
            eprintln!("error: {err}");
            eprintln!("Try: ezcmd --help");
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
            writeln!(out, "Run 'ezcmd --help' to see all commands.")?;
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
            let search_res = matcher::search(all_tools(), os, &query, None);

            match search_res {
                SearchResult::Single(hit) => {
                    render::render_section(out, hit.section, os, &render_ctx)?;
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
                                render::render_section(out, hits[idx].section, os, &render_ctx)?;
                                Ok(0)
                            }
                            None => Ok(1), // User aborted or invalid input
                        }
                    } else {
                        // Non-interactive or piped: output best match and exit 0
                        render::render_section(out, hits[0].section, os, &render_ctx)?;
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
                            writeln!(out, "  - {}", h.section.title)?;
                        }
                    }
                    Ok(1)
                }
            }
        }
        Action::Help {
            os: opt_os,
            tool: opt_tool,
            section: opt_section,
        } => {
            let target_os = opt_os.unwrap_or_else(Os::current);

            // If a specific section query is requested: `--help -<tool> -<section>`
            if let Some(sec_query) = opt_section {
                let search_res =
                    matcher::search(all_tools(), target_os, &sec_query, opt_tool.as_deref());

                match search_res {
                    SearchResult::Single(hit) => {
                        render::render_section(out, hit.section, target_os, &render_ctx)?;
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
                                        hits[idx].section,
                                        target_os,
                                        &render_ctx,
                                    )?;
                                    Ok(0)
                                }
                                None => Ok(1),
                            }
                        } else {
                            render::render_section(out, hits[0].section, target_os, &render_ctx)?;
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
                                writeln!(out, "  - {}", h.section.title)?;
                            }
                        }
                        Ok(1)
                    }
                }
            } else {
                // Whole help output or single tool's help output
                print_usage_header(out, &render_ctx)?;
                writeln!(out)?;

                let target_tools: Vec<_> = if let Some(ref tool_name) = opt_tool {
                    all_tools()
                        .filter(|tool| tool.slug.eq_ignore_ascii_case(tool_name))
                        .collect()
                } else {
                    all_tools().collect()
                };

                let mut first_tool = true;
                for tool in target_tools {
                    let available_sections: Vec<_> = tool
                        .sections()
                        .filter(|s| s.available_on(target_os))
                        .collect();

                    if available_sections.is_empty() {
                        continue;
                    }

                    if !first_tool {
                        writeln!(out)?;
                    }

                    if opt_tool.is_none() {
                        if render_ctx.color {
                            writeln!(
                                out,
                                "{}TOOL: {}{}\n",
                                term::CYAN_BOLD,
                                tool.slug,
                                term::RESET
                            )?;
                        } else {
                            writeln!(out, "TOOL: {}\n", tool.slug)?;
                        }
                    }

                    render::render_sections(out, &available_sections, target_os, &render_ctx)?;
                    first_tool = false;
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
