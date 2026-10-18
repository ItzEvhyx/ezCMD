use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "ezcmd {}",
            env!("CARGO_PKG_VERSION")
        )));
}

#[test]
fn test_usage_no_args() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("USAGE:"))
        .stdout(predicate::str::contains("ezcmd <query>"));
}

#[test]
fn test_query_initializing_repo() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["initializing", "repo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initializing a Repo"))
        .stdout(predicate::str::contains("git init"))
        .stdout(predicate::str::contains("git push -u origin main"));
}

#[test]
fn test_plain_output_no_ansi() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["--plain", "initializing", "repo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# ---Initializing a Repo---"))
        .stdout(predicate::str::contains(
            "git init --- Initializes a repo in the current folder",
        ))
        .stdout(predicate::str::contains("\x1b[").not());
}

#[test]
fn test_no_color_env_no_ansi() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["initializing", "repo"])
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initializing a Repo"))
        .stdout(predicate::str::contains("\x1b[").not());
}

#[test]
fn test_no_match_exit_code_1() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["xyznonexistentrandomquery123"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("No match for"));
}

#[test]
fn test_unknown_flag_exit_code_2() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["--bogus-flag"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unknown flag"));
}

#[test]
fn test_piped_stdin_does_not_block() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["repo"]).assert().success();
}

#[test]
fn test_help_all() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE:"))
        .stdout(predicate::str::contains("TOOL: git"))
        .stdout(predicate::str::contains("TOOL: shell"))
        .stdout(predicate::str::contains("Initializing a Repo"))
        .stdout(predicate::str::contains("Navigating Directories"));
}

#[test]
fn test_help_tool_git_only() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["--help", "-git"])
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE:"))
        .stdout(predicate::str::contains("Initializing a Repo"))
        .stdout(predicate::str::contains("Navigating Directories").not())
        .stdout(predicate::str::contains("TOOL: shell").not());
}

#[test]
fn test_help_tool_shell_only() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["--help", "-shell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE:"))
        .stdout(predicate::str::contains("Navigating Directories"))
        .stdout(predicate::str::contains("Initializing a Repo").not())
        .stdout(predicate::str::contains("TOOL: git").not());
}

#[test]
fn test_help_windows_view() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["--help", "-windows"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rmdir /s /q <folder>"))
        .stdout(predicate::str::contains("type nul > <file>"))
        .stdout(predicate::str::contains("cls"));
}

#[test]
fn test_help_linux_view() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["--help", "-linux"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rm -rf <folder>"))
        .stdout(predicate::str::contains("touch <file>"))
        .stdout(predicate::str::contains("clear"));
}

#[test]
fn test_help_git_single_section() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["--help", "-git", "-initialize", "repo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initializing a Repo"))
        .stdout(predicate::str::contains("git init"))
        .stdout(predicate::str::contains("USAGE:").not())
        .stdout(predicate::str::contains("TOOL:").not())
        .stdout(predicate::str::contains("Cloning a Repo").not());
}

#[test]
fn test_help_git_fuzzy_section() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["--help", "-git", "-start", "repo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initializing a Repo"))
        .stdout(predicate::str::contains("git init"));
}

#[test]
fn test_help_windows_git_single_section() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["--help", "-windows", "-git", "-initialize", "repo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initializing a Repo"))
        .stdout(predicate::str::contains("git init"));
}

#[test]
fn test_help_unknown_selector_searches_sections() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["--help", "-nonexistent"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("No match for \"nonexistent\""));
}

#[test]
fn test_help_plain_mode() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["--plain", "--help", "-git"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# ---Initializing a Repo---"))
        .stdout(predicate::str::contains("\x1b[").not());
}

#[test]
fn test_fuzzy_search_uninstall() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["uninstall"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Uninstalling ezcmd"))
        .stdout(predicate::str::contains("ezcmd --uninstall"));
}

#[test]
fn test_welcome_flag() {
    let mut cmd = Command::cargo_bin("ezcmd").expect("binary exists");
    cmd.args(["--welcome"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Welcome to ezCMD!"))
        .stdout(predicate::str::contains("ItzEvhyx"))
        .stdout(predicate::str::contains("https://github.com/ItzEvhyx"));
}
