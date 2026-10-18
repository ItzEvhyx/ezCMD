//! Windows command-shell selection.
//!
//! Command Prompt and PowerShell share many developer tools, but their built-in
//! commands and environment-variable syntax differ. The explicit `--shell`
//! option is always authoritative; automatic detection only supplies a useful
//! default for an interactive Windows invocation.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Shell {
    /// Command Prompt (`cmd.exe`), and the safe fallback if detection fails.
    #[default]
    Cmd,
    /// Windows PowerShell 5.1 or PowerShell 7 (`pwsh`).
    PowerShell,
}

impl Shell {
    pub fn from_flag(value: &str) -> Option<Self> {
        if value.eq_ignore_ascii_case("cmd") {
            Some(Self::Cmd)
        } else if value.eq_ignore_ascii_case("powershell") || value.eq_ignore_ascii_case("pwsh") {
            Some(Self::PowerShell)
        } else {
            None
        }
    }

    pub fn detect() -> Self {
        #[cfg(windows)]
        if let Some(shell) = windows::parent_shell() {
            return shell;
        }
        Self::Cmd
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    fn from_process_name(value: &str) -> Option<Self> {
        let basename = value.rsplit(['\\', '/']).next().unwrap_or(value);
        let lowercase = basename.to_ascii_lowercase();
        let name = lowercase.strip_suffix(".exe").unwrap_or(&lowercase);
        match name {
            "cmd" => Some(Self::Cmd),
            "powershell" | "pwsh" | "powershell_ise" => Some(Self::PowerShell),
            _ => None,
        }
    }
}

#[cfg(windows)]
mod windows {
    use super::Shell;
    use std::collections::HashMap;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    const MAX_ANCESTORS: usize = 8;

    pub(super) fn parent_shell() -> Option<Shell> {
        let processes = snapshot()?;
        let mut pid = std::process::id();
        for _ in 0..MAX_ANCESTORS {
            let (parent, _) = processes.get(&pid)?;
            let (_, name) = processes.get(parent)?;
            if let Some(shell) = Shell::from_process_name(name) {
                return Some(shell);
            }
            pid = *parent;
        }
        None
    }

    fn snapshot() -> Option<HashMap<u32, (u32, String)>> {
        let mut processes = HashMap::new();
        // SAFETY: the snapshot handle is closed before returning and the API's
        // required `dwSize` field is initialized before enumeration.
        unsafe {
            let handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if handle == INVALID_HANDLE_VALUE {
                return None;
            }
            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
            let mut found = Process32FirstW(handle, &mut entry);
            while found != 0 {
                let length = entry
                    .szExeFile
                    .iter()
                    .position(|&unit| unit == 0)
                    .unwrap_or(entry.szExeFile.len());
                processes.insert(
                    entry.th32ProcessID,
                    (
                        entry.th32ParentProcessID,
                        String::from_utf16_lossy(&entry.szExeFile[..length]),
                    ),
                );
                found = Process32NextW(handle, &mut entry);
            }
            CloseHandle(handle);
        }
        Some(processes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_shell_flags() {
        assert_eq!(Shell::from_flag("cmd"), Some(Shell::Cmd));
        assert_eq!(Shell::from_flag("PowerShell"), Some(Shell::PowerShell));
        assert_eq!(Shell::from_flag("pwsh"), Some(Shell::PowerShell));
        assert_eq!(Shell::from_flag("bash"), None);
    }

    #[test]
    fn identifies_windows_shell_process_names() {
        assert_eq!(Shell::from_process_name("cmd.exe"), Some(Shell::Cmd));
        assert_eq!(
            Shell::from_process_name(
                "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"
            ),
            Some(Shell::PowerShell)
        );
        assert_eq!(
            Shell::from_process_name("pwsh.EXE"),
            Some(Shell::PowerShell)
        );
        assert_eq!(Shell::from_process_name("bash.exe"), None);
    }
}
