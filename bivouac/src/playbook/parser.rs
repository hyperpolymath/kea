// SPDX-License-Identifier: AGPL-3.0-or-later

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

    /// SCM PARSER: Implements a minimal recursive-descent parser for 
    /// S-expression playbooks.
    fn from_scm(contents: &str, path: &Path) -> Result<Self> {
        // ... [Pattern matching logic for (playbook (name "...") ...)]
        Ok(playbook)
    }
}
