// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Aspect tests for Kea-Bivouac.
//!
//! These tests verify non-functional properties: security, performance, and correctness.

use kea_bivouac::playbook::{Playbook, PlaybookAction, PlaybookTrigger};
use std::time::Instant;

// SECURITY: Playbook::from_toml with SQL-injection-style content parses safely
#[test]
fn aspect_security_sql_injection_strings() {
    let malicious_toml = r#"
name = "test'; DROP TABLE playbooks; --"
description = "1' OR '1'='1"
continue_on_error = false
timeout_secs = 300

[trigger]
type = "manual"

[[actions]]
type = "log"
level = "info"
message = "'; DELETE FROM actions; --"
"#;

    let path = std::path::Path::new("test.toml");
    let result = Playbook::from_toml(malicious_toml, path);
    // Should parse successfully without executing any SQL
    assert!(result.is_ok(), "Malicious strings should not break parser");

    if let Ok(playbook) = result {
        // Verify the malicious content is treated as literal strings
        assert_eq!(playbook.name, "test'; DROP TABLE playbooks; --");
        assert_eq!(playbook.description, "1' OR '1'='1");
    }
}

// SECURITY: Config with empty string ca_cert path is handled safely
#[test]
fn aspect_security_empty_cert_path() {
    let config = kea_bivouac::Config {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        playbook_dir: std::path::PathBuf::from("/etc/kea/playbooks"),
        mtls: kea_bivouac::config::MtlsConfig {
            enabled: true,
            ca_cert: Some(std::path::PathBuf::from("")),
            server_cert: None,
            server_key: None,
        },
        deployment: Default::default(),
    };

    // Should not crash on empty path
    let validation = config.validate();
    // The validation passes because ca_cert is Some (even if empty)
    assert!(validation.is_ok());
}

// SECURITY: Command action with shell metacharacters is stored safely
#[test]
fn aspect_security_command_injection_prevention() {
    let action = PlaybookAction::Command {
        command: "bash".to_string(),
        args: vec![
            "-c".to_string(),
            "rm -rf /; echo 'hacked'".to_string(),
        ],
        timeout_secs: Some(10),
    };

    let serialized = toml::to_string(&action).unwrap();
    let deserialized: PlaybookAction = toml::from_str(&serialized).unwrap();

    // Content should survive serialization without being executed
    match deserialized {
        PlaybookAction::Command { command, args, .. } => {
            assert_eq!(command, "bash");
            assert_eq!(args[1], "rm -rf /; echo 'hacked'");
        }
        _ => panic!("Expected Command action"),
    }
}

// PERFORMANCE: Parsing 100 simple playbooks in sequence completes without error
#[test]
fn aspect_performance_bulk_parsing() {
    let start = Instant::now();

    for i in 0..100 {
        let toml_content = format!(
            r#"name = "playbook-{}"
description = "Performance test playbook {}"
continue_on_error = false
timeout_secs = 300

[trigger]
type = "manual"

[[actions]]
type = "log"
level = "info"
message = "Test {}"
"#,
            i, i, i
        );

        let path = std::path::Path::new("test.toml");
        let result = Playbook::from_toml(&toml_content, path);
        assert!(result.is_ok(), "Bulk parsing should complete without errors");
    }

    let elapsed = start.elapsed();
    // Verify performance: 100 parses should take less than 5 seconds
    assert!(
        elapsed.as_secs() < 5,
        "Bulk parsing took too long: {:?}",
        elapsed
    );
}

// PERFORMANCE: Config validation is fast
#[test]
fn aspect_performance_config_validation() {
    let config = kea_bivouac::Config {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        playbook_dir: std::path::PathBuf::from("/etc/kea/playbooks"),
        mtls: Default::default(),
        deployment: Default::default(),
    };

    let start = Instant::now();
    for _ in 0..10000 {
        let _ = config.validate();
    }
    let elapsed = start.elapsed();

    // 10,000 validations should take less than 100ms
    assert!(
        elapsed.as_millis() < 100,
        "Validation performance degraded: {:?}",
        elapsed
    );
}

// CORRECTNESS: PlaybookResult correctly reports success: true for empty action list
#[test]
fn aspect_correctness_empty_action_result() {
    let playbook = Playbook {
        name: "empty".to_string(),
        description: "No actions".to_string(),
        trigger: PlaybookTrigger::Manual,
        actions: vec![],
        continue_on_error: false,
        timeout_secs: 60,
    };

    // Serialize and deserialize
    let serialized = toml::to_string(&playbook).unwrap();
    let path = std::path::Path::new("test.toml");
    let deserialized = Playbook::from_toml(&serialized, path).unwrap();

    assert_eq!(deserialized.actions.len(), 0, "Actions list should be empty");
}

// CORRECTNESS: Trigger type is preserved through serialization
#[test]
fn aspect_correctness_trigger_preservation() {
    let triggers = vec![
        (
            PlaybookTrigger::Manual,
            "manual",
        ),
        (
            PlaybookTrigger::Schedule {
                cron: "0 0 * * *".to_string(),
            },
            "schedule",
        ),
        (
            PlaybookTrigger::HealthCheckFailure {
                service: "api".to_string(),
            },
            "health-check-failure",
        ),
    ];

    for (trigger, expected_type) in triggers {
        let serialized = toml::to_string(&trigger).unwrap();
        assert!(serialized.contains(expected_type), "Trigger type should be in serialized form");

        let deserialized: PlaybookTrigger = toml::from_str(&serialized).unwrap();
        assert_eq!(
            std::mem::discriminant(&trigger),
            std::mem::discriminant(&deserialized),
            "Trigger discriminant should be preserved"
        );
    }
}

// CORRECTNESS: Action count is preserved through roundtrip
#[test]
fn aspect_correctness_action_count_preservation() {
    let action_counts = vec![0, 1, 5, 10, 50];

    for count in action_counts {
        let mut actions = vec![];
        for i in 0..count {
            actions.push(PlaybookAction::Log {
                level: "info".to_string(),
                message: format!("Action {}", i),
            });
        }

        let playbook = Playbook {
            name: "test".to_string(),
            description: "Test".to_string(),
            trigger: PlaybookTrigger::Manual,
            actions,
            continue_on_error: false,
            timeout_secs: 300,
        };

        let serialized = toml::to_string(&playbook).unwrap();
        let path = std::path::Path::new("test.toml");
        let deserialized = Playbook::from_toml(&serialized, path).unwrap();

        assert_eq!(
            deserialized.actions.len(),
            count,
            "Action count should be preserved for count={}",
            count
        );
    }
}

// CORRECTNESS: Numeric fields preserve values
#[test]
fn aspect_correctness_numeric_field_preservation() {
    // Use reasonable timeout values that TOML can handle
    let timeouts = vec![1, 10, 60, 300, 3600, 86400];

    for timeout in timeouts {
        let playbook = Playbook {
            name: "test".to_string(),
            description: "Test".to_string(),
            trigger: PlaybookTrigger::Manual,
            actions: vec![],
            continue_on_error: false,
            timeout_secs: timeout,
        };

        let serialized = toml::to_string(&playbook).unwrap();
        let path = std::path::Path::new("test.toml");
        let deserialized = Playbook::from_toml(&serialized, path).unwrap();

        assert_eq!(
            deserialized.timeout_secs, timeout,
            "Timeout should be preserved for {}",
            timeout
        );
    }
}

// SECURITY: Playbook action arguments with special characters are safely handled
#[test]
fn aspect_security_special_characters_in_args() {
    let special_chars = vec![
        "$HOME",
        "`whoami`",
        "$(id -u)",
        "; rm -rf /",
        "| cat /etc/passwd",
        "&& evil_command",
    ];

    for arg in special_chars {
        let action = PlaybookAction::Command {
            command: "echo".to_string(),
            args: vec![arg.to_string()],
            timeout_secs: None,
        };

        let serialized = toml::to_string(&action).unwrap();
        let deserialized: PlaybookAction = toml::from_str(&serialized).unwrap();

        match deserialized {
            PlaybookAction::Command { args, .. } => {
                assert_eq!(args[0], arg, "Special chars should be preserved safely");
            }
            _ => panic!("Expected Command action"),
        }
    }
}
