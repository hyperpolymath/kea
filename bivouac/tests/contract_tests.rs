// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Contract tests for Kea-Bivouac.
//!
//! These tests verify invariants and contracts that must hold for the system.

use kea_bivouac::playbook::{Playbook, PlaybookAction, PlaybookTrigger, PlaybookExecutor};
use kea_bivouac::Config;
use std::path::PathBuf;
use tempfile::tempdir;

// INVARIANT: PlaybookExecutor::execute always returns Ok (not Err) for valid playbooks in dry_run mode
#[tokio::test]
async fn contract_dry_run_always_succeeds_for_valid_playbook() {
    let executor = PlaybookExecutor { dry_run: true };
    let playbook = Playbook {
        name: "valid-playbook".to_string(),
        description: "A valid playbook".to_string(),
        trigger: PlaybookTrigger::Manual,
        actions: vec![PlaybookAction::Log {
            level: "info".to_string(),
            message: "Test".to_string(),
        }],
        continue_on_error: false,
        timeout_secs: 30,
    };

    let result = executor.execute(&playbook).await;
    assert!(result.is_ok(), "Dry run should always succeed for valid playbooks");
}

// INVARIANT: Playbook::from_toml with invalid TOML returns Err
#[test]
fn contract_invalid_toml_returns_error() {
    let invalid_toml = "{ this is [ not valid ] toml }";
    let path = std::path::Path::new("test.toml");
    let result = Playbook::from_toml(invalid_toml, path);
    assert!(result.is_err(), "Invalid TOML should return error");
}

// INVARIANT: Playbook::from_toml with missing required field returns Err
#[test]
fn contract_missing_required_field_returns_error() {
    let toml_missing_name = r#"
description = "Missing name"
[trigger]
type = "manual"
[[actions]]
type = "log"
level = "info"
message = "Test"
"#;
    let path = std::path::Path::new("test.toml");
    let result = Playbook::from_toml(toml_missing_name, path);
    assert!(result.is_err(), "Missing required field should return error");
}

// INVARIANT: Config::validate returns Ok(()) for default config
#[test]
fn contract_default_config_validates() {
    let config = Config {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        playbook_dir: PathBuf::from("/etc/kea/playbooks"),
        mtls: Default::default(),
        deployment: Default::default(),
    };
    assert!(config.validate().is_ok(), "Default config must be valid");
}

// INVARIANT: list_playbooks on non-existent dir returns Err
#[test]
fn contract_list_playbooks_nonexistent_dir_returns_error() {
    let result = kea_bivouac::playbook::list_playbooks("/nonexistent/path/12345");
    assert!(result.is_err(), "list_playbooks on non-existent dir should return error");
}

// INVARIANT: list_playbooks returns only .toml and .scm files
#[test]
fn contract_list_playbooks_filters_extensions() {
    let temp_dir = tempdir().unwrap();

    // Create various file types
    std::fs::write(temp_dir.path().join("playbook.toml"), "").unwrap();
    std::fs::write(temp_dir.path().join("script.scm"), "").unwrap();
    std::fs::write(temp_dir.path().join("readme.txt"), "").unwrap();
    std::fs::write(temp_dir.path().join("config.json"), "").unwrap();

    let playbooks = kea_bivouac::playbook::list_playbooks(temp_dir.path()).unwrap();

    assert_eq!(playbooks.len(), 2, "Should only find .toml and .scm files");
    assert!(playbooks.contains(&"playbook.toml".to_string()));
    assert!(playbooks.contains(&"script.scm".to_string()));
    assert!(!playbooks.contains(&"readme.txt".to_string()));
    assert!(!playbooks.contains(&"config.json".to_string()));
}

// INVARIANT: Playbook with empty actions is valid
#[test]
fn contract_empty_actions_is_valid() {
    let playbook = Playbook {
        name: "empty-actions".to_string(),
        description: "Playbook with no actions".to_string(),
        trigger: PlaybookTrigger::Manual,
        actions: vec![],
        continue_on_error: false,
        timeout_secs: 60,
    };

    // Should serialize without error
    let serialized = toml::to_string(&playbook).unwrap();
    assert!(serialized.contains("empty-actions"));

    // Should deserialize without error
    let path = std::path::Path::new("test.toml");
    let deserialized = Playbook::from_toml(&serialized, path).unwrap();
    assert_eq!(deserialized.actions.len(), 0);
}

// INVARIANT: PlaybookTrigger::Manual can be serialized and deserialized
#[test]
fn contract_manual_trigger_roundtrip() {
    let trigger = PlaybookTrigger::Manual;
    let serialized = toml::to_string(&trigger).unwrap();
    let deserialized: PlaybookTrigger = toml::from_str(&serialized).unwrap();
    assert_eq!(trigger, deserialized);
}

// INVARIANT: Config with mTLS enabled but missing ca_cert is invalid
#[test]
fn contract_mtls_enabled_without_ca_cert_is_invalid() {
    let config = Config {
        name: "bad-mtls".to_string(),
        version: "1.0.0".to_string(),
        playbook_dir: PathBuf::from("/etc/kea/playbooks"),
        mtls: kea_bivouac::config::MtlsConfig {
            enabled: true,
            ca_cert: None,
            server_cert: Some(PathBuf::from("/etc/tls/cert.pem")),
            server_key: Some(PathBuf::from("/etc/tls/key.pem")),
        },
        deployment: Default::default(),
    };
    assert!(
        config.validate().is_err(),
        "mTLS enabled without ca_cert should be invalid"
    );
}

// INVARIANT: Playbook from file with unsupported extension returns error
#[test]
fn contract_unsupported_file_extension_returns_error() {
    let temp_dir = tempdir().unwrap();
    let yaml_path = temp_dir.path().join("playbook.yaml");
    std::fs::write(&yaml_path, "name: test").unwrap();

    let result = Playbook::from_file(&yaml_path);
    assert!(
        result.is_err(),
        "Unsupported file extension should return error"
    );
}

// INVARIANT: SCM format files return not-implemented error
#[test]
fn contract_scm_format_not_yet_implemented() {
    let temp_dir = tempdir().unwrap();
    let scm_path = temp_dir.path().join("playbook.scm");
    std::fs::write(&scm_path, "(playbook (name \"test\"))").unwrap();

    let result = Playbook::from_file(&scm_path);
    assert!(
        result.is_err(),
        "SCM format should return error (not yet implemented)"
    );
}
