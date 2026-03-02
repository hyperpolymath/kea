// SPDX-License-Identifier: AGPL-3.0-or-later
//! Slop-Gate: Runtime bloat rejection filter
//!
//! This crate provides functionality to detect and filter out non-functional
//! bloat from runtime paths. It identifies:
//! - Duplicate files
//! - Unnecessary development artifacts
//! - Orphaned dependencies
//! - Temporary and cache files

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use kea_beak::{AuditResult, Auditor, FileInfo, Finding, Severity};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Categories of bloat that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BloatCategory {
    /// Duplicate files with identical content
    Duplicate,
    /// Development-only files in production
    DevArtifact,
    /// Orphaned or unused dependencies
    OrphanedDependency,
    /// Temporary and cache files
    TemporaryFile,
    /// Large binary files that could be optimized
    LargeBinary,
    /// Unnecessary backup files
    BackupFile,
    /// Source maps in production
    SourceMap,
    /// Documentation in node_modules
    BundledDocs,
    /// Test files in production
    TestFile,
    /// Example files in dependencies
    ExampleFile,
}

impl std::fmt::Display for BloatCategory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BloatCategory::Duplicate => write!(formatter, "Duplicate"),
            BloatCategory::DevArtifact => write!(formatter, "Dev Artifact"),
            BloatCategory::OrphanedDependency => write!(formatter, "Orphaned Dependency"),
            BloatCategory::TemporaryFile => write!(formatter, "Temporary File"),
            BloatCategory::LargeBinary => write!(formatter, "Large Binary"),
            BloatCategory::BackupFile => write!(formatter, "Backup File"),
            BloatCategory::SourceMap => write!(formatter, "Source Map"),
            BloatCategory::BundledDocs => write!(formatter, "Bundled Docs"),
            BloatCategory::TestFile => write!(formatter, "Test File"),
            BloatCategory::ExampleFile => write!(formatter, "Example File"),
        }
    }
}

/// A detected piece of bloat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloatItem {
    /// Path to the bloated file or directory
    pub path: PathBuf,

    /// Category of bloat
    pub category: BloatCategory,

    /// Size in bytes
    pub size: u64,

    /// Description of why this is considered bloat
    pub reason: String,

    /// Recommended action
    pub action: BloatAction,
}

/// Recommended action for dealing with bloat
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BloatAction {
    /// Safe to delete
    Delete,
    /// Review before deleting
    Review,
    /// Consider optimizing
    Optimize,
    /// Ignore (false positive or intentional)
    Ignore,
}

/// Bloat detection report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BloatReport {
    /// All detected bloat items
    pub items: Vec<BloatItem>,

    /// Total size of detected bloat in bytes
    pub total_bloat_size: u64,

    /// Breakdown by category
    pub by_category: HashMap<String, CategoryStats>,
}

/// Statistics for a bloat category
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryStats {
    /// Number of items in this category
    pub count: usize,

    /// Total size in bytes
    pub size: u64,
}

impl BloatReport {
    /// Add an item to the report
    pub fn add_item(&mut self, item: BloatItem) {
        self.total_bloat_size += item.size;

        let category_name = item.category.to_string();
        let stats = self.by_category.entry(category_name).or_default();
        stats.count += 1;
        stats.size += item.size;

        self.items.push(item);
    }
}

/// Configuration for the Slop-Gate filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlopGateConfiguration {
    /// Enable duplicate detection
    pub detect_duplicates: bool,

    /// Enable dev artifact detection
    pub detect_dev_artifacts: bool,

    /// Enable temporary file detection
    pub detect_temp_files: bool,

    /// Enable large binary detection
    pub detect_large_binaries: bool,

    /// Threshold for large binary detection (bytes)
    pub large_binary_threshold: u64,

    /// Enable source map detection
    pub detect_source_maps: bool,

    /// Enable test file detection
    pub detect_test_files: bool,

    /// Patterns for development-only files
    pub dev_patterns: Vec<String>,

    /// Patterns for temporary files
    pub temp_patterns: Vec<String>,
}

impl Default for SlopGateConfiguration {
    fn default() -> Self {
        Self {
            detect_duplicates: true,
            detect_dev_artifacts: true,
            detect_temp_files: true,
            detect_large_binaries: true,
            large_binary_threshold: 10 * 1024 * 1024, // 10MB
            detect_source_maps: true,
            detect_test_files: true,
            dev_patterns: vec![
                ".eslintrc*".to_string(),
                ".prettierrc*".to_string(),
                "tsconfig*.json".to_string(),
                "jest.config.*".to_string(),
                "webpack.config.*".to_string(),
                ".babelrc*".to_string(),
                "Makefile".to_string(),
                "Gruntfile.*".to_string(),
                "Gulpfile.*".to_string(),
                ".editorconfig".to_string(),
            ],
            temp_patterns: vec![
                "*.tmp".to_string(),
                "*.temp".to_string(),
                "*.bak".to_string(),
                "*.swp".to_string(),
                "*.swo".to_string(),
                "*~".to_string(),
                "*.log".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                "*.cache".to_string(),
            ],
        }
    }
}

/// The main Slop-Gate bloat detector
pub struct SlopGate {
    /// Configuration for detection
    configuration: SlopGateConfiguration,

    /// Hash to paths mapping for duplicate detection
    hash_map: HashMap<String, Vec<PathBuf>>,
}

impl SlopGate {
    /// Create a new SlopGate with the given configuration
    pub fn new(configuration: SlopGateConfiguration) -> Self {
        Self {
            configuration,
            hash_map: HashMap::new(),
        }
    }

    /// Analyze files for bloat
    pub fn analyze(&mut self, files: &[FileInfo]) -> BloatReport {
        let mut report = BloatReport::default();

        // Build hash map for duplicate detection
        if self.configuration.detect_duplicates {
            self.build_hash_map(files);
            self.detect_duplicates(&mut report);
        }

        // Detect other types of bloat
        for file in files {
            if file.is_directory {
                continue;
            }

            // Dev artifacts
            if self.configuration.detect_dev_artifacts {
                if let Some(item) = self.check_dev_artifact(file) {
                    report.add_item(item);
                }
            }

            // Temporary files
            if self.configuration.detect_temp_files {
                if let Some(item) = self.check_temp_file(file) {
                    report.add_item(item);
                }
            }

            // Large binaries
            if self.configuration.detect_large_binaries {
                if let Some(item) = self.check_large_binary(file) {
                    report.add_item(item);
                }
            }

            // Source maps
            if self.configuration.detect_source_maps {
                if let Some(item) = self.check_source_map(file) {
                    report.add_item(item);
                }
            }

            // Test files
            if self.configuration.detect_test_files {
                if let Some(item) = self.check_test_file(file) {
                    report.add_item(item);
                }
            }

            // Bundled docs in node_modules
            if let Some(item) = self.check_bundled_docs(file) {
                report.add_item(item);
            }

            // Backup files
            if let Some(item) = self.check_backup_file(file) {
                report.add_item(item);
            }
        }

        info!(
            "Slop-Gate analysis complete: {} items, {} bytes of bloat",
            report.items.len(),
            report.total_bloat_size
        );

        report
    }

    /// Build hash map for duplicate detection
    fn build_hash_map(&mut self, files: &[FileInfo]) {
        self.hash_map.clear();

        for file in files {
            if file.is_directory {
                continue;
            }

            if let Some(ref hash) = file.hash {
                self.hash_map
                    .entry(hash.clone())
                    .or_default()
                    .push(file.path.clone());
            }
        }
    }

    /// Detect duplicate files
    fn detect_duplicates(&self, report: &mut BloatReport) {
        for paths in self.hash_map.values() {
            if paths.len() > 1 {
                // Skip the first file, mark the rest as duplicates
                for path in paths.iter().skip(1) {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        report.add_item(BloatItem {
                            path: path.clone(),
                            category: BloatCategory::Duplicate,
                            size: metadata.len(),
                            reason: format!(
                                "Duplicate of {} ({} copies total)",
                                paths[0].display(),
                                paths.len()
                            ),
                            action: BloatAction::Review,
                        });
                    }
                }
            }
        }
    }

    /// Check if file is a development artifact
    fn check_dev_artifact(&self, file: &FileInfo) -> Option<BloatItem> {
        let file_name = file.path.file_name()?.to_string_lossy();
        let path_str = file.path.to_string_lossy();

        // Check in node_modules context
        if !path_str.contains("node_modules") {
            return None;
        }

        for pattern in &self.configuration.dev_patterns {
            if self.matches_pattern(&file_name, pattern) {
                return Some(BloatItem {
                    path: file.path.clone(),
                    category: BloatCategory::DevArtifact,
                    size: file.size,
                    reason: format!("Development artifact in production: {}", pattern),
                    action: BloatAction::Delete,
                });
            }
        }

        None
    }

    /// Check if file is a temporary file
    fn check_temp_file(&self, file: &FileInfo) -> Option<BloatItem> {
        let file_name = file.path.file_name()?.to_string_lossy();

        for pattern in &self.configuration.temp_patterns {
            if self.matches_pattern(&file_name, pattern) {
                return Some(BloatItem {
                    path: file.path.clone(),
                    category: BloatCategory::TemporaryFile,
                    size: file.size,
                    reason: format!("Temporary file: {}", pattern),
                    action: BloatAction::Delete,
                });
            }
        }

        None
    }

    /// Check if file is a large binary
    fn check_large_binary(&self, file: &FileInfo) -> Option<BloatItem> {
        if file.size <= self.configuration.large_binary_threshold {
            return None;
        }

        // Check if it's a binary file (heuristic based on extension)
        let binary_extensions = [
            "exe", "dll", "so", "dylib", "a", "lib", "bin", "dat", "db", "sqlite", "pdf", "zip",
            "tar", "gz", "rar", "7z", "iso", "dmg", "pkg", "deb", "rpm",
        ];

        let extension = file.extension.as_ref()?;
        if binary_extensions.contains(&extension.as_str()) {
            return Some(BloatItem {
                path: file.path.clone(),
                category: BloatCategory::LargeBinary,
                size: file.size,
                reason: format!(
                    "Large binary file: {} bytes (threshold: {} bytes)",
                    file.size, self.configuration.large_binary_threshold
                ),
                action: BloatAction::Optimize,
            });
        }

        None
    }

    /// Check if file is a source map
    fn check_source_map(&self, file: &FileInfo) -> Option<BloatItem> {
        let extension = file.extension.as_ref()?;

        if extension == "map" {
            return Some(BloatItem {
                path: file.path.clone(),
                category: BloatCategory::SourceMap,
                size: file.size,
                reason: "Source map file in production".to_string(),
                action: BloatAction::Review,
            });
        }

        None
    }

    /// Check if file is a test file
    fn check_test_file(&self, file: &FileInfo) -> Option<BloatItem> {
        let path_str = file.path.to_string_lossy();
        let file_name = file.path.file_name()?.to_string_lossy();

        // Check for test directories and files in node_modules
        if !path_str.contains("node_modules") {
            return None;
        }

        let test_patterns = [
            "__tests__",
            "__test__",
            "__mocks__",
            "test/",
            "tests/",
            "spec/",
            ".test.",
            ".spec.",
            "_test.",
            "_spec.",
        ];

        for pattern in test_patterns {
            if path_str.contains(pattern) || file_name.contains(pattern) {
                return Some(BloatItem {
                    path: file.path.clone(),
                    category: BloatCategory::TestFile,
                    size: file.size,
                    reason: format!("Test file in production: {}", pattern),
                    action: BloatAction::Delete,
                });
            }
        }

        None
    }

    /// Check if file is bundled documentation
    fn check_bundled_docs(&self, file: &FileInfo) -> Option<BloatItem> {
        let path_str = file.path.to_string_lossy();
        let file_name = file.path.file_name()?.to_string_lossy();

        if !path_str.contains("node_modules") {
            return None;
        }

        // Common documentation patterns in node_modules
        let doc_patterns = [
            "README",
            "CHANGELOG",
            "HISTORY",
            "CONTRIBUTING",
            "LICENSE",
            "AUTHORS",
            "CONTRIBUTORS",
        ];

        let doc_extensions = ["md", "txt", "rst", "adoc"];

        let extension = file.extension.as_deref().unwrap_or("");
        let upper_name = file_name.to_uppercase();

        if doc_extensions.contains(&extension) {
            for pattern in doc_patterns {
                if upper_name.starts_with(pattern) {
                    return Some(BloatItem {
                        path: file.path.clone(),
                        category: BloatCategory::BundledDocs,
                        size: file.size,
                        reason: format!("Documentation file in node_modules: {}", file_name),
                        action: BloatAction::Delete,
                    });
                }
            }
        }

        None
    }

    /// Check if file is a backup file
    fn check_backup_file(&self, file: &FileInfo) -> Option<BloatItem> {
        let file_name = file.path.file_name()?.to_string_lossy();
        let extension = file.extension.as_ref()?;

        let backup_extensions = ["bak", "backup", "old", "orig", "original"];
        let backup_patterns = [".backup.", ".bak.", ".old.", "~"];

        if backup_extensions.contains(&extension.as_str()) {
            return Some(BloatItem {
                path: file.path.clone(),
                category: BloatCategory::BackupFile,
                size: file.size,
                reason: format!("Backup file: .{}", extension),
                action: BloatAction::Delete,
            });
        }

        for pattern in backup_patterns {
            if file_name.contains(pattern) {
                return Some(BloatItem {
                    path: file.path.clone(),
                    category: BloatCategory::BackupFile,
                    size: file.size,
                    reason: format!("Backup file: contains '{}'", pattern),
                    action: BloatAction::Delete,
                });
            }
        }

        None
    }

    /// Simple glob-style pattern matching
    fn matches_pattern(&self, file_name: &str, pattern: &str) -> bool {
        if let Some(suffix) = pattern.strip_prefix('*') {
            file_name.ends_with(suffix)
        } else if let Some(prefix) = pattern.strip_suffix('*') {
            file_name.starts_with(prefix)
        } else {
            file_name == pattern
        }
    }
}

/// Slop-Gate auditor that integrates with kea-beak
pub struct SlopGateAuditor {
    configuration: SlopGateConfiguration,
}

impl SlopGateAuditor {
    /// Create a new SlopGate auditor
    pub fn new(configuration: SlopGateConfiguration) -> Self {
        Self { configuration }
    }
}

impl Default for SlopGateAuditor {
    fn default() -> Self {
        Self::new(SlopGateConfiguration::default())
    }
}

#[async_trait]
impl Auditor for SlopGateAuditor {
    fn name(&self) -> &str {
        "Slop-Gate Bloat Detector"
    }

    fn description(&self) -> &str {
        "Detects and flags non-functional bloat in the runtime path"
    }

    async fn audit_file(&self, file: &FileInfo) -> AuditResult<Vec<Finding>> {
        let mut findings = Vec::new();

        // Create a temporary SlopGate for single-file analysis
        let slop_gate = SlopGate::new(self.configuration.clone());

        // Check various bloat categories
        if let Some(item) = slop_gate.check_temp_file(file) {
            findings.push(bloat_to_finding(item));
        }

        if let Some(item) = slop_gate.check_large_binary(file) {
            findings.push(bloat_to_finding(item));
        }

        if let Some(item) = slop_gate.check_source_map(file) {
            findings.push(bloat_to_finding(item));
        }

        if let Some(item) = slop_gate.check_test_file(file) {
            findings.push(bloat_to_finding(item));
        }

        if let Some(item) = slop_gate.check_bundled_docs(file) {
            findings.push(bloat_to_finding(item));
        }

        if let Some(item) = slop_gate.check_backup_file(file) {
            findings.push(bloat_to_finding(item));
        }

        if let Some(item) = slop_gate.check_dev_artifact(file) {
            findings.push(bloat_to_finding(item));
        }

        Ok(findings)
    }

    async fn audit_directory(
        &self,
        _path: &Path,
        files: &[FileInfo],
    ) -> AuditResult<Vec<Finding>> {
        let mut findings = Vec::new();

        // Run full SlopGate analysis including duplicate detection
        let mut slop_gate = SlopGate::new(self.configuration.clone());
        let report = slop_gate.analyze(files);

        // Convert bloat items to findings (just duplicates here as files are already processed)
        for item in report.items {
            if matches!(item.category, BloatCategory::Duplicate) {
                findings.push(bloat_to_finding(item));
            }
        }

        Ok(findings)
    }
}

/// Convert a BloatItem to a Finding
fn bloat_to_finding(item: BloatItem) -> Finding {
    let severity = match item.action {
        BloatAction::Delete => Severity::Low,
        BloatAction::Review => Severity::Low,
        BloatAction::Optimize => Severity::Info,
        BloatAction::Ignore => Severity::Info,
    };

    let finding_id = format!("SLOP{:03}", category_to_id(&item.category));

    Finding::new(finding_id, item.reason.clone(), severity, item.path)
        .with_context(format!("Category: {}, Size: {} bytes", item.category, item.size))
        .with_recommendation(action_recommendation(&item.action))
}

/// Map category to numeric ID
fn category_to_id(category: &BloatCategory) -> u32 {
    match category {
        BloatCategory::Duplicate => 1,
        BloatCategory::DevArtifact => 2,
        BloatCategory::OrphanedDependency => 3,
        BloatCategory::TemporaryFile => 4,
        BloatCategory::LargeBinary => 5,
        BloatCategory::BackupFile => 6,
        BloatCategory::SourceMap => 7,
        BloatCategory::BundledDocs => 8,
        BloatCategory::TestFile => 9,
        BloatCategory::ExampleFile => 10,
    }
}

/// Get recommendation text for an action
fn action_recommendation(action: &BloatAction) -> String {
    match action {
        BloatAction::Delete => "Safe to delete this file".to_string(),
        BloatAction::Review => "Review this file before deleting".to_string(),
        BloatAction::Optimize => "Consider optimizing or compressing this file".to_string(),
        BloatAction::Ignore => "Can be ignored if intentional".to_string(),
    }
}

/// Utility function to format bytes as human-readable size
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_slop_gate_detects_temp_files() {
        let config = SlopGateConfiguration::default();
        let slop_gate = SlopGate::new(config);

        let temp_file = FileInfo {
            path: PathBuf::from("/test/file.tmp"),
            size: 1024,
            is_directory: false,
            is_symlink: false,
            extension: Some("tmp".to_string()),
            hash: None,
            is_hidden: false,
        };

        let item = slop_gate.check_temp_file(&temp_file);
        assert!(item.is_some());
        assert_eq!(item.unwrap().category, BloatCategory::TemporaryFile);
    }

    #[test]
    fn test_slop_gate_detects_source_maps() {
        let config = SlopGateConfiguration::default();
        let slop_gate = SlopGate::new(config);

        let map_file = FileInfo {
            path: PathBuf::from("/dist/bundle.js.map"),
            size: 50000,
            is_directory: false,
            is_symlink: false,
            extension: Some("map".to_string()),
            hash: None,
            is_hidden: false,
        };

        let item = slop_gate.check_source_map(&map_file);
        assert!(item.is_some());
        assert_eq!(item.unwrap().category, BloatCategory::SourceMap);
    }

    #[test]
    fn test_slop_gate_detects_backup_files() {
        let config = SlopGateConfiguration::default();
        let slop_gate = SlopGate::new(config);

        let backup_file = FileInfo {
            path: PathBuf::from("/config/settings.bak"),
            size: 512,
            is_directory: false,
            is_symlink: false,
            extension: Some("bak".to_string()),
            hash: None,
            is_hidden: false,
        };

        let item = slop_gate.check_backup_file(&backup_file);
        assert!(item.is_some());
        assert_eq!(item.unwrap().category, BloatCategory::BackupFile);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_bloat_report() {
        let mut report = BloatReport::default();

        report.add_item(BloatItem {
            path: PathBuf::from("/test/file.tmp"),
            category: BloatCategory::TemporaryFile,
            size: 1000,
            reason: "Test".to_string(),
            action: BloatAction::Delete,
        });

        report.add_item(BloatItem {
            path: PathBuf::from("/test/file.map"),
            category: BloatCategory::SourceMap,
            size: 2000,
            reason: "Test".to_string(),
            action: BloatAction::Review,
        });

        assert_eq!(report.items.len(), 2);
        assert_eq!(report.total_bloat_size, 3000);
        assert_eq!(report.by_category.len(), 2);
    }
}
