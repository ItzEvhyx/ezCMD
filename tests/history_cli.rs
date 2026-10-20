use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_config_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ezcmd-history-cli-{unique}"));
    fs::create_dir_all(&path).expect("temporary config directory");
    path
}

#[test]
fn history_stores_recipe_ids_only_and_forget_clears_it() {
    let config = temp_config_dir();
    let mut query = Command::cargo_bin("ezcmd").expect("binary exists");
    query
        .env("XDG_CONFIG_HOME", &config)
        .env_remove("EZCMD_NO_HISTORY")
        .args(["--plain", "make", "a", "nextjs", "project"])
        .assert()
        .success();

    let history_path = config.join("ezcmd/history");
    let history = fs::read_to_string(&history_path).expect("history file");
    assert!(history.starts_with("next.create\t"));
    assert!(!history.contains("make a nextjs project"));

    let mut forget = Command::cargo_bin("ezcmd").expect("binary exists");
    forget
        .env("XDG_CONFIG_HOME", &config)
        .env_remove("EZCMD_NO_HISTORY")
        .arg("--forget")
        .assert()
        .success()
        .stdout(predicate::str::contains("history cleared"));
    assert!(!history_path.exists());
    fs::remove_dir_all(config).expect("cleanup");
}

#[test]
fn history_opt_out_never_creates_a_file() {
    let config = temp_config_dir();
    let mut query = Command::cargo_bin("ezcmd").expect("binary exists");
    query
        .env("XDG_CONFIG_HOME", &config)
        .env("EZCMD_NO_HISTORY", "1")
        .args(["--plain", "make", "a", "nextjs", "project"])
        .assert()
        .success();
    assert!(!config.join("ezcmd/history").exists());
    fs::remove_dir_all(config).expect("cleanup");
}
