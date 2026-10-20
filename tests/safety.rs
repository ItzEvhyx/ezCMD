use assert_cmd::Command;
use ezcmd::os::Os;
use ezcmd::shell::Shell;
use predicates::prelude::*;

/// Safety messaging must exercise the command syntax the spawned CLI uses on
/// the host. Windows PowerShell, Windows CMD, and Unix deliberately use
/// different recursive-delete commands; all must retain the same `DANGER:`
/// label and placeholder-only safety contract.
fn recursive_delete_template() -> &'static str {
    match (Os::current(), Shell::detect()) {
        (Os::Windows, Shell::PowerShell) => "Remove-Item -Recurse <folder>",
        (Os::Windows, Shell::Cmd) => "rmdir /s <folder>",
        (Os::Linux | Os::Macos, _) => "rm -r <folder>",
    }
}

#[test]
fn recursive_delete_is_never_the_default() {
    let recursive = recursive_delete_template();
    let mut command = Command::cargo_bin("ezcmd").expect("binary exists");
    command
        .args(["--plain", "delete", "folder"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "rmdir <folder> --- Removes an empty folder",
        ))
        .stdout(predicate::str::contains(format!("DANGER: {recursive}")));
}

#[test]
fn explicit_recursive_delete_is_labeled_and_previews_first() {
    let recursive = recursive_delete_template();
    let mut command = Command::cargo_bin("ezcmd").expect("binary exists");
    command
        .args(["--plain", "delete", "folder", "recursively"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("DANGER: {recursive}")))
        .stdout(predicate::str::contains("DANGER: This permanently deletes"))
        .stdout(predicate::str::contains("preview first: ls <folder>"));
}

#[test]
fn shell_injection_is_not_substituted_into_a_command() {
    let mut command = Command::cargo_bin("ezcmd").expect("binary exists");
    command
        .args([
            "--plain", "make", "a", "nextjs", "project", "named", "a;rm-rf",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("<project-name>"))
        .stdout(predicate::str::contains("a;rm-rf").not());
}

#[test]
fn system_paths_are_not_filled_into_the_delete_template() {
    let recursive = recursive_delete_template();
    let mut command = Command::cargo_bin("ezcmd").expect("binary exists");
    command
        .args(["--plain", "delete", "folder", "named", "/"])
        .assert()
        .success()
        .stdout(predicate::str::contains(recursive))
        .stdout(predicate::str::contains(recursive.replace("<folder>", "/")).not());
}

#[test]
fn system_path_attempt_explains_why_the_placeholder_was_kept() {
    let mut command = Command::cargo_bin("ezcmd").expect("binary exists");
    command
        .args(["--plain", "delete", "folder", "named", "/"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "ezcmd never fills in system or home paths for deleting commands.",
        ));
}

#[test]
fn protected_natural_language_delete_targets_are_blocked_before_ranking() {
    for query in [
        "delete everything",
        "delete root",
        "delete root file",
        "remove all my files",
        "delete home folder",
        "wipe my disk",
    ] {
        let mut command = Command::cargo_bin("ezcmd").expect("binary exists");
        command
            .arg("--plain")
            .args(query.split_whitespace())
            .assert()
            .code(1)
            .stdout(predicate::str::contains("Request blocked:"))
            .stdout(predicate::str::contains(
                "ezcmd never fills in system or home paths for deleting commands.",
            ))
            .stdout(predicate::str::contains("kubectl delete").not())
            .stdout(predicate::str::contains("rsync --delete").not())
            .stdout(predicate::str::contains("rm -r /").not());
    }
}
