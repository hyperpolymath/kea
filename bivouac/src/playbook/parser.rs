// SPDX-License-Identifier: MPL-2.0

//! Kea-Bivouac Playbook Parser.
//!
//! This module implements the loading and parsing of "Response Playbooks". 
//! It supports two distinct file formats:
//! 1. **TOML**: The preferred format for complex, modern playbooks.
//! 2. **S-Expression (SCM)**: A legacy format used for compatibility 
//!    with symbolic logic-based definitions.

use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::error::{BivouacError, Result};

/// PLAYBOOK: The formal specification of a response strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playbook {
    pub name: String,
    pub description: String,
    pub trigger: PlaybookTrigger, // What activates the playbook
    pub actions: Vec<PlaybookAction>, // What the playbook does
    pub continue_on_error: bool,
    pub timeout_secs: u64,
}

/// ACTION: A single step in a response playbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum PlaybookAction {
    /// Run a shell command with optional timeout.
    Command { command: String, args: Vec<String>, timeout_secs: Option<u64> },
    /// Emit a structured log event.
    Log { level: String, message: String },
    /// Pause execution for a duration.
    Wait { duration_secs: u64 },
    /// Trigger DNS record rotation via RRecord Fluctuator.
    RotateDns { zone: String, record: String },
    /// Orchestrate a safe service restart.
    RestartService { service: String },
}

/// TRIGGER: Defines the activation event for a playbook.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum PlaybookTrigger {
    Manual,
    Schedule { cron: String },
    IntegrityViolation { severity: String },
    Deployment { event: String },
    HealthCheckFailure { service: String },
    Webhook { path: String, secret: Option<String> },
}

impl Playbook {
    /// LOADER: Dispatches to the appropriate parser based on file extension.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)?;
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            "toml" => Self::from_toml(&contents, path),
            "scm" => Self::from_scm(&contents, path),
            _ => Err(BivouacError::PlaybookParseError {
                path: path.display().to_string(),
                message: "Unsupported file extension".to_string(),
            }),
        }
    }

    /// TOML PARSER: Deserializes a TOML playbook definition.
    pub fn from_toml(contents: &str, path: &Path) -> Result<Self> {
        toml::from_str(contents).map_err(|e| BivouacError::PlaybookParseError {
            path: path.display().to_string(),
            message: e.to_string(),
        })
    }

    /// SCM PARSER: Minimal S-expression playbook parser (legacy format).
    /// Returns an error for malformed SCM — full parsing not yet implemented.
    fn from_scm(_contents: &str, path: &Path) -> Result<Self> {
        Err(BivouacError::PlaybookParseError {
            path: path.display().to_string(),
            message: "SCM playbook format not yet fully implemented".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playbook_trigger_manual_serde() {
        let trigger = PlaybookTrigger::Manual;
        let serialized = toml::to_string(&trigger).expect("TODO: handle error");
        let deserialized: PlaybookTrigger = toml::from_str(&serialized).expect("TODO: handle error");
        assert_eq!(trigger, deserialized);
    }

    #[test]
    fn test_playbook_trigger_schedule_serde() {
        let trigger = PlaybookTrigger::Schedule {
            cron: "0 0 * * *".to_string(),
        };
        let serialized = toml::to_string(&trigger).expect("TODO: handle error");
        let deserialized: PlaybookTrigger = toml::from_str(&serialized).expect("TODO: handle error");
        assert_eq!(trigger, deserialized);
    }

    #[test]
    fn test_playbook_trigger_health_check_failure_serde() {
        let trigger = PlaybookTrigger::HealthCheckFailure {
            service: "api-server".to_string(),
        };
        let serialized = toml::to_string(&trigger).expect("TODO: handle error");
        let deserialized: PlaybookTrigger = toml::from_str(&serialized).expect("TODO: handle error");
        assert_eq!(trigger, deserialized);
    }

    #[test]
    fn test_playbook_action_command_construction() {
        let action = PlaybookAction::Command {
            command: "systemctl".to_string(),
            args: vec!["restart".to_string(), "nginx".to_string()],
            timeout_secs: Some(30),
        };
        match action {
            PlaybookAction::Command { command, args, timeout_secs } => {
                assert_eq!(command, "systemctl");
                assert_eq!(args.len(), 2);
                assert_eq!(timeout_secs, Some(30));
            }
            _ => panic!("Expected Command variant"),
        }
    }

    #[test]
    fn test_playbook_action_log_construction() {
        let action = PlaybookAction::Log {
            level: "info".to_string(),
            message: "Test log message".to_string(),
        };
        match action {
            PlaybookAction::Log { level, message } => {
                assert_eq!(level, "info");
                assert_eq!(message, "Test log message");
            }
            _ => panic!("Expected Log variant"),
        }
    }

    #[test]
    fn test_playbook_action_wait_construction() {
        let action = PlaybookAction::Wait {
            duration_secs: 60,
        };
        match action {
            PlaybookAction::Wait { duration_secs } => {
                assert_eq!(duration_secs, 60);
            }
            _ => panic!("Expected Wait variant"),
        }
    }

    #[test]
    fn test_playbook_action_rotate_dns_construction() {
        let action = PlaybookAction::RotateDns {
            zone: "example.com".to_string(),
            record: "api".to_string(),
        };
        match action {
            PlaybookAction::RotateDns { zone, record } => {
                assert_eq!(zone, "example.com");
                assert_eq!(record, "api");
            }
            _ => panic!("Expected RotateDns variant"),
        }
    }

    #[test]
    fn test_playbook_action_restart_service_construction() {
        let action = PlaybookAction::RestartService {
            service: "postgresql".to_string(),
        };
        match action {
            PlaybookAction::RestartService { service } => {
                assert_eq!(service, "postgresql");
            }
            _ => panic!("Expected RestartService variant"),
        }
    }

    #[test]
    fn test_playbook_action_wait_roundtrip_serde() {
        let action = PlaybookAction::Wait {
            duration_secs: 120,
        };
        let serialized = toml::to_string(&action).expect("TODO: handle error");
        let deserialized: PlaybookAction = toml::from_str(&serialized).expect("TODO: handle error");
        match (action, deserialized) {
            (
                PlaybookAction::Wait { duration_secs: d1 },
                PlaybookAction::Wait { duration_secs: d2 },
            ) => {
                assert_eq!(d1, d2);
            }
            _ => panic!("Serde roundtrip failed"),
        }
    }

    #[test]
    fn test_playbook_from_toml_with_valid_content() {
        let toml_content = r#"
name = "test-playbook"
description = "A test playbook"
continue_on_error = false
timeout_secs = 300

[trigger]
type = "manual"

[[actions]]
type = "log"
level = "info"
message = "Starting playbook"
"#;
        let path = std::path::Path::new("test.toml");
        let playbook = Playbook::from_toml(toml_content, path).expect("TODO: handle error");
        assert_eq!(playbook.name, "test-playbook");
        assert_eq!(playbook.description, "A test playbook");
        assert!(!playbook.continue_on_error);
        assert_eq!(playbook.timeout_secs, 300);
        assert!(matches!(playbook.trigger, PlaybookTrigger::Manual));
        assert_eq!(playbook.actions.len(), 1);
    }

    #[test]
    fn test_playbook_from_toml_with_invalid_content() {
        let toml_content = "{ invalid toml [[ ]";
        let path = std::path::Path::new("test.toml");
        assert!(Playbook::from_toml(toml_content, path).is_err());
    }

    #[test]
    fn test_playbook_action_command_serde() {
        let action = PlaybookAction::Command {
            command: "bash".to_string(),
            args: vec!["-c".to_string(), "echo 'hello'".to_string()],
            timeout_secs: Some(10),
        };
        let serialized = toml::to_string(&action).expect("TODO: handle error");
        let deserialized: PlaybookAction = toml::from_str(&serialized).expect("TODO: handle error");
        match (action, deserialized) {
            (
                PlaybookAction::Command { command: c1, args: a1, timeout_secs: t1 },
                PlaybookAction::Command { command: c2, args: a2, timeout_secs: t2 },
            ) => {
                assert_eq!(c1, c2);
                assert_eq!(a1, a2);
                assert_eq!(t1, t2);
            }
            _ => panic!("Serde roundtrip failed"),
        }
    }
}
