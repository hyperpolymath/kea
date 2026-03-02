// SPDX-License-Identifier: AGPL-3.0-or-later

//! Kea-Bivouac Configuration Kernel.
//!
//! This module defines the formal schema for the Bivouac controller. 
//! It uses `serde` for type-safe deserialization from TOML manifests.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::error::{BivouacError, Result};

/// MASTER CONFIG: The top-level specification for a Bivouac instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub version: String,
    
    /// Path to the directory containing response playbooks (*.toml/*.scm).
    #[serde(default = "default_playbook_dir")]
    pub playbook_dir: PathBuf,

    /// SECURITY: Parameters for mutual TLS enforcement.
    #[serde(default)]
    pub mtls: MtlsConfig,

    /// ORCHESTRATION: Logic for nomadic and satellite management.
    #[serde(default)]
    pub deployment: DeploymentConfig,
}

/// NOMADIC STRATEGY: Configuration for infrastructure rotation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// If true, the system will continuously rotate IP/DNS identifiers.
    #[serde(default)]
    pub nomadic: bool,

    /// The interval (in seconds) between nomadic fluctuations.
    #[serde(default = "default_fluctuation_interval")]
    pub fluctuation_interval_secs: u64,

    /// List of target satellite node identifiers.
    #[serde(default)]
    pub satellites: Vec<String>,
}

impl Config {
    /// LOADER: Reads and validates a configuration file from disk.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        // ... [File reading and TOML parsing logic]
        Ok(config)
    }

    /// VALIDATION: Enforces cross-field constraints (e.g. mTLS requirements).
    pub fn validate(&self) -> Result<()> {
        if self.mtls.enabled && self.mtls.ca_cert.is_none() {
            return Err(BivouacError::InvalidConfig {
                message: "mTLS enabled but ca_cert is missing".to_string(),
            });
        }
        Ok(())
    }
}
