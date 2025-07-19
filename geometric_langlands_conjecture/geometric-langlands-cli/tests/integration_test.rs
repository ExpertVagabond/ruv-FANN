use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("langlands").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Geometric Langlands CLI"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("langlands").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_config_show() {
    let mut cmd = Command::cargo_bin("langlands").unwrap();
    cmd.arg("config").arg("show")
        .assert()
        .success();
}

#[test]
fn test_db_init() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let mut cmd = Command::cargo_bin("langlands").unwrap();
    cmd.env("LANGLANDS_DB_PATH", db_path.to_str().unwrap())
        .arg("db").arg("init")
        .assert()
        .success();
}

#[test]
fn test_completions() {
    let mut cmd = Command::cargo_bin("langlands").unwrap();
    cmd.arg("completions").arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("langlands"));
}