// SPDX-License-Identifier: MPL-2.0

//! Kea-Bivouac — Strategic Orchestration and Deployment Controller.
//!
//! The Bivouac is the "Brain" of the Kea ecosystem. It coordinates the actions
//! of the "Flock" (satellite nodes) and manages the lifecycle of invisible 
//! infrastructure.
//!
//! DESIGN PATTERNS:
//! 1. **Nomadic Deployment**: Continuous rotation of infrastructure identifiers 
//!    (IPs, DNS records) to evade detection.
//! 2. **Administrative Isolation**: Complete separation of the "Wharf" (Admin) 
//!    from the "Range" (Runtime).
//! 3. **mTLS Everywhere**: Mandatory mutual TLS for all control-plane traffic.

#![forbid(unsafe_code)]
pub mod config;
pub mod error;
pub mod playbook;

// RE-EXPORTS: Primary types for consuming services.
pub use config::Config;
pub use error::{BivouacError, Result};
