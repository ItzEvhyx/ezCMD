use crate::content::all_tools;
use crate::os::Os;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct Invocation {
    pub plain: bool,
    pub action: Action,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Usage,         // no arguments at all
    Version,       // --version
    Query(String), // free-text search
    Help {
        os: Option<Os>,
        tool: Option<String>,
        section: Option<String>,
    },
    Uninstall {
        yes: bool,
    }, // --uninstall
    Welcome, // --welcome
    Install, // --install
}

#[derive(Debug, PartialEq, Eq)]
pub enum CliError {
    UnknownFlag(String),
    UnknownSelector(String),
    SectionWithoutTool(String),
    EmptyQuery,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::UnknownFlag(flag) => write!(f, "unknown flag: {flag}"),
            CliError::UnknownSelector(sel) => {
                let tool_list = all_tools()
                    .map(|tool| tool.slug)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(
                    f,
                    "unknown selector \"-{sel}\". Available tools: {tool_list}. Available OS: windows, linux, macos"
                )
            }
            CliError::SectionWithoutTool(sec) => {
                write!(
                    f,
                    "section query \"{sec}\" requires a tool selector (e.g. ezcmd --help -git -{sec})"
                )
            }
            CliError::EmptyQuery => write!(f, "query cannot be empty"),
        }
    }
}

impl std::error::Error for CliError {}

fn find_tool_name(candidate: &str) -> Option<&'static str> {
    all_tools()
        .find(|tool| tool.matches_flag(candidate))
        .map(|tool| tool.slug)
}

/// Parse command-line arguments (excluding program name).
pub fn parse<S: AsRef<str>>(args: &[S]) -> Result<Invocation, CliError> {
    if args.is_empty() {
        return Ok(Invocation {
            plain: false,
            action: Action::Usage,
        });
    }

    // Check for bare `--` position
    let bare_dash_pos = args.iter().position(|a| a.as_ref() == "--");

    // Scan for --plain before bare `--`
    let mut plain = false;
    let pre_dash_limit = bare_dash_pos.unwrap_or(args.len());
    for arg in &args[..pre_dash_limit] {
        if arg.as_ref() == "--plain" {
            plain = true;
        }
    }

    // Check for --version before bare `--`
    for arg in &args[..pre_dash_limit] {
        if arg.as_ref() == "--version" {
            return Ok(Invocation {
                plain,
                action: Action::Version,
            });
        }
    }

    // Check for --uninstall before bare `--`
    let has_uninstall = args[..pre_dash_limit]
        .iter()
        .any(|a| a.as_ref() == "--uninstall" || a.as_ref() == "--unintall");

    if has_uninstall {
        let yes = args[..pre_dash_limit]
            .iter()
            .any(|a| a.as_ref() == "-y" || a.as_ref() == "--yes");
        return Ok(Invocation {
            plain,
            action: Action::Uninstall { yes },
        });
    }

    // Check for --welcome before bare `--`
    let has_welcome = args[..pre_dash_limit]
        .iter()
        .any(|a| a.as_ref() == "--welcome");
    if has_welcome {
        return Ok(Invocation {
            plain,
            action: Action::Welcome,
        });
    }

    // Check for --install before bare `--`
    let has_install = args[..pre_dash_limit]
        .iter()
        .any(|a| a.as_ref() == "--install");
    if has_install {
        return Ok(Invocation {
            plain,
            action: Action::Install,
        });
    }

    // Check for --help before bare `--`
    let has_help = args[..pre_dash_limit]
        .iter()
        .any(|a| a.as_ref() == "--help");
    if has_help {
        return parse_help(args, plain);
    }

    // If bare `--` is present
    if let Some(pos) = bare_dash_pos {
        let pre_tokens = &args[..pos];
        let post_tokens = &args[pos + 1..];

        // Check pre-tokens for unknown flags
        let mut query_parts = Vec::new();
        for arg in pre_tokens {
            let s = arg.as_ref();
            if s == "--plain" {
                continue;
            }
            if s.starts_with("--") {
                return Err(CliError::UnknownFlag(s.to_string()));
            }
            query_parts.push(s.to_string());
        }

        // Post-tokens are all treated as query text verbatim
        for arg in post_tokens {
            query_parts.push(arg.as_ref().to_string());
        }

        if query_parts.is_empty() {
            return Ok(Invocation {
                plain,
                action: Action::Usage,
            });
        }

        return Ok(Invocation {
            plain,
            action: Action::Query(query_parts.join(" ")),
        });
    }

    // Normal query mode (no --help, no bare --)
    let mut query_parts = Vec::new();
    for arg in args {
        let s = arg.as_ref();
        if s == "--plain" {
            continue;
        }
        if s.starts_with("--") {
            return Err(CliError::UnknownFlag(s.to_string()));
        }
        query_parts.push(s.to_string());
    }

    if query_parts.is_empty() {
        return Ok(Invocation {
            plain,
            action: Action::Usage,
        });
    }

    Ok(Invocation {
        plain,
        action: Action::Query(query_parts.join(" ")),
    })
}

fn parse_help<S: AsRef<str>>(args: &[S], mut plain: bool) -> Result<Invocation, CliError> {
    let mut os: Option<Os> = None;
    let mut tool: Option<String> = None;
    let mut section_tokens: Vec<String> = Vec::new();

    // Collect all tokens that are not the first `--help`
    let mut passed_first_help = false;
    let mut help_args = Vec::new();
    for arg in args {
        let s = arg.as_ref();
        if s == "--help" && !passed_first_help {
            passed_first_help = true;
            continue;
        }
        help_args.push(s);
    }

    let mut i = 0;
    while i < help_args.len() {
        let s = help_args[i];

        if s == "--plain" {
            plain = true;
            i += 1;
            continue;
        }

        if s.starts_with("--") {
            return Err(CliError::UnknownFlag(s.to_string()));
        }

        // Once section tokens have begun, append non-flag tokens to section query
        if !section_tokens.is_empty() {
            if s.starts_with('-') {
                // If it's an OS selector that wasn't set yet, allow it
                let without_dash = s.trim_start_matches('-');
                if let Some(parsed_os) = Os::from_flag(without_dash) {
                    if os.is_none() {
                        os = Some(parsed_os);
                        i += 1;
                        continue;
                    }
                }
                return Err(CliError::UnknownSelector(without_dash.to_string()));
            }
            section_tokens.push(s.to_string());
            i += 1;
            continue;
        }

        // Section tokens have not started yet
        if let Some(stripped) = s.strip_prefix('-') {
            if stripped.is_empty() {
                return Err(CliError::UnknownSelector("-".to_string()));
            }

            // 1. Check if it's an OS selector
            if let Some(parsed_os) = Os::from_flag(stripped) {
                os = Some(parsed_os);
                i += 1;
                continue;
            }

            // 2. Check if it's a known tool
            if let Some(tool_name) = find_tool_name(stripped) {
                tool = Some(tool_name.to_string());
                i += 1;
                continue;
            }

            // 3. If a tool was already selected, it begins the section query
            if tool.is_some() {
                section_tokens.push(stripped.to_string());
                i += 1;
                continue;
            }

            // 4. Otherwise, unknown selector
            return Err(CliError::UnknownSelector(stripped.to_string()));
        } else {
            // Token does NOT start with '-'
            // If a tool was already selected, it begins the section query
            if tool.is_some() {
                section_tokens.push(s.to_string());
                i += 1;
                continue;
            }

            // Token without '-' and no tool selected: section query without tool is invalid
            return Err(CliError::SectionWithoutTool(s.to_string()));
        }
    }

    let section = if section_tokens.is_empty() {
        None
    } else {
        Some(section_tokens.join(" "))
    };

    Ok(Invocation {
        plain,
        action: Action::Help { os, tool, section },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_args_usage() {
        let inv = parse::<&str>(&[]).expect("parse empty");
        assert!(!inv.plain);
        assert_eq!(inv.action, Action::Usage);
    }

    #[test]
    fn test_version() {
        let inv = parse(&["--version"]).expect("parse version");
        assert!(!inv.plain);
        assert_eq!(inv.action, Action::Version);
    }

    #[test]
    fn test_plain_version() {
        let inv = parse(&["--plain", "--version"]).expect("parse plain version");
        assert!(inv.plain);
        assert_eq!(inv.action, Action::Version);
    }

    #[test]
    fn test_query() {
        let inv = parse(&["initializing", "repo"]).expect("parse query");
        assert!(!inv.plain);
        assert_eq!(inv.action, Action::Query("initializing repo".into()));
    }

    #[test]
    fn test_bare_dash_query() {
        let inv = parse(&["--", "reset", "--hard"]).expect("parse bare dash");
        assert!(!inv.plain);
        assert_eq!(inv.action, Action::Query("reset --hard".into()));
    }

    #[test]
    fn test_help_all() {
        let inv = parse(&["--help"]).expect("parse help");
        assert!(!inv.plain);
        assert_eq!(
            inv.action,
            Action::Help {
                os: None,
                tool: None,
                section: None
            }
        );
    }

    #[test]
    fn test_help_tool() {
        let inv = parse(&["--help", "-git"]).expect("parse help tool");
        assert!(!inv.plain);
        assert_eq!(
            inv.action,
            Action::Help {
                os: None,
                tool: Some("git".into()),
                section: None
            }
        );
    }

    #[test]
    fn test_help_tool_section() {
        let inv =
            parse(&["--help", "-git", "-initialize", "repo"]).expect("parse help tool section");
        assert!(!inv.plain);
        assert_eq!(
            inv.action,
            Action::Help {
                os: None,
                tool: Some("git".into()),
                section: Some("initialize repo".into())
            }
        );
    }

    #[test]
    fn test_help_os() {
        let inv = parse(&["--help", "-windows"]).expect("parse help os");
        assert!(!inv.plain);
        assert_eq!(
            inv.action,
            Action::Help {
                os: Some(Os::Windows),
                tool: None,
                section: None
            }
        );
    }

    #[test]
    fn test_help_all_three() {
        let inv = parse(&["--help", "-windows", "-git", "-initialize", "repo"])
            .expect("parse help all three");
        assert!(!inv.plain);
        assert_eq!(
            inv.action,
            Action::Help {
                os: Some(Os::Windows),
                tool: Some("git".into()),
                section: Some("initialize repo".into())
            }
        );
    }

    #[test]
    fn test_help_order_independent() {
        let inv = parse(&["--help", "-git", "-windows", "-initialize", "repo"])
            .expect("parse help order independent");
        assert!(!inv.plain);
        assert_eq!(
            inv.action,
            Action::Help {
                os: Some(Os::Windows),
                tool: Some("git".into()),
                section: Some("initialize repo".into())
            }
        );
    }

    #[test]
    fn test_help_unknown_selector() {
        let err = parse(&["--help", "-nope"]).unwrap_err();
        assert_eq!(err, CliError::UnknownSelector("nope".into()));
    }

    #[test]
    fn test_unknown_flag() {
        let err = parse(&["--bogus"]).unwrap_err();
        assert_eq!(err, CliError::UnknownFlag("--bogus".into()));
    }

    #[test]
    fn test_plain_query() {
        let inv = parse(&["--plain", "initializing", "repo"]).expect("parse plain query");
        assert!(inv.plain);
        assert_eq!(inv.action, Action::Query("initializing repo".into()));
    }

    #[test]
    fn test_section_without_tool_error() {
        let err = parse(&["--help", "-initialize", "repo"]).unwrap_err();
        assert_eq!(err, CliError::UnknownSelector("initialize".into()));

        let err2 = parse(&["--help", "initialize", "repo"]).unwrap_err();
        assert_eq!(err2, CliError::SectionWithoutTool("initialize".into()));
    }

    #[test]
    fn test_help_quoted_section() {
        let inv = parse(&["--help", "-git", "-initialize repo"]).expect("parse quoted section");
        assert!(!inv.plain);
        assert_eq!(
            inv.action,
            Action::Help {
                os: None,
                tool: Some("git".into()),
                section: Some("initialize repo".into())
            }
        );
    }

    #[test]
    fn test_help_case_insensitive_flags() {
        let inv = parse(&["--help", "-Windows", "-GIT"]).expect("parse case insensitive");
        assert_eq!(
            inv.action,
            Action::Help {
                os: Some(Os::Windows),
                tool: Some("git".into()),
                section: None
            }
        );
    }

    #[test]
    fn test_help_trailing_plain() {
        let inv = parse(&["--help", "-git", "--plain"]).expect("parse trailing plain");
        assert!(inv.plain);
        assert_eq!(
            inv.action,
            Action::Help {
                os: None,
                tool: Some("git".into()),
                section: None
            }
        );
    }

    #[test]
    fn test_bare_dash_with_flags_verbatim() {
        let inv = parse(&["--", "git", "commit", "-m", "hello"]).expect("parse bare dash verbatim");
        assert!(!inv.plain);
        assert_eq!(inv.action, Action::Query("git commit -m hello".into()));
    }
}
