use std::io::IsTerminal;

pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[90m";
pub const CYAN_BOLD: &str = "\x1b[1;36m";
pub const GREEN_BOLD: &str = "\x1b[1;32m";
pub const MAGENTA_BOLD: &str = "\x1b[1;35m";
pub const YELLOW: &str = "\x1b[33m";
pub const RED_BOLD: &str = "\x1b[1;31m";
/// Explicit RGB red keeps safety text red even when a terminal theme remaps
/// the normal ANSI red palette to purple or pink.
pub const WARNING_RED: &str = "\x1b[1;38;2;255;78;78m";
pub const CHOICE_GREEN: &str = "\x1b[1;38;2;74;222;128m";
pub const CHOICE_AMBER: &str = "\x1b[1;38;2;251;191;36m";
pub const CHOICE_PINK: &str = "\x1b[1;38;2;244;114;182m";
pub const CHOICE_BLUE: &str = "\x1b[1;38;2;96;165;250m";
pub const UNDERLINE: &str = "\x1b[4m";
pub const CMD_COLOR: &str = "\x1b[1;38;2;150;208;251m";
pub const RESET: &str = "\x1b[0m";

#[cfg(windows)]
fn init_windows_vt() -> bool {
    enable_ansi_support::enable_ansi_support().is_ok()
}

#[cfg(not(windows))]
fn init_windows_vt() -> bool {
    true
}

/// True only if stdout is a terminal, plain is false, NO_COLOR is unset or empty,
/// and TERM is not dumb. On Windows, calls enable_ansi_support() first and returns
/// false if it fails.
pub fn color_enabled(plain: bool) -> bool {
    if plain {
        return false;
    }
    if !std::io::stdout().is_terminal() {
        return false;
    }
    if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
        return false;
    }
    if std::env::var("TERM").is_ok_and(|v| v == "dumb") {
        return false;
    }
    init_windows_vt()
}

/// Terminal width from terminal_size, falling back to 80.
pub fn width() -> usize {
    terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(80)
}

/// Give adjacent interactive choices distinct whole-row colors. The sequence
/// repeats for menus with more than four choices.
pub fn choice_color(index: usize) -> &'static str {
    [CHOICE_GREEN, CHOICE_AMBER, CHOICE_PINK, CHOICE_BLUE][index % 4]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_always_disables_color() {
        assert!(!color_enabled(true));
    }

    #[test]
    fn test_width_fallback() {
        let w = width();
        assert!(w >= 20, "terminal width should be reasonable");
    }

    #[test]
    fn test_style_constants() {
        assert_eq!(BOLD, "\x1b[1m");
        assert_eq!(DIM, "\x1b[90m");
        assert_eq!(CYAN_BOLD, "\x1b[1;36m");
        assert_eq!(GREEN_BOLD, "\x1b[1;32m");
        assert_eq!(MAGENTA_BOLD, "\x1b[1;35m");
        assert_eq!(YELLOW, "\x1b[33m");
        assert_eq!(RED_BOLD, "\x1b[1;31m");
        assert_eq!(WARNING_RED, "\x1b[1;38;2;255;78;78m");
        assert_eq!(UNDERLINE, "\x1b[4m");
        assert_eq!(CMD_COLOR, "\x1b[1;38;2;150;208;251m");
        assert_eq!(RESET, "\x1b[0m");
    }

    #[test]
    fn choice_colors_cycle_in_a_stable_order() {
        assert_eq!(choice_color(0), CHOICE_GREEN);
        assert_eq!(choice_color(1), CHOICE_AMBER);
        assert_eq!(choice_color(2), CHOICE_PINK);
        assert_eq!(choice_color(3), CHOICE_BLUE);
        assert_eq!(choice_color(4), CHOICE_GREEN);
    }
}
