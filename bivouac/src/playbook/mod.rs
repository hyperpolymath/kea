// SPDX-License-Identifier: PMPL-1.0-or-later

//! Kea-Bivouac Playbook Module.
//!
//! Playbooks are the "Recipes" of the Kea ecosystem. They define 
//! automated responses to triggers like integrity violations, 
//! failover events, or scheduled rotations.
//!
//! This module re-exports the primary parsing and execution interfaces.

mod parser;
mod executor;

pub use parser::{Playbook, PlaybookAction, PlaybookTrigger};
pub use executor::PlaybookExecutor;

use std::path::Path;
use crate::error::Result;

/// LOADER: High-level utility to load a playbook from a specific path.
pub fn load_playbook<P: AsRef<Path>>(path: P) -> Result<Playbook> {
    parser::Playbook::from_file(path)
}

/// DISCOVERY: Scans a directory for valid playbook files (*.toml, *.scm).
pub fn list_playbooks<P: AsRef<Path>>(dir: P) -> Result<Vec<String>> {
    let mut playbooks = Vec::new();
    for entry in std::fs::read_dir(dir.as_ref())? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext == "toml" || ext == "scm" {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    playbooks.push(name.to_string());
                }
            }
        }
    }
    Ok(playbooks)
}
