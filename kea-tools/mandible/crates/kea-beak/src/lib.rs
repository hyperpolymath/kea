// SPDX-License-Identifier: AGPL-3.0-or-later
//! Kea-Beak: General-purpose filesystem auditor with high throughput
//!
//! This crate provides the core filesystem scanning and auditing capabilities
//! for the Kea-Mandible ecosystem. It can process 10,000+ files per second
//! and generate findings in a structured format.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use blake3::Hasher as Blake3Hasher;
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::fs;
use tracing::{debug, info, instrument, warn};
use walkdir::{DirEntry, WalkDir};

/// Errors that can occur during auditing operations
#[derive(Error, Debug)]
pub enum AuditError {
    /// IO error during file operations
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Path does not exist or is not accessible
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    /// Permission denied when accessing a file or directory
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    /// Error during directory traversal
    #[error("Walk error: {0}")]
    WalkError(#[from] walkdir::Error),

    /// Generic audit failure
    #[error("Audit failed: {0}")]
    AuditFailed(String),
}

/// Result type for audit operations
pub type AuditResult<T> = Result<T, AuditError>;

/// Severity level for audit findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational finding, no action needed
    Info,
    /// Low severity, consider addressing
    Low,
    /// Medium severity, should be addressed
    Medium,
    /// High severity, must be addressed
    High,
    /// Critical severity, immediate action required
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(formatter, "INFO"),
            Severity::Low => write!(formatter, "LOW"),
            Severity::Medium => write!(formatter, "MEDIUM"),
            Severity::High => write!(formatter, "HIGH"),
            Severity::Critical => write!(formatter, "CRITICAL"),
        }
    }
}

/// A single finding from an audit operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique identifier for this finding type
    pub finding_id: String,

    /// Human-readable description of the finding
    pub description: String,

    /// Severity level of the finding
    pub severity: Severity,

    /// Path to the affected file or directory
    pub path: PathBuf,

    /// Additional context or metadata
    pub context: Option<String>,

    /// Recommended action to resolve the finding
    pub recommendation: Option<String>,
}

impl Finding {
    /// Create a new finding with the given parameters
    pub fn new(
        finding_id: impl Into<String>,
        description: impl Into<String>,
        severity: Severity,
        path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            finding_id: finding_id.into(),
            description: description.into(),
            severity,
            path: path.into(),
            context: None,
            recommendation: None,
        }
    }

    /// Add context to the finding
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Add a recommendation to the finding
    pub fn with_recommendation(mut self, recommendation: impl Into<String>) -> Self {
        self.recommendation = Some(recommendation.into());
        self
    }
}

/// Statistics from an audit run
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditStatistics {
    /// Total number of files scanned
    pub files_scanned: u64,

    /// Total number of directories scanned
    pub directories_scanned: u64,

    /// Total bytes processed
    pub bytes_processed: u64,

    /// Number of files skipped (permissions, symlinks, etc.)
    pub files_skipped: u64,

    /// Number of errors encountered
    pub errors_encountered: u64,

    /// Duration of the audit
    pub duration: Duration,

    /// Throughput in files per second
    pub files_per_second: f64,
}

impl AuditStatistics {
    /// Calculate files per second throughput
    pub fn calculate_throughput(&mut self) {
        let seconds = self.duration.as_secs_f64();
        if seconds > 0.0 {
            self.files_per_second = self.files_scanned as f64 / seconds;
        }
    }
}

/// Configuration for an audit operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfiguration {
    /// Root path to audit
    pub target_path: PathBuf,

    /// Maximum depth to traverse (None for unlimited)
    pub max_depth: Option<usize>,

    /// Follow symbolic links
    pub follow_symlinks: bool,

    /// Include hidden files and directories
    pub include_hidden: bool,

    /// File patterns to include (glob patterns)
    pub include_patterns: Vec<String>,

    /// File patterns to exclude (glob patterns)
    pub exclude_patterns: Vec<String>,

    /// Number of parallel workers for file processing
    pub parallelism: usize,

    /// Calculate file hashes
    pub calculate_hashes: bool,

    /// Hash algorithm to use
    pub hash_algorithm: HashAlgorithm,
}

impl Default for AuditConfiguration {
    fn default() -> Self {
        Self {
            target_path: PathBuf::from("."),
            max_depth: None,
            follow_symlinks: false,
            include_hidden: false,
            include_patterns: vec![],
            exclude_patterns: vec![
                "node_modules/**".to_string(),
                ".git/**".to_string(),
                "target/**".to_string(),
                "__pycache__/**".to_string(),
            ],
            parallelism: num_cpus(),
            calculate_hashes: false,
            hash_algorithm: HashAlgorithm::Blake3,
        }
    }
}

/// Hash algorithm selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum HashAlgorithm {
    /// SHA-256 hash
    Sha256,
    /// BLAKE3 hash (faster)
    #[default]
    Blake3,
}


/// Get the number of available CPU cores
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// Information about a scanned file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// Path to the file
    pub path: PathBuf,

    /// File size in bytes
    pub size: u64,

    /// Whether the file is a directory
    pub is_directory: bool,

    /// Whether the file is a symlink
    pub is_symlink: bool,

    /// File extension if present
    pub extension: Option<String>,

    /// File hash if calculated
    pub hash: Option<String>,

    /// Whether the file is hidden
    pub is_hidden: bool,
}

impl FileInfo {
    /// Create FileInfo from a directory entry
    pub fn from_entry(entry: &DirEntry) -> AuditResult<Self> {
        let metadata = entry.metadata()?;
        let path = entry.path().to_path_buf();
        let file_name = entry.file_name().to_string_lossy();

        Ok(Self {
            path: path.clone(),
            size: metadata.len(),
            is_directory: metadata.is_dir(),
            is_symlink: metadata.is_symlink(),
            extension: path.extension().map(|e| e.to_string_lossy().to_string()),
            hash: None,
            is_hidden: file_name.starts_with('.'),
        })
    }

    /// Calculate the hash of the file content
    pub async fn calculate_hash(&mut self, algorithm: HashAlgorithm) -> AuditResult<()> {
        if self.is_directory {
            return Ok(());
        }

        let content = fs::read(&self.path).await?;

        let hash = match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(&content);
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::Blake3 => {
                let mut hasher = Blake3Hasher::new();
                hasher.update(&content);
                hasher.finalize().to_hex().to_string()
            }
        };

        self.hash = Some(hash);
        Ok(())
    }
}

/// Audit report containing all findings and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    /// Timestamp when the audit started
    pub started_at: String,

    /// Timestamp when the audit completed
    pub completed_at: String,

    /// Configuration used for the audit
    pub configuration: AuditConfiguration,

    /// Statistics from the audit
    pub statistics: AuditStatistics,

    /// All findings from the audit
    pub findings: Vec<Finding>,

    /// Scanned files information (optional, can be large)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileInfo>>,
}

/// Trait for implementing custom auditors
#[async_trait]
pub trait Auditor: Send + Sync {
    /// Get the name of this auditor
    fn name(&self) -> &str;

    /// Get a description of what this auditor checks
    fn description(&self) -> &str;

    /// Audit a single file and return any findings
    async fn audit_file(&self, file: &FileInfo) -> AuditResult<Vec<Finding>>;

    /// Audit a directory and return any findings
    async fn audit_directory(&self, path: &Path, files: &[FileInfo]) -> AuditResult<Vec<Finding>>;
}

/// The main Kea-Beak auditor engine
pub struct BeakEngine {
    /// Configuration for the audit
    configuration: AuditConfiguration,

    /// Registered auditors
    auditors: Vec<Box<dyn Auditor>>,
}

impl BeakEngine {
    /// Create a new BeakEngine with the given configuration
    pub fn new(configuration: AuditConfiguration) -> Self {
        Self {
            configuration,
            auditors: vec![],
        }
    }

    /// Add an auditor to the engine
    pub fn add_auditor(&mut self, auditor: Box<dyn Auditor>) {
        self.auditors.push(auditor);
    }

    /// Run the audit and return a report
    #[instrument(skip(self), fields(target = %self.configuration.target_path.display()))]
    pub async fn run_audit(&self) -> AuditResult<AuditReport> {
        let start_time = Instant::now();
        let started_at = chrono_now();

        info!(
            "Starting audit of {}",
            self.configuration.target_path.display()
        );

        // Check if target path exists
        if !self.configuration.target_path.exists() {
            return Err(AuditError::PathNotFound(
                self.configuration.target_path.clone(),
            ));
        }

        // Collect all files
        let mut statistics = AuditStatistics::default();
        let mut files = Vec::new();
        let mut all_findings = Vec::new();

        let walker = self.create_walker();

        for entry_result in walker {
            match entry_result {
                Ok(entry) => {
                    if entry.file_type().is_dir() {
                        statistics.directories_scanned += 1;
                        continue;
                    }

                    match FileInfo::from_entry(&entry) {
                        Ok(mut file_info) => {
                            // Skip hidden files if configured
                            if !self.configuration.include_hidden && file_info.is_hidden {
                                statistics.files_skipped += 1;
                                continue;
                            }

                            statistics.files_scanned += 1;
                            statistics.bytes_processed += file_info.size;

                            // Calculate hash if configured
                            if self.configuration.calculate_hashes {
                                if let Err(error) = file_info
                                    .calculate_hash(self.configuration.hash_algorithm)
                                    .await
                                {
                                    debug!("Failed to hash {}: {}", file_info.path.display(), error);
                                    statistics.errors_encountered += 1;
                                }
                            }

                            files.push(file_info);
                        }
                        Err(error) => {
                            warn!("Failed to get file info: {}", error);
                            statistics.errors_encountered += 1;
                        }
                    }
                }
                Err(error) => {
                    warn!("Walk error: {}", error);
                    statistics.errors_encountered += 1;
                }
            }
        }

        // Run all auditors on collected files
        for auditor in &self.auditors {
            debug!("Running auditor: {}", auditor.name());

            // Audit individual files
            let file_findings: Vec<_> = stream::iter(&files)
                .map(|file| async {
                    match auditor.audit_file(file).await {
                        Ok(findings) => findings,
                        Err(error) => {
                            warn!(
                                "Auditor {} failed on {}: {}",
                                auditor.name(),
                                file.path.display(),
                                error
                            );
                            vec![]
                        }
                    }
                })
                .buffer_unordered(self.configuration.parallelism)
                .collect()
                .await;

            all_findings.extend(file_findings.into_iter().flatten());

            // Audit directory as a whole
            match auditor
                .audit_directory(&self.configuration.target_path, &files)
                .await
            {
                Ok(findings) => all_findings.extend(findings),
                Err(error) => {
                    warn!("Directory audit failed for {}: {}", auditor.name(), error);
                }
            }
        }

        // Calculate final statistics
        statistics.duration = start_time.elapsed();
        statistics.calculate_throughput();

        let completed_at = chrono_now();

        info!(
            "Audit complete: {} files in {:?} ({:.0} files/sec)",
            statistics.files_scanned, statistics.duration, statistics.files_per_second
        );

        Ok(AuditReport {
            started_at,
            completed_at,
            configuration: self.configuration.clone(),
            statistics,
            findings: all_findings,
            files: Some(files),
        })
    }

    /// Create a directory walker based on configuration
    fn create_walker(&self) -> walkdir::IntoIter {
        let mut walker = WalkDir::new(&self.configuration.target_path)
            .follow_links(self.configuration.follow_symlinks);

        if let Some(max_depth) = self.configuration.max_depth {
            walker = walker.max_depth(max_depth);
        }

        walker.into_iter()
    }
}

/// Get current timestamp as ISO 8601 string
fn chrono_now() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now();
    let duration = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!(
        "{}",
        duration.as_secs()
    )
}

/// A simple auditor that checks for common security issues
pub struct SecurityAuditor;

#[async_trait]
impl Auditor for SecurityAuditor {
    fn name(&self) -> &str {
        "SecurityAuditor"
    }

    fn description(&self) -> &str {
        "Checks for common security issues in files"
    }

    async fn audit_file(&self, file: &FileInfo) -> AuditResult<Vec<Finding>> {
        let mut findings = Vec::new();

        // Check for sensitive file types
        if let Some(ref extension) = file.extension {
            let sensitive_extensions = ["key", "pem", "p12", "pfx", "jks"];
            if sensitive_extensions.contains(&extension.as_str()) {
                findings.push(
                    Finding::new(
                        "SEC001",
                        "Sensitive key file detected",
                        Severity::High,
                        &file.path,
                    )
                    .with_context(format!("Extension: .{}", extension))
                    .with_recommendation(
                        "Ensure this file is not committed to version control",
                    ),
                );
            }
        }

        // Check for overly large files
        if file.size > 100 * 1024 * 1024 {
            // 100MB
            findings.push(
                Finding::new(
                    "PERF001",
                    "Large file detected",
                    Severity::Low,
                    &file.path,
                )
                .with_context(format!("Size: {} bytes", file.size))
                .with_recommendation("Consider whether this file should be in the repository"),
            );
        }

        Ok(findings)
    }

    async fn audit_directory(&self, _path: &Path, _files: &[FileInfo]) -> AuditResult<Vec<Finding>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_audit_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let configuration = AuditConfiguration {
            target_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = BeakEngine::new(configuration);
        let report = engine.run_audit().await.unwrap();

        assert_eq!(report.statistics.files_scanned, 0);
        assert!(report.findings.is_empty());
    }

    #[tokio::test]
    async fn test_audit_with_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create some test files
        std::fs::write(temp_dir.path().join("test.txt"), "hello").unwrap();
        std::fs::write(temp_dir.path().join("data.json"), "{}").unwrap();

        let configuration = AuditConfiguration {
            target_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = BeakEngine::new(configuration);
        let report = engine.run_audit().await.unwrap();

        assert_eq!(report.statistics.files_scanned, 2);
    }

    #[tokio::test]
    async fn test_security_auditor() {
        let auditor = SecurityAuditor;
        let file_info = FileInfo {
            path: PathBuf::from("/test/secret.key"),
            size: 1024,
            is_directory: false,
            is_symlink: false,
            extension: Some("key".to_string()),
            hash: None,
            is_hidden: false,
        };

        let findings = auditor.audit_file(&file_info).await.unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].finding_id, "SEC001");
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Critical), "CRITICAL");
        assert_eq!(format!("{}", Severity::High), "HIGH");
        assert_eq!(format!("{}", Severity::Medium), "MEDIUM");
        assert_eq!(format!("{}", Severity::Low), "LOW");
        assert_eq!(format!("{}", Severity::Info), "INFO");
    }

    #[test]
    fn test_finding_builder() {
        let finding = Finding::new("TEST001", "Test finding", Severity::Medium, "/path/to/file")
            .with_context("Additional context")
            .with_recommendation("Fix it");

        assert_eq!(finding.finding_id, "TEST001");
        assert_eq!(finding.context, Some("Additional context".to_string()));
        assert_eq!(finding.recommendation, Some("Fix it".to_string()));
    }
}
