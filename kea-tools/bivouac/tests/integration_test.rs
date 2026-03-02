// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration tests for Kea-Bivouac

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::tempdir;

/// Test the version command
#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("bivouac").unwrap();
    cmd.arg("version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Kea-Bivouac"))
        .stdout(predicate::str::contains("Command Authority"));
}

/// Test the help output
#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("bivouac").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Orchestration"))
        .stdout(predicate::str::contains("execute"))
        .stdout(predicate::str::contains("list"));
}

/// Test listing playbooks in empty directory
#[test]
fn test_list_empty() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("bivouac.toml");

    // Create a config pointing to empty playbook dir
    let config_content = format!(
        r#"name = "test"
playbook_dir = "{}"

[mtls]
enabled = false
"#,
        temp_dir.path().join("playbooks").display()
    );
    std::fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("bivouac").unwrap();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No playbooks found"));
}

/// Test listing playbooks with files
#[test]
fn test_list_with_playbooks() {
    let temp_dir = tempdir().unwrap();
    let playbook_dir = temp_dir.path().join("playbooks");
    std::fs::create_dir_all(&playbook_dir).unwrap();

    // Create test playbook
    std::fs::write(
        playbook_dir.join("test.toml"),
        r#"
name = "test"
[trigger]
type = "manual"
[[actions]]
type = "log"
level = "info"
message = "Test"
"#,
    )
    .unwrap();

    let config_path = temp_dir.path().join("bivouac.toml");
    let config_content = format!(
        r#"name = "test"
playbook_dir = "{}"

[mtls]
enabled = false
"#,
        playbook_dir.display()
    );
    std::fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("bivouac").unwrap();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

/// Test validating a playbook
#[test]
fn test_validate_playbook() {
    let temp_dir = tempdir().unwrap();
    let playbook_path = temp_dir.path().join("test.toml");

    std::fs::write(
        &playbook_path,
        r#"
name = "validate-test"
description = "A test playbook"

[trigger]
type = "manual"

[[actions]]
type = "log"
level = "info"
message = "Test action"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("bivouac").unwrap();
    cmd.arg("validate").arg(&playbook_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("validate-test"))
        .stdout(predicate::str::contains("is valid"));
}

/// Test validating an invalid playbook (missing actions)
#[test]
fn test_validate_invalid_playbook() {
    let temp_dir = tempdir().unwrap();
    let playbook_path = temp_dir.path().join("invalid.toml");

    std::fs::write(
        &playbook_path,
        r#"
name = "invalid-test"

[trigger]
type = "manual"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("bivouac").unwrap();
    cmd.arg("validate").arg(&playbook_path);
    cmd.assert()
        .failure();
}

/// Test executing a playbook in dry-run mode
#[test]
fn test_execute_dry_run() {
    let temp_dir = tempdir().unwrap();
    let playbook_path = temp_dir.path().join("test.toml");

    std::fs::write(
        &playbook_path,
        r#"
name = "dry-run-test"
description = "Test dry run"

[trigger]
type = "manual"

[[actions]]
type = "log"
level = "info"
message = "This is a test"

[[actions]]
type = "command"
cmd = "echo 'hello world'"
timeout_secs = 10
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("bivouac").unwrap();
    cmd.arg("--dry-run")
        .arg("execute")
        .arg(&playbook_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN"))
        .stdout(predicate::str::contains("completed successfully"));
}

/// Test executing a real playbook
#[test]
fn test_execute_real() {
    let temp_dir = tempdir().unwrap();
    let playbook_path = temp_dir.path().join("test.toml");

    std::fs::write(
        &playbook_path,
        r#"
name = "real-test"
description = "Test real execution"

[trigger]
type = "manual"

[[actions]]
type = "log"
level = "info"
message = "Starting test"

[[actions]]
type = "command"
cmd = "echo 'test passed'"
timeout_secs = 10
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("bivouac").unwrap();
    cmd.arg("-v")
        .arg("execute")
        .arg(&playbook_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("completed successfully"))
        .stdout(predicate::str::contains("Actions succeeded: 2"));
}

/// Test init command creates config file
#[test]
fn test_init_creates_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("bivouac.toml");

    let mut cmd = Command::cargo_bin("bivouac").unwrap();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("init");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Created configuration file"));

    assert!(config_path.exists());
    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("SPDX-License-Identifier"));
    assert!(content.contains("playbook_dir"));
}

/// Test init with --force overwrites existing config
#[test]
fn test_init_force() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("bivouac.toml");

    // Create initial file
    std::fs::write(&config_path, "old content").unwrap();

    let mut cmd = Command::cargo_bin("bivouac").unwrap();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("init")
        .arg("--force");
    cmd.assert().success();

    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(!content.contains("old content"));
    assert!(content.contains("name = \"bivouac\""));
}

/// Test config command shows defaults when no file exists
#[test]
fn test_config_defaults() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("nonexistent.toml");

    let mut cmd = Command::cargo_bin("bivouac").unwrap();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("config");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Using defaults"))
        .stdout(predicate::str::contains("bivouac"));
}

/// Test trigger command with missing playbook
#[test]
fn test_trigger_missing_playbook() {
    let temp_dir = tempdir().unwrap();
    let playbook_dir = temp_dir.path().join("playbooks");
    std::fs::create_dir_all(&playbook_dir).unwrap();

    let config_path = temp_dir.path().join("bivouac.toml");
    let config_content = format!(
        r#"name = "test"
playbook_dir = "{}"

[mtls]
enabled = false
"#,
        playbook_dir.display()
    );
    std::fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("bivouac").unwrap();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("trigger")
        .arg("nonexistent");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No playbook found"));
}
