use crate::os::Os;
use crate::shell::Shell;

#[derive(Debug, PartialEq, Eq)]
pub struct Category {
    pub name: &'static str,
    pub tools: &'static [Tool],
}

#[derive(Debug, PartialEq, Eq)]
pub struct Tool {
    pub slug: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub aliases: &'static [&'static str],
    pub flag_aliases: &'static [&'static str],
    pub platforms: &'static [Os],
    pub notes: &'static [&'static str],
    pub groups: &'static [Group],
}

#[derive(Debug, PartialEq, Eq)]
pub struct Group {
    pub title: &'static str,
    pub sections: &'static [Section],
}

#[derive(Debug, PartialEq, Eq)]
pub struct Section {
    pub title: &'static str,
    pub aliases: &'static [&'static str],
    pub commands: &'static [Command],
}

#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    pub note: &'static str,
    pub cmd: Option<&'static str>,  // all OSes
    pub unix: Option<&'static str>, // linux + macos
    pub windows: Option<&'static str>,
    pub powershell: Option<&'static str>,
    pub linux: Option<&'static str>,
    pub macos: Option<&'static str>,
}

impl Command {
    /// Precedence: exact OS key > unix (linux/macos only) > cmd.
    pub fn resolve(&self, os: Os) -> Option<&'static str> {
        self.resolve_for(os, Shell::Cmd)
    }

    /// Resolve a command for an OS and (on Windows) a selected shell.
    /// A PowerShell-specific entry wins over the existing Windows/CMD entry.
    pub fn resolve_for(&self, os: Os, shell: Shell) -> Option<&'static str> {
        match os {
            Os::Windows => match shell {
                Shell::Cmd => self.windows.or(self.cmd),
                Shell::PowerShell => self.powershell.or(self.windows).or(self.cmd),
            },
            Os::Linux => self.linux.or(self.unix).or(self.cmd),
            Os::Macos => self.macos.or(self.unix).or(self.cmd),
        }
    }
}

impl Section {
    pub fn visible_commands(&self, os: Os) -> impl Iterator<Item = &Command> {
        self.visible_commands_for(os, Shell::Cmd)
    }

    pub fn visible_commands_for(&self, os: Os, shell: Shell) -> impl Iterator<Item = &Command> {
        self.commands
            .iter()
            .filter(move |command| command.resolve_for(os, shell).is_some())
    }

    /// True if at least one command resolves for `os`.
    pub fn available_on(&self, os: Os) -> bool {
        self.visible_commands(os).next().is_some()
    }

    pub fn available_for(&self, os: Os, shell: Shell) -> bool {
        self.visible_commands_for(os, shell).next().is_some()
    }
}

impl Tool {
    pub fn sections(&self) -> impl Iterator<Item = &Section> {
        self.groups.iter().flat_map(|group| group.sections.iter())
    }

    pub fn available_on(&self, os: Os) -> bool {
        self.available_for(os, Shell::Cmd)
    }

    pub fn available_for(&self, os: Os, shell: Shell) -> bool {
        (self.platforms.is_empty() || self.platforms.contains(&os))
            && self
                .sections()
                .any(|section| section.available_for(os, shell))
    }

    pub fn matches_flag(&self, name: &str) -> bool {
        self.slug.eq_ignore_ascii_case(name)
            || self
                .flag_aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_cmd_fallback() {
        let cmd = Command {
            note: "test note",
            cmd: Some("generic-cmd"),
            unix: None,
            windows: None,
            powershell: None,
            linux: None,
            macos: None,
        };
        assert_eq!(cmd.resolve(Os::Windows), Some("generic-cmd"));
        assert_eq!(cmd.resolve(Os::Linux), Some("generic-cmd"));
        assert_eq!(cmd.resolve(Os::Macos), Some("generic-cmd"));
    }

    #[test]
    fn test_resolve_unix_precedence() {
        let cmd = Command {
            note: "test note",
            cmd: Some("generic-cmd"),
            unix: Some("unix-cmd"),
            windows: None,
            powershell: None,
            linux: None,
            macos: None,
        };
        assert_eq!(cmd.resolve(Os::Windows), Some("generic-cmd"));
        assert_eq!(cmd.resolve(Os::Linux), Some("unix-cmd"));
        assert_eq!(cmd.resolve(Os::Macos), Some("unix-cmd"));
    }

    #[test]
    fn test_resolve_exact_os_precedence() {
        let cmd = Command {
            note: "test note",
            cmd: Some("generic-cmd"),
            unix: Some("unix-cmd"),
            windows: Some("win-cmd"),
            powershell: None,
            linux: Some("linux-cmd"),
            macos: Some("macos-cmd"),
        };
        assert_eq!(cmd.resolve(Os::Windows), Some("win-cmd"));
        assert_eq!(cmd.resolve(Os::Linux), Some("linux-cmd"));
        assert_eq!(cmd.resolve(Os::Macos), Some("macos-cmd"));
    }

    #[test]
    fn test_resolve_none_when_no_match() {
        let cmd = Command {
            note: "test note",
            cmd: None,
            unix: None,
            windows: Some("win-only"),
            powershell: None,
            linux: None,
            macos: None,
        };
        assert_eq!(cmd.resolve(Os::Windows), Some("win-only"));
        assert_eq!(cmd.resolve(Os::Linux), None);
        assert_eq!(cmd.resolve(Os::Macos), None);
    }

    #[test]
    fn test_section_available_on() {
        static CMD_WIN: [Command; 1] = [Command {
            note: "win",
            cmd: None,
            unix: None,
            windows: Some("win-only"),
            powershell: None,
            linux: None,
            macos: None,
        }];
        let sec = Section {
            title: "Test Section",
            aliases: &["test"],
            commands: &CMD_WIN,
        };
        assert!(sec.available_on(Os::Windows));
        assert!(!sec.available_on(Os::Linux));
        assert!(!sec.available_on(Os::Macos));
    }

    #[test]
    fn test_tool_availability_and_flag_aliases() {
        static COMMANDS: [Command; 1] = [Command {
            note: "mac only",
            cmd: None,
            unix: None,
            windows: None,
            powershell: None,
            linux: None,
            macos: Some("mac-tool"),
        }];
        static SECTIONS: [Section; 1] = [Section {
            title: "Mac task",
            aliases: &["mac task", "apple task", "darwin task"],
            commands: &COMMANDS,
        }];
        static GROUPS: [Group; 1] = [Group {
            title: "Everything",
            sections: &SECTIONS,
        }];
        let tool = Tool {
            slug: "pods",
            name: "CocoaPods",
            category: "Mobile Development",
            aliases: &["cocoapods"],
            flag_aliases: &["cocoapods"],
            platforms: &[Os::Macos],
            notes: &[],
            groups: &GROUPS,
        };

        assert!(tool.matches_flag("pods"));
        assert!(tool.matches_flag("COCOAPODS"));
        assert!(!tool.matches_flag("pod"));
        assert!(tool.available_on(Os::Macos));
        assert!(!tool.available_on(Os::Linux));
    }

    #[test]
    fn powershell_command_wins_on_windows() {
        let command = Command {
            note: "shell-specific",
            cmd: Some("generic"),
            unix: None,
            windows: Some("cmd-command"),
            powershell: Some("powershell-command"),
            linux: None,
            macos: None,
        };

        assert_eq!(
            command.resolve_for(Os::Windows, Shell::Cmd),
            Some("cmd-command")
        );
        assert_eq!(
            command.resolve_for(Os::Windows, Shell::PowerShell),
            Some("powershell-command")
        );
    }

    #[test]
    fn powershell_only_section_is_not_exposed_to_cmd() {
        static COMMANDS: [Command; 1] = [Command {
            note: "PowerShell only",
            cmd: None,
            unix: None,
            windows: None,
            powershell: Some("Get-ChildItem"),
            linux: None,
            macos: None,
        }];
        let section = Section {
            title: "PowerShell",
            aliases: &[],
            commands: &COMMANDS,
        };

        assert!(!section.available_for(Os::Windows, Shell::Cmd));
        assert!(section.available_for(Os::Windows, Shell::PowerShell));
    }
}
