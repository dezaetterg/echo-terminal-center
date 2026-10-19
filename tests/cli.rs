//! Integration tests for echo-terminal CLI.
//!
//! These tests spawn the binary and verify exit codes, output shapes, and
//! argument parsing — without requiring a real TTY or system-specific hardware.

use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("echo-terminal").expect("binary must be built")
}

#[test]
fn version_flag_prints_version() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("echo-terminal "));
}

#[test]
fn short_version_flag() {
    cmd()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("echo-terminal "));
}

#[test]
fn help_flag_prints_usage() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE"))
        .stdout(predicate::str::contains("fetch"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--short"));
}

#[test]
fn short_help_flag() {
    cmd()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE"));
}

#[test]
fn unknown_flag_exits_nonzero() {
    cmd().arg("--this-flag-does-not-exist").assert().failure();
}

#[test]
fn json_output_is_valid_json() {
    let output = cmd().arg("--json").output().expect("process must run");

    assert!(output.status.success(), "exit code must be 0");

    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("--json output must be valid JSON");

    assert!(parsed.is_object(), "JSON root must be an object");
}

#[test]
fn json_output_contains_required_keys() {
    let output = cmd().arg("--json").output().expect("process must run");

    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let obj = parsed.as_object().unwrap();

    for key in &["os", "kernel", "shell", "cpu", "memory", "disk"] {
        assert!(obj.contains_key(*key), "JSON must contain key '{key}'");
    }
}

#[test]
fn short_output_is_single_line() {
    let output = cmd().arg("--short").output().expect("process must run");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "--short must emit exactly one line");
}

#[test]
fn short_output_contains_pipe_separators() {
    let output = cmd().arg("--short").output().expect("process must run");

    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    assert!(
        stdout.contains('|'),
        "--short output must contain pipe separators"
    );
}

#[test]
fn fetch_exits_zero() {
    cmd().arg("fetch").assert().success();
}
