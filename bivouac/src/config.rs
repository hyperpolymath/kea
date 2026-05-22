// SPDX-License-Identifier: MPL-2.0

//! Kea-Bivouac Configuration Kernel.
//!
//! This module defines the formal schema for the Bivouac controller. 
//! It uses `serde` for type-safe deserialization from TOML manifests.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::error::{BivouacError, Result};

/// SECURITY: Parameters for mutual TLS enforcement.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MtlsConfig {
    /// Whether mTLS is required for inbound connections.
    #[serde(default)]
    pub enabled: bool,
    /// Path to the CA certificate PEM file.
    pub ca_cert: Option<PathBuf>,
    /// Path to the server certificate PEM file.
    pub server_cert: Option<PathBuf>,
    /// Path to the server private key PEM file.
    pub server_key: Option<PathBuf>,
}

fn default_playbook_dir() -> PathBuf {
    PathBuf::from("/etc/kea/playbooks")
}

fn default_fluctuation_interval() -> u64 {
    3600
}

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
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path).map_err(|_| BivouacError::ConfigNotFound {
            path: path.display().to_string(),
        })?;
        let config: Config = toml::from_str(&contents).map_err(|e| BivouacError::InvalidConfig {
            message: e.to_string(),
        })?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mtls_config_default_disabled() {
        let mtls = MtlsConfig::default();
        assert!(!mtls.enabled);
        assert!(mtls.ca_cert.is_none());
        assert!(mtls.server_cert.is_none());
        assert!(mtls.server_key.is_none());
    }

    #[test]
    fn test_config_validate_passes_with_mtls_disabled() {
        let config = Config {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            playbook_dir: PathBuf::from("/etc/kea/playbooks"),
            mtls: MtlsConfig {
                enabled: false,
                ca_cert: None,
                server_cert: None,
                server_key: None,
            },
            deployment: Default::default(),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_fails_when_mtls_enabled_but_ca_cert_missing() {
        let config = Config {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            playbook_dir: PathBuf::from("/etc/kea/playbooks"),
            mtls: MtlsConfig {
                enabled: true,
                ca_cert: None,
                server_cert: None,
                server_key: None,
            },
            deployment: Default::default(),
        };
        assert!(config.validate().is_err());
        if let Err(BivouacError::InvalidConfig { message }) = config.validate() {
            assert!(message.contains("mTLS enabled"));
        }
    }

    #[test]
    fn test_config_validate_passes_when_mtls_enabled_with_ca_cert() {
        let config = Config {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            playbook_dir: PathBuf::from("/etc/kea/playbooks"),
            mtls: MtlsConfig {
                enabled: true,
                ca_cert: Some(PathBuf::from("/etc/tls/ca.pem")),
                server_cert: Some(PathBuf::from("/etc/tls/cert.pem")),
                server_key: Some(PathBuf::from("/etc/tls/key.pem")),
            },
            deployment: Default::default(),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_deployment_config_default_values() {
        let deployment = DeploymentConfig::default();
        assert!(!deployment.nomadic);
        // Default trait implementation gives 0, serde default gives 3600
        assert_eq!(deployment.fluctuation_interval_secs, 0);
        assert!(deployment.satellites.is_empty());
    }

    #[test]
    fn test_deployment_config_serde_defaults() {
        // When deserialized from TOML with nomadic but no interval, serde default applies
        let toml_content = "nomadic = true\n";
        let deployment: DeploymentConfig = toml::from_str(toml_content).expect("TODO: handle error");
        assert_eq!(deployment.fluctuation_interval_secs, 3600);
        assert!(deployment.nomadic);
    }
}
