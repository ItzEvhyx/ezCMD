use crate::term;
use std::io::{self, Write};
use std::path::PathBuf;

pub const ASCII_LINES: [&str; 8] = [
    r"                      /$$$$$$  /$$      /$$ /$$$$$$$ ",
    r"                     /$$__  $$| $$$    /$$$| $$__  $$",
    r"  /$$$$$$  /$$$$$$$$| $$  \__/| $$$$  /$$$$| $$  \ $$",
    r" /$$__  $$|____ /$$/| $$      | $$ $$/$$ $$| $$  | $$",
    r"| $$$$$$$$   /$$$$/ | $$      | $$  $$$| $$| $$  | $$",
    r"| $$_____/  /$$__/  | $$    $$| $$\  $ | $$| $$  | $$",
    r"|  $$$$$$$ /$$$$$$$$|  $$$$$$/| $$ \/  | $$| $$$$$$$/",
    r" \_______/|________/ \______/ |__/     |__/|_______/ ",
];

const GRADIENT: [&str; 8] = [
    "\x1b[1;38;2;165;220;253m", // Ice blue
    "\x1b[1;38;2;145;205;251m",
    "\x1b[1;38;2;125;188;248m",
    "\x1b[1;38;2;100;170;244m",
    "\x1b[1;38;2;75;150;238m",
    "\x1b[1;38;2;50;130;230m",
    "\x1b[1;38;2;30;110;218m",
    "\x1b[1;38;2;15;90;200m", // Deep cobalt blue
];

pub fn get_config_dir() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("ezcmd"));
    }
    #[cfg(windows)]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return Some(PathBuf::from(appdata).join("ezcmd"));
        }
        if let Some(userprofile) = std::env::var_os("USERPROFILE") {
            return Some(PathBuf::from(userprofile).join(".ezcmd"));
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            return Some(PathBuf::from(home).join(".config").join("ezcmd"));
        }
    }
    None
}

pub fn get_marker_path() -> Option<PathBuf> {
    get_config_dir().map(|d| d.join(".installed"))
}

pub fn check_first_run() -> bool {
    if std::env::var("EZCMD_NO_WELCOME").is_ok() || std::env::var("CI").is_ok() {
        return false;
    }
    if let Some(marker) = get_marker_path() {
        if marker.exists() {
            return false;
        }
        if let Some(parent) = marker.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&marker, "installed");
        return true;
    }
    false
}

pub fn run_install<W: Write>(w: &mut W, color: bool) -> io::Result<i32> {
    #[cfg(not(windows))]
    let bin_dir = std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join(".local/bin"))
        .unwrap_or_else(|| PathBuf::from("/usr/local/bin"));

    #[cfg(windows)]
    let bin_dir = std::env::var_os("USERPROFILE")
        .map(|u| PathBuf::from(u).join(".local/bin"))
        .unwrap_or_else(|| PathBuf::from("C:\\bin"));

    let _ = std::fs::create_dir_all(&bin_dir);

    #[cfg(windows)]
    let dest_exe = bin_dir.join("ezcmd.exe");
    #[cfg(not(windows))]
    let dest_exe = bin_dir.join("ezcmd");

    if let Ok(current_exe) = std::env::current_exe() {
        if current_exe != dest_exe {
            let _ = std::fs::remove_file(&dest_exe);
            let _ = std::fs::copy(&current_exe, &dest_exe);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = std::fs::metadata(&dest_exe) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    let _ = std::fs::set_permissions(&dest_exe, perms);
                }
            }
        }
    }

    if let Some(marker) = get_marker_path() {
        if let Some(parent) = marker.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&marker, "installed");
    }

    if color {
        writeln!(
            w,
            "{}✓ Successfully installed ezcmd to {}!{}",
            term::CYAN_BOLD,
            dest_exe.display(),
            term::RESET
        )?;
    } else {
        writeln!(
            w,
            "✓ Successfully installed ezcmd to {}!",
            dest_exe.display()
        )?;
    }
    writeln!(w)?;

    print_welcome(w, color)?;
    Ok(0)
}

pub fn print_welcome<W: Write>(w: &mut W, color: bool) -> io::Result<()> {
    if color {
        const C_CYAN: &str = "\x1b[1;36m";
        const C_YELLOW: &str = "\x1b[1;33m";
        const C_MAGENTA: &str = "\x1b[1;38;2;215;135;255m";

        for (i, line) in ASCII_LINES.iter().enumerate() {
            writeln!(w, "{}{}{}", GRADIENT[i], line, term::RESET)?;
        }
        writeln!(w)?;
        writeln!(
            w,
            "  {}Welcome to ezCMD!{} {}Type what you want to do. Get the commands, in order.{}",
            term::BOLD,
            term::RESET,
            term::CMD_COLOR,
            term::RESET
        )?;
        writeln!(
            w,
            "  {}Fast, offline terminal cheatsheet with zero telemetry — commands in < 20ms.{}\n",
            term::DIM,
            term::RESET
        )?;

        writeln!(
            w,
            "  {}+-- [ UPDATE ] ----------------------------------------------------------+{}",
            C_MAGENTA,
            term::RESET
        )?;
        writeln!(
            w,
            "  {}|{}  {}Focused answers:{} specific queries return only what you need.          {}|{}",
            C_MAGENTA,
            term::RESET,
            term::CMD_COLOR,
            term::RESET,
            C_MAGENTA,
            term::RESET
        )?;
        writeln!(
            w,
            "  {}|{}  Broad searches show a primary command with clear alternatives.        {}|{}",
            C_MAGENTA,
            term::RESET,
            C_MAGENTA,
            term::RESET
        )?;
        writeln!(
            w,
            "  {}|{}  Workflows stay complete; warnings and choices are color-coded.        {}|{}",
            C_MAGENTA,
            term::RESET,
            C_MAGENTA,
            term::RESET
        )?;
        writeln!(
            w,
            "  {}+------------------------------------------------------------------------+{}\n",
            C_MAGENTA,
            term::RESET
        )?;

        writeln!(
            w,
            "  {}+-- [ 🚀 QUICK START / HOW TO USE ] ------------------------------------+{}",
            C_CYAN,
            term::RESET
        )?;
        writeln!(
            w,
            "  {}|{}  {}ezcmd <query>{}            {}---{} Search for any command or workflow       {}|{}",
            C_CYAN, term::RESET, term::CMD_COLOR, term::RESET, term::DIM, term::RESET, C_CYAN, term::RESET
        )?;
        writeln!(
            w,
            "  {}|{}  {}ezcmd --help{}             {}---{} Browse categories and tools for your OS   {}|{}",
            C_CYAN, term::RESET, term::CMD_COLOR, term::RESET, term::DIM, term::RESET, C_CYAN, term::RESET
        )?;
        writeln!(
            w,
            "  {}|{}  {}ezcmd --help -git{}        {}---{} View Git groups and section titles        {}|{}",
            C_CYAN, term::RESET, term::CMD_COLOR, term::RESET, term::DIM, term::RESET, C_CYAN, term::RESET
        )?;
        writeln!(
            w,
            "  {}|{}  {}ezcmd --help -shell{}      {}---{} View Shell groups and section titles      {}|{}",
            C_CYAN, term::RESET, term::CMD_COLOR, term::RESET, term::DIM, term::RESET, C_CYAN, term::RESET
        )?;
        writeln!(
            w,
            "  {}|{}  {}ezcmd --uninstall{}        {}---{} Uninstall ezcmd from your system         {}|{}",
            C_CYAN, term::RESET, term::CMD_COLOR, term::RESET, term::DIM, term::RESET, C_CYAN, term::RESET
        )?;
        writeln!(
            w,
            "  {}|{}  {}ezcmd update{}             {}---{} Install the latest published release       {}|{}",
            C_CYAN, term::RESET, term::CMD_COLOR, term::RESET, term::DIM, term::RESET, C_CYAN, term::RESET
        )?;
        writeln!(
            w,
            "  {}+------------------------------------------------------------------------+{}\n",
            C_CYAN,
            term::RESET
        )?;

        writeln!(w, "  {}⚡ WORKFLOWS TO TRY:{}", C_YELLOW, term::RESET)?;
        writeln!(
            w,
            "    {}ezcmd{} initializing repo       {}# Starting a new Git repo{}",
            term::CMD_COLOR,
            term::RESET,
            term::DIM,
            term::RESET
        )?;
        writeln!(
            w,
            "    {}ezcmd{} undo last commit         {}# Safely revert commits{}",
            term::CMD_COLOR,
            term::RESET,
            term::DIM,
            term::RESET
        )?;
        writeln!(
            w,
            "    {}ezcmd{} how to branch            {}# Branching & merging workflows{}",
            term::CMD_COLOR,
            term::RESET,
            term::DIM,
            term::RESET
        )?;
        writeln!(
            w,
            "    {}ezcmd{} delete files             {}# File & directory cleanup{}",
            term::CMD_COLOR,
            term::RESET,
            term::DIM,
            term::RESET
        )?;
        writeln!(w)?;

        writeln!(w, "  {}✍️ AUTHOR:{}", C_MAGENTA, term::RESET)?;
        writeln!(w, "    Created by {}ItzEvhyx{}", term::BOLD, term::RESET)?;
        writeln!(
            w,
            "    GitHub: {}https://github.com/ItzEvhyx{}",
            term::UNDERLINE,
            term::RESET
        )?;
        writeln!(
            w,
            "    Repo:   {}https://github.com/ItzEvhyx/ezCMD{}\n",
            term::UNDERLINE,
            term::RESET
        )?;
    } else {
        for line in ASCII_LINES {
            writeln!(w, "{}", line)?;
        }
        writeln!(w)?;
        writeln!(
            w,
            "  Welcome to ezCMD! Type what you want to do. Get the commands, in order."
        )?;
        writeln!(
            w,
            "  Fast, offline terminal cheatsheet with zero telemetry — commands in < 20ms.\n"
        )?;

        writeln!(
            w,
            "  +-- [ UPDATE ] ----------------------------------------------------------+"
        )?;
        writeln!(
            w,
            "  |  Focused answers: specific queries return only what you need.          |"
        )?;
        writeln!(
            w,
            "  |  Broad searches show a primary command with clear alternatives.        |"
        )?;
        writeln!(
            w,
            "  |  Workflows stay complete; warnings and choices are color-coded.        |"
        )?;
        writeln!(
            w,
            "  +------------------------------------------------------------------------+\n"
        )?;

        writeln!(
            w,
            "  +-- [ QUICK START / HOW TO USE ] ----------------------------------------+"
        )?;
        writeln!(
            w,
            "  |  ezcmd <query>            --- Search for any command or workflow       |"
        )?;
        writeln!(
            w,
            "  |  ezcmd --help             --- Browse categories and tools for your OS   |"
        )?;
        writeln!(
            w,
            "  |  ezcmd --help -git        --- View Git groups and section titles        |"
        )?;
        writeln!(
            w,
            "  |  ezcmd --help -shell      --- View Shell groups and section titles      |"
        )?;
        writeln!(
            w,
            "  |  ezcmd --uninstall        --- Uninstall ezcmd from your system         |"
        )?;
        writeln!(
            w,
            "  |  ezcmd update             --- Install the latest published release       |"
        )?;
        writeln!(
            w,
            "  +------------------------------------------------------------------------+\n"
        )?;

        writeln!(w, "  WORKFLOWS TO TRY:")?;
        writeln!(
            w,
            "    ezcmd initializing repo       # Starting a new Git repo"
        )?;
        writeln!(
            w,
            "    ezcmd undo last commit         # Safely revert commits"
        )?;
        writeln!(
            w,
            "    ezcmd how to branch            # Branching & merging workflows"
        )?;
        writeln!(
            w,
            "    ezcmd delete files             # File & directory cleanup\n"
        )?;

        writeln!(w, "  AUTHOR:")?;
        writeln!(w, "    Created by ItzEvhyx")?;
        writeln!(w, "    GitHub: https://github.com/ItzEvhyx")?;
        writeln!(w, "    Repo:   https://github.com/ItzEvhyx/ezCMD\n")?;
    }
    Ok(())
}
