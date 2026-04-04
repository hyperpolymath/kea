// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! End-to-end tests for Kea-Bivouac.
//!
//! These tests verify complete workflows from playbook creation to execution.

use kea_bivouac::playbook::{Playbook, PlaybookAction, PlaybookTrigger, PlaybookExecutor};
use tempfile::tempdir;

#[test]
fn test_e2e_playbook_creation_serialization_roundtrip() {
    // Create a playbook in memory
    let playbook = Playbook {
        name: "e2e-test".to_string(),
        description: "End-to-end test playbook".to_string(),
        trigger: PlaybookTrigger::Manual,
        actions: vec![
            PlaybookAction::Log {
                level: "info".to_string(),
                message: "Starting E2E test".to_string(),
            },
            PlaybookAction::Wait {
                duration_secs: 5,
            },
            PlaybookAction::Log {
                level: "info".to_string(),
                message: "E2E test completed".to_string(),
            },
        ],
        continue_on_error: false,
        timeout_secs: 60,
    };

    // Serialize to TOML
    let serialized = toml::to_string(&playbook).unwrap();
    assert!(serialized.contains("e2e-test"));
    assert!(serialized.contains("End-to-end test playbook"));

    // Parse back from TOML
    let path = std::path::Path::new("e2e-test.toml");
    let deserialized = Playbook::from_toml(&serialized, path).unwrap();

    // Verify fields round-trip correctly
    assert_eq!(deserialized.name, "e2e-test");
    assert_eq!(deserialized.description, "End-to-end test playbook");
    assert_eq!(deserialized.timeout_secs, 60);
    assert!(!deserialized.continue_on_error);
    assert_eq!(deserialized.actions.len(), 3);
    assert!(matches!(deserialized.trigger, PlaybookTrigger::Manual));
}

#[test]
fn test_e2e_playbook_file_roundtrip() {
    let temp_dir = tempdir().unwrap();
    let playbook_path = temp_dir.path().join("e2e-playbook.toml");

    let toml_content = r#"name = "file-roundtrip-test"
description = "Test file roundtrip"
continue_on_error = true
timeout_secs = 120

[trigger]
type = "schedule"
cron = "0 */6 * * *"

[[actions]]
type = "log"
level = "debug"
message = "Scheduled execution"

[[actions]]
type = "wait"
duration_secs = 30

[[actions]]
type = "log"
level = "info"
message = "Task completed"
"#;

    std::fs::write(&playbook_path, toml_content).unwrap();

    // Load from file
    let playbook = Playbook::from_file(&playbook_path).unwrap();

    // Verify loaded playbook
    assert_eq!(playbook.name, "file-roundtrip-test");
    assert!(playbook.continue_on_error);
    assert_eq!(playbook.timeout_secs, 120);
    assert_eq!(playbook.actions.len(), 3);
    assert!(matches!(
        playbook.trigger,
        PlaybookTrigger::Schedule { .. }
    ));
}

#[tokio::test]
async fn test_e2e_playbook_creation_and_execution() {
    // Create a playbook
    let playbook = Playbook {
        name: "execution-test".to_string(),
        description: "Test execution pipeline".to_string(),
        trigger: PlaybookTrigger::Manual,
        actions: vec![
            PlaybookAction::Log {
                level: "info".to_string(),
                message: "Execution starting".to_string(),
            },
        ],
        continue_on_error: false,
        timeout_secs: 30,
    };

    // Create executor in dry-run mode
    let executor = PlaybookExecutor { dry_run: true };

    // Execute the playbook
    let result = executor.execute(&playbook).await.unwrap();

    // Verify results
    assert_eq!(result.playbook_name, "execution-test");
    assert!(result.success);
}

#[test]
fn test_e2e_complex_playbook_with_multiple_triggers() {
    let triggers = vec![
        PlaybookTrigger::Manual,
        PlaybookTrigger::Schedule {
            cron: "0 0 * * *".to_string(),
        },
        PlaybookTrigger::HealthCheckFailure {
            service: "api".to_string(),
        },
        PlaybookTrigger::Deployment {
            event: "rollout-complete".to_string(),
        },
    ];

    for trigger in triggers {
        let playbook = Playbook {
            name: format!("test-{:?}", trigger),
            description: "Multi-trigger test".to_string(),
            trigger: trigger.clone(),
            actions: vec![PlaybookAction::Log {
                level: "info".to_string(),
                message: "Test".to_string(),
            }],
            continue_on_error: false,
            timeout_secs: 300,
        };

        // Serialize and deserialize
        let serialized = toml::to_string(&playbook).unwrap();
        let path = std::path::Path::new("test.toml");
        let deserialized = Playbook::from_toml(&serialized, path).unwrap();

        // Verify trigger survived roundtrip
        assert_eq!(
            std::mem::discriminant(&playbook.trigger),
            std::mem::discriminant(&deserialized.trigger)
        );
    }
}

#[test]
fn test_e2e_playbook_with_all_action_types() {
    let actions = vec![
        PlaybookAction::Command {
            command: "systemctl".to_string(),
            args: vec!["restart".to_string(), "nginx".to_string()],
            timeout_secs: Some(60),
        },
        PlaybookAction::Log {
            level: "info".to_string(),
            message: "Service restarted".to_string(),
        },
        PlaybookAction::Wait {
            duration_secs: 10,
        },
        PlaybookAction::RotateDns {
            zone: "example.com".to_string(),
            record: "api".to_string(),
        },
        PlaybookAction::RestartService {
            service: "postgresql".to_string(),
        },
    ];

    let playbook = Playbook {
        name: "all-actions-test".to_string(),
        description: "Test all action types".to_string(),
        trigger: PlaybookTrigger::Manual,
        actions,
        continue_on_error: false,
        timeout_secs: 300,
    };

    let serialized = toml::to_string(&playbook).unwrap();
    let path = std::path::Path::new("test.toml");
    let deserialized = Playbook::from_toml(&serialized, path).unwrap();

    assert_eq!(deserialized.actions.len(), 5);
    assert!(matches!(deserialized.actions[0], PlaybookAction::Command { .. }));
    assert!(matches!(deserialized.actions[1], PlaybookAction::Log { .. }));
    assert!(matches!(deserialized.actions[2], PlaybookAction::Wait { .. }));
    assert!(matches!(deserialized.actions[3], PlaybookAction::RotateDns { .. }));
    assert!(matches!(deserialized.actions[4], PlaybookAction::RestartService { .. }));
}
