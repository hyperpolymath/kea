// SPDX-License-Identifier: MPL-2.0

//! Kea-Bivouac Error Space.
//!
//! This module defines all possible failure modes for the Bivouac controller.
//! It uses the `thiserror` crate to provide high-fidelity error messages 
//! while maintaining low-overhead error propagation.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, BivouacError>;

#[derive(Error, Debug)]
pub enum BivouacError {
    /// CONFIG: The required bivouac.toml file is missing.
    #[error("Configuration file not found: {path}")]
    ConfigNotFound { path: String },

    /// SPEC: The playbook file is syntactically or logically malformed.
    #[error("Failed to parse playbook '{path}': {message}")]
    PlaybookParseError { path: String, message: String },

    /// EXECUTION: A concrete playbook action (e.g. Command) failed at runtime.
    #[error("Action '{action}' failed: {message}")]
    ActionFailed { action: String, message: String },

    /// SECURITY: Failure during mTLS negotiation or certificate loading.
    #[error("mTLS configuration error: {message}")]
    MtlsError { message: String },

    /// CONFIG: A configuration field is invalid or inconsistent.
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    /// BRIDGE: Wrapped IO or serialization errors.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// BRIDGE: Wrapped TOML deserialization errors.
    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),
}
