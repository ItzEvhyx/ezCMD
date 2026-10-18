use crate::content::all_tools;
use crate::os::Os;
use crate::shell::Shell;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct Invocation {
    pub plain: bool,
    pub shell: Option<Shell>,
    pub action: Action,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HelpRequest {
    pub os: Option<Os>,
    pub tool: Option<&'static str>,
    pub section: Option<String>,
    pub forced: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpLevel {
    Overview,
    ToolToc,
    ToolSection,
    SectionSearch,
}

impl HelpRequest {
    pub fn level(&self) -> HelpLevel {
        match (self.tool.is_some(), self.section.is_some()) {
            (false, false) => HelpLevel::Overview,
            (true, false) => HelpLevel::ToolToc,
            (true, true) => HelpLevel::ToolSection,
            (false, true) => HelpLevel::SectionSearch,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Usage,         // no arguments at all
    Version,       // --version
    Query(String), // free-text search
    Help(HelpRequest),
    Uninstall { yes: bool }, // --uninstall
    Welcome,                 // --welcome
    Install,                 // --install
    Update,                  // update / --update
}

#[derive(Debug, PartialEq, Eq)]
pub enum CliError {
    UnknownFlag(String),
    TwoTools { first: String, second: String },
    StrayWord(String),
    SelectorAfterText(String),
    EmptyQuery,
    InvalidShell(String),
    ShellSpecifiedTwice,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::UnknownFlag(flag) => write!(f, "unknown flag: {flag}"),
            CliError::TwoTools { first, second } => {
                write!(f, "cannot select both -{first} and -{second}")
            }
            CliError::StrayWord(word) => write!(f, "unexpected word \"{word}\" after --help"),
            CliError::SelectorAfterText(selector) => {
                write!(f, "selector \"{selector}\" appears after section text")
            }
            CliError::EmptyQuery => write!(f, "query cannot be empty"),
            CliError::InvalidShell(value) if value.is_empty() => {
                write!(f, "--shell needs a value: cmd or powershell")
            }
            CliError::InvalidShell(value) => {
                write!(f, "unknown shell \"{value}\". Available: cmd, powershell")
            }
            CliError::ShellSpecifiedTwice => write!(f, "--shell can only be provided once"),
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
    let (shell, remaining) = take_shell_flag(args)?;
    let mut invocation = parse_without_shell(&remaining)?;
    invocation.shell = shell;
    Ok(invocation)
}

/// Extract `--shell <cmd|powershell>` before normal parsing. Anything after a
/// bare `--` stays literal query text, consistent with the existing grammar.
fn take_shell_flag<S: AsRef<str>>(args: &[S]) -> Result<(Option<Shell>, Vec<String>), CliError> {
    let mut shell = None;
    let mut remaining = Vec::with_capacity(args.len());
    let mut values = args.iter().map(AsRef::as_ref);
    while let Some(arg) = values.next() {
        if arg == "--" {
            remaining.push(arg.to_owned());
            remaining.extend(values.map(str::to_owned));
            break;
        }
        let value = if arg == "--shell" {
            values.next().unwrap_or("")
        } else if let Some(value) = arg.strip_prefix("--shell=") {
            value
        } else {
            remaining.push(arg.to_owned());
            continue;
        };
        if shell.is_some() {
            return Err(CliError::ShellSpecifiedTwice);
        }
        shell = Some(Shell::from_flag(value).ok_or_else(|| CliError::InvalidShell(value.into()))?);
    }
    Ok((shell, remaining))
}

fn parse_without_shell<S: AsRef<str>>(args: &[S]) -> Result<Invocation, CliError> {
    if args.is_empty() {
        return Ok(Invocation {
            plain: false,
            shell: None,
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
                shell: None,
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
            shell: None,
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
            shell: None,
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
            shell: None,
            action: Action::Install,
        });
    }

    // Check for --update before bare `--`.
    let has_update = args[..pre_dash_limit]
        .iter()
        .any(|a| a.as_ref() == "--update");
    if has_update {
        return Ok(Invocation {
            plain,
            shell: None,
            action: Action::Update,
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
                shell: None,
                action: Action::Usage,
            });
        }

        return Ok(Invocation {
            plain,
            shell: None,
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
            shell: None,
            action: Action::Usage,
        });
    }

    // `ezcmd update` is deliberately a command rather than a fuzzy query so
    // users can update an installed binary without remembering a flag.
    if query_parts.len() == 1 && query_parts[0].eq_ignore_ascii_case("update") {
        return Ok(Invocation {
            plain,
            shell: None,
            action: Action::Update,
        });
    }

    Ok(Invocation {
        plain,
        shell: None,
        action: Action::Query(query_parts.join(" ")),
    })
}

fn parse_help<S: AsRef<str>>(args: &[S], mut plain: bool) -> Result<Invocation, CliError> {
    let mut os = None;
    let mut tool: Option<&'static str> = None;
    let mut section_tokens = Vec::new();
    let mut seen_help = false;
    let mut forced = false;

    for arg in args {
        let token = arg.as_ref();
        if token == "--help" && !seen_help {
            seen_help = true;
            continue;
        }
        if !seen_help {
            if token == "--plain" {
                plain = true;
                continue;
            }
            return Err(CliError::StrayWord(token.to_string()));
        }
        if forced {
            push_section_token(&mut section_tokens, token)?;
            continue;
        }
        if token == "--plain" {
            plain = true;
            continue;
        }
        if token == "--" {
            if !section_tokens.is_empty() {
                return Err(CliError::SelectorAfterText(token.to_string()));
            }
            forced = true;
            continue;
        }
        if token.starts_with("--") {
            return Err(CliError::UnknownFlag(token.to_string()));
        }
        if !section_tokens.is_empty() {
            if token.starts_with('-') {
                return Err(CliError::SelectorAfterText(token.to_string()));
            }
            section_tokens.push(token.to_string());
            continue;
        }
        if let Some(selector) = token.strip_prefix('-') {
            if selector.is_empty() {
                return Err(CliError::StrayWord(token.to_string()));
            }
            if let Some(parsed_os) = Os::from_flag(selector) {
                os = Some(parsed_os);
                continue;
            }
            if let Some(selected_tool) = find_tool_name(selector) {
                if let Some(first) = tool {
                    if first == selected_tool {
                        push_section_token(&mut section_tokens, selector)?;
                        continue;
                    }
                    return Err(CliError::TwoTools {
                        first: first.to_string(),
                        second: selected_tool.to_string(),
                    });
                }
                tool = Some(selected_tool);
                continue;
            }
            push_section_token(&mut section_tokens, selector)?;
            continue;
        }
        return Err(CliError::StrayWord(token.to_string()));
    }

    if forced && section_tokens.is_empty() {
        return Err(CliError::EmptyQuery);
    }
    Ok(Invocation {
        plain,
        shell: None,
        action: Action::Help(HelpRequest {
            os,
            tool,
            section: (!section_tokens.is_empty()).then(|| section_tokens.join(" ")),
            forced,
        }),
    })
}

fn push_section_token(tokens: &mut Vec<String>, token: &str) -> Result<(), CliError> {
    let first = tokens.is_empty();
    let value = if first {
        token.strip_prefix('-').unwrap_or(token)
    } else {
        token
    };
    if value.is_empty() {
        return Err(CliError::EmptyQuery);
    }
    tokens.push(value.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn help(
        os: Option<Os>,
        tool: Option<&'static str>,
        section: Option<&str>,
        forced: bool,
    ) -> Action {
        Action::Help(HelpRequest {
            os,
            tool,
            section: section.map(str::to_owned),
            forced,
        })
    }

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
    fn test_update_command_and_flag() {
        let inv = parse(&["update"]).expect("parse update command");
        assert_eq!(inv.action, Action::Update);

        let inv = parse(&["--plain", "--update"]).expect("parse update flag");
        assert!(inv.plain);
        assert_eq!(inv.action, Action::Update);
    }

    #[test]
    fn test_shell_flag_is_extracted_before_normal_parsing() {
        let inv = parse(&["--shell", "powershell", "--help", "-windows", "-shell"])
            .expect("parse PowerShell help");
        assert_eq!(inv.shell, Some(Shell::PowerShell));
        assert_eq!(
            inv.action,
            help(Some(Os::Windows), Some("shell"), None, false)
        );

        let inv = parse(&["delete", "folder", "--shell=cmd"]).expect("parse CMD query");
        assert_eq!(inv.shell, Some(Shell::Cmd));
        assert_eq!(inv.action, Action::Query("delete folder".into()));

        let inv = parse(&["--", "--shell", "cmd"]).expect("literal shell query");
        assert_eq!(inv.shell, None);
        assert_eq!(inv.action, Action::Query("--shell cmd".into()));
    }

    #[test]
    fn test_shell_flag_errors_are_specific() {
        assert_eq!(
            parse(&["--shell"]).unwrap_err(),
            CliError::InvalidShell(String::new())
        );
        assert_eq!(
            parse(&["--shell", "bash"]).unwrap_err(),
            CliError::InvalidShell("bash".into())
        );
        assert_eq!(
            parse(&["--shell", "cmd", "--shell", "powershell"]).unwrap_err(),
            CliError::ShellSpecifiedTwice
        );
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
        assert_eq!(inv.action, help(None, None, None, false));
    }

    #[test]
    fn test_help_tool() {
        let inv = parse(&["--help", "-git"]).expect("parse help tool");
        assert!(!inv.plain);
        assert_eq!(inv.action, help(None, Some("git"), None, false));
    }

    #[test]
    fn test_help_tool_section() {
        let inv =
            parse(&["--help", "-git", "-initialize", "repo"]).expect("parse help tool section");
        assert!(!inv.plain);
        assert_eq!(
            inv.action,
            help(None, Some("git"), Some("initialize repo"), false)
        );
    }

    #[test]
    fn test_help_os() {
        let inv = parse(&["--help", "-windows"]).expect("parse help os");
        assert!(!inv.plain);
        assert_eq!(inv.action, help(Some(Os::Windows), None, None, false));
    }

    #[test]
    fn test_help_all_three() {
        let inv = parse(&["--help", "-windows", "-git", "-initialize", "repo"])
            .expect("parse help all three");
        assert!(!inv.plain);
        assert_eq!(
            inv.action,
            help(
                Some(Os::Windows),
                Some("git"),
                Some("initialize repo"),
                false
            )
        );
    }

    #[test]
    fn test_help_order_independent() {
        let inv = parse(&["--help", "-git", "-windows", "-initialize", "repo"])
            .expect("parse help order independent");
        assert!(!inv.plain);
        assert_eq!(
            inv.action,
            help(
                Some(Os::Windows),
                Some("git"),
                Some("initialize repo"),
                false
            )
        );
    }

    #[test]
    fn test_help_unknown_selector_starts_a_cross_tool_search() {
        let inv = parse(&["--help", "-initialize"]).expect("parse section search");
        assert_eq!(inv.action, help(None, None, Some("initialize"), false));
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
    fn test_help_stray_word_error() {
        let err = parse(&["--help", "initialize", "repo"]).unwrap_err();
        assert_eq!(err, CliError::StrayWord("initialize".into()));
    }

    #[test]
    fn test_help_quoted_section() {
        let inv = parse(&["--help", "-git", "-initialize repo"]).expect("parse quoted section");
        assert!(!inv.plain);
        assert_eq!(
            inv.action,
            help(None, Some("git"), Some("initialize repo"), false)
        );
    }

    #[test]
    fn test_help_case_insensitive_flags() {
        let inv = parse(&["--help", "-Windows", "-GIT"]).expect("parse case insensitive");
        assert_eq!(
            inv.action,
            help(Some(Os::Windows), Some("git"), None, false)
        );
    }

    #[test]
    fn test_help_trailing_plain() {
        let inv = parse(&["--help", "-git", "--plain"]).expect("parse trailing plain");
        assert!(inv.plain);
        assert_eq!(inv.action, help(None, Some("git"), None, false));
    }

    #[test]
    fn test_help_tool_alias_resolves_to_canonical_slug() {
        let inv = parse(&["--help", "-rebase"]).expect("parse tool alias");
        assert_eq!(inv.action, help(None, Some("git"), None, false));
    }

    #[test]
    fn test_help_same_tool_alias_can_select_a_section() {
        let inv = parse(&["--help", "-git", "-worktree"])
            .expect("parse same-tool alias as section selector");
        assert_eq!(inv.action, help(None, Some("git"), Some("worktree"), false));
    }

    #[test]
    fn test_help_forced_section_search() {
        let inv = parse(&["--help", "--", "find", "files"]).expect("parse forced search");
        assert_eq!(inv.action, help(None, None, Some("find files"), true));

        let inv = parse(&["--help", "-git", "--", "history"]).expect("parse forced tool search");
        assert_eq!(inv.action, help(None, Some("git"), Some("history"), true));
    }

    #[test]
    fn test_help_parser_errors() {
        let err = parse(&["--help", "-git", "-shell"]).unwrap_err();
        assert_eq!(
            err,
            CliError::TwoTools {
                first: "git".into(),
                second: "shell".into(),
            }
        );

        let err = parse(&["--help", "-git", "-initialize", "-windows"]).unwrap_err();
        assert_eq!(err, CliError::SelectorAfterText("-windows".into()));

        let err = parse(&["--help", "--unknown"]).unwrap_err();
        assert_eq!(err, CliError::UnknownFlag("--unknown".into()));
    }

    #[test]
    fn test_help_levels_are_derived_from_request_shape() {
        let overview = HelpRequest {
            os: None,
            tool: None,
            section: None,
            forced: false,
        };
        assert_eq!(overview.level(), HelpLevel::Overview);

        let tool_toc = HelpRequest {
            tool: Some("git"),
            ..overview
        };
        assert_eq!(tool_toc.level(), HelpLevel::ToolToc);

        let tool_section = HelpRequest {
            section: Some("history".into()),
            ..tool_toc
        };
        assert_eq!(tool_section.level(), HelpLevel::ToolSection);

        let search = HelpRequest {
            tool: None,
            ..tool_section
        };
        assert_eq!(search.level(), HelpLevel::SectionSearch);
    }

    #[test]
    fn test_bare_dash_with_flags_verbatim() {
        let inv = parse(&["--", "git", "commit", "-m", "hello"]).expect("parse bare dash verbatim");
        assert!(!inv.plain);
        assert_eq!(inv.action, Action::Query("git commit -m hello".into()));
    }
}
