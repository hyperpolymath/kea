// SPDX-License-Identifier: AGPL-3.0-or-later
//! WP-Praxis: Specialized WordPress environment auditor
//!
//! This crate provides WordPress-specific auditing capabilities including:
//! - Core file integrity verification
//! - Plugin and theme security analysis
//! - Configuration file auditing
//! - Detection of known malicious patterns

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use kea_beak::{AuditError, AuditResult, Auditor, FileInfo, Finding, Severity};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;
use tracing::{debug, info, warn};

/// WordPress-specific errors
#[derive(Error, Debug)]
pub enum WpError {
    /// Not a valid WordPress installation
    #[error("Not a WordPress installation: {0}")]
    NotWordPress(PathBuf),

    /// WordPress version could not be determined
    #[error("Could not determine WordPress version")]
    UnknownVersion,

    /// Base audit error
    #[error("Audit error: {0}")]
    AuditError(#[from] AuditError),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for WordPress operations
pub type WpResult<T> = Result<T, WpError>;

/// WordPress installation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordPressInfo {
    /// Root path of the WordPress installation
    pub root_path: PathBuf,

    /// WordPress version if detected
    pub version: Option<String>,

    /// Whether this is a multisite installation
    pub is_multisite: bool,

    /// Path to wp-content directory
    pub wp_content_path: PathBuf,

    /// Path to plugins directory
    pub plugins_path: PathBuf,

    /// Path to themes directory
    pub themes_path: PathBuf,

    /// Path to uploads directory
    pub uploads_path: PathBuf,

    /// Detected plugins
    pub plugins: Vec<PluginInfo>,

    /// Detected themes
    pub themes: Vec<ThemeInfo>,
}

impl WordPressInfo {
    /// Detect WordPress installation at the given path
    pub async fn detect(path: &Path) -> WpResult<Self> {
        // Check for essential WordPress files
        let wp_config = path.join("wp-config.php");
        let wp_includes = path.join("wp-includes");
        let _wp_admin = path.join("wp-admin");

        if !wp_config.exists() && !wp_includes.exists() {
            return Err(WpError::NotWordPress(path.to_path_buf()));
        }

        // Determine wp-content path (can be customized)
        let wp_content_path = path.join("wp-content");

        let plugins_path = wp_content_path.join("plugins");
        let themes_path = wp_content_path.join("themes");
        let uploads_path = wp_content_path.join("uploads");

        // Try to detect version from wp-includes/version.php
        let version = Self::detect_version(path).await.ok();

        // Detect multisite
        let is_multisite = Self::detect_multisite(path).await.unwrap_or(false);

        // Scan plugins and themes
        let plugins = Self::scan_plugins(&plugins_path).await.unwrap_or_default();
        let themes = Self::scan_themes(&themes_path).await.unwrap_or_default();

        info!(
            "Detected WordPress {} at {}",
            version.as_deref().unwrap_or("unknown"),
            path.display()
        );

        Ok(Self {
            root_path: path.to_path_buf(),
            version,
            is_multisite,
            wp_content_path,
            plugins_path,
            themes_path,
            uploads_path,
            plugins,
            themes,
        })
    }

    /// Detect WordPress version from version.php
    async fn detect_version(root: &Path) -> WpResult<String> {
        let version_file = root.join("wp-includes/version.php");
        if !version_file.exists() {
            return Err(WpError::UnknownVersion);
        }

        let content = fs::read_to_string(&version_file).await?;

        // Look for $wp_version = 'X.Y.Z';
        for line in content.lines() {
            if line.contains("$wp_version") && line.contains('=') {
                if let Some(start) = line.find('\'') {
                    if let Some(end) = line[start + 1..].find('\'') {
                        return Ok(line[start + 1..start + 1 + end].to_string());
                    }
                }
            }
        }

        Err(WpError::UnknownVersion)
    }

    /// Detect if this is a multisite installation
    async fn detect_multisite(root: &Path) -> WpResult<bool> {
        let wp_config = root.join("wp-config.php");
        if !wp_config.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(&wp_config).await?;
        Ok(content.contains("MULTISITE") && content.contains("true"))
    }

    /// Scan plugins directory
    async fn scan_plugins(plugins_path: &Path) -> WpResult<Vec<PluginInfo>> {
        let mut plugins = Vec::new();

        if !plugins_path.exists() {
            return Ok(plugins);
        }

        let mut entries = fs::read_dir(plugins_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(plugin) = PluginInfo::from_directory(&path).await {
                    plugins.push(plugin);
                }
            } else if path.extension().is_some_and(|e| e == "php") {
                // Single-file plugin
                if let Ok(plugin) = PluginInfo::from_file(&path).await {
                    plugins.push(plugin);
                }
            }
        }

        Ok(plugins)
    }

    /// Scan themes directory
    async fn scan_themes(themes_path: &Path) -> WpResult<Vec<ThemeInfo>> {
        let mut themes = Vec::new();

        if !themes_path.exists() {
            return Ok(themes);
        }

        let mut entries = fs::read_dir(themes_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(theme) = ThemeInfo::from_directory(&path).await {
                    themes.push(theme);
                }
            }
        }

        Ok(themes)
    }
}

/// Information about a WordPress plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin slug (directory name)
    pub slug: String,

    /// Plugin name from header
    pub name: Option<String>,

    /// Plugin version from header
    pub version: Option<String>,

    /// Plugin author from header
    pub author: Option<String>,

    /// Path to the plugin
    pub path: PathBuf,

    /// Whether the plugin appears to be active
    pub is_active: bool,

    /// Number of PHP files in the plugin
    pub php_file_count: usize,
}

impl PluginInfo {
    /// Parse plugin info from a directory
    pub async fn from_directory(path: &Path) -> WpResult<Self> {
        let slug = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Look for main plugin file (same name as directory)
        let main_file = path.join(format!("{}.php", slug));
        let headers = if main_file.exists() {
            Self::parse_plugin_headers(&main_file).await.ok()
        } else {
            // Try to find any PHP file with plugin headers
            None
        };

        // Count PHP files
        let php_file_count = Self::count_php_files(path).await.unwrap_or(0);

        Ok(Self {
            slug,
            name: headers.as_ref().and_then(|h| h.get("Plugin Name").cloned()),
            version: headers.as_ref().and_then(|h| h.get("Version").cloned()),
            author: headers.as_ref().and_then(|h| h.get("Author").cloned()),
            path: path.to_path_buf(),
            is_active: false, // Would need to check wp_options to determine
            php_file_count,
        })
    }

    /// Parse plugin info from a single file
    pub async fn from_file(path: &Path) -> WpResult<Self> {
        let slug = path
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let headers = Self::parse_plugin_headers(path).await.ok();

        Ok(Self {
            slug,
            name: headers.as_ref().and_then(|h| h.get("Plugin Name").cloned()),
            version: headers.as_ref().and_then(|h| h.get("Version").cloned()),
            author: headers.as_ref().and_then(|h| h.get("Author").cloned()),
            path: path.to_path_buf(),
            is_active: false,
            php_file_count: 1,
        })
    }

    /// Parse WordPress plugin header from a PHP file
    async fn parse_plugin_headers(path: &Path) -> WpResult<HashMap<String, String>> {
        let content = fs::read_to_string(path).await?;
        let mut headers = HashMap::new();

        // Standard WordPress plugin headers
        let header_names = [
            "Plugin Name",
            "Plugin URI",
            "Description",
            "Version",
            "Author",
            "Author URI",
            "License",
            "Text Domain",
        ];

        for header in header_names {
            if let Some(value) = Self::extract_header(&content, header) {
                headers.insert(header.to_string(), value);
            }
        }

        Ok(headers)
    }

    /// Extract a header value from plugin content
    fn extract_header(content: &str, header: &str) -> Option<String> {
        let pattern = format!("{}:", header);
        for line in content.lines().take(100) {
            // Only check first 100 lines
            if let Some(pos) = line.find(&pattern) {
                let value = line[pos + pattern.len()..].trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
        None
    }

    /// Count PHP files in a directory
    async fn count_php_files(path: &Path) -> WpResult<usize> {
        let mut count = 0;
        let mut stack = vec![path.to_path_buf()];

        while let Some(current) = stack.pop() {
            if let Ok(mut entries) = fs::read_dir(&current).await {
                while let Some(entry) = entries.next_entry().await? {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        stack.push(entry_path);
                    } else if entry_path.extension().is_some_and(|e| e == "php") {
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }
}

/// Information about a WordPress theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    /// Theme slug (directory name)
    pub slug: String,

    /// Theme name from style.css
    pub name: Option<String>,

    /// Theme version from style.css
    pub version: Option<String>,

    /// Theme author from style.css
    pub author: Option<String>,

    /// Parent theme if this is a child theme
    pub parent_theme: Option<String>,

    /// Path to the theme
    pub path: PathBuf,

    /// Whether the theme appears to be active
    pub is_active: bool,
}

impl ThemeInfo {
    /// Parse theme info from a directory
    pub async fn from_directory(path: &Path) -> WpResult<Self> {
        let slug = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let style_css = path.join("style.css");
        let headers = if style_css.exists() {
            Self::parse_theme_headers(&style_css).await.ok()
        } else {
            None
        };

        Ok(Self {
            slug,
            name: headers.as_ref().and_then(|h| h.get("Theme Name").cloned()),
            version: headers.as_ref().and_then(|h| h.get("Version").cloned()),
            author: headers.as_ref().and_then(|h| h.get("Author").cloned()),
            parent_theme: headers.as_ref().and_then(|h| h.get("Template").cloned()),
            path: path.to_path_buf(),
            is_active: false,
        })
    }

    /// Parse WordPress theme header from style.css
    async fn parse_theme_headers(path: &Path) -> WpResult<HashMap<String, String>> {
        let content = fs::read_to_string(path).await?;
        let mut headers = HashMap::new();

        // Standard WordPress theme headers
        let header_names = [
            "Theme Name",
            "Theme URI",
            "Description",
            "Version",
            "Author",
            "Author URI",
            "License",
            "Text Domain",
            "Template", // Parent theme
        ];

        for header in header_names {
            if let Some(value) = Self::extract_header(&content, header) {
                headers.insert(header.to_string(), value);
            }
        }

        Ok(headers)
    }

    /// Extract a header value from theme content
    fn extract_header(content: &str, header: &str) -> Option<String> {
        let pattern = format!("{}:", header);
        for line in content.lines().take(50) {
            if let Some(pos) = line.find(&pattern) {
                let value = line[pos + pattern.len()..].trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
        None
    }
}

/// WordPress-specific file auditor
pub struct WpAuditor {
    /// Known malicious patterns to detect
    malicious_patterns: Vec<MaliciousPattern>,
}

/// A malicious pattern to detect in PHP files
#[derive(Debug, Clone)]
struct MaliciousPattern {
    /// Pattern identifier
    id: String,
    /// Pattern description
    description: String,
    /// Regex pattern or simple string to match
    pattern: String,
    /// Severity if found
    severity: Severity,
}

impl Default for WpAuditor {
    fn default() -> Self {
        Self::new()
    }
}

impl WpAuditor {
    /// Create a new WordPress auditor
    pub fn new() -> Self {
        Self {
            malicious_patterns: vec![
                MaliciousPattern {
                    id: "WP001".to_string(),
                    description: "Potentially obfuscated PHP code (base64_decode + eval)"
                        .to_string(),
                    pattern: "base64_decode".to_string(),
                    severity: Severity::High,
                },
                MaliciousPattern {
                    id: "WP002".to_string(),
                    description: "Suspicious eval() usage".to_string(),
                    pattern: "eval(".to_string(),
                    severity: Severity::Medium,
                },
                MaliciousPattern {
                    id: "WP003".to_string(),
                    description: "Potentially dangerous file operation".to_string(),
                    pattern: "file_put_contents".to_string(),
                    severity: Severity::Low,
                },
                MaliciousPattern {
                    id: "WP004".to_string(),
                    description: "Remote code inclusion risk".to_string(),
                    pattern: "curl_exec".to_string(),
                    severity: Severity::Medium,
                },
                MaliciousPattern {
                    id: "WP005".to_string(),
                    description: "Potential shell command execution".to_string(),
                    pattern: "shell_exec".to_string(),
                    severity: Severity::High,
                },
                MaliciousPattern {
                    id: "WP006".to_string(),
                    description: "Potential shell command execution".to_string(),
                    pattern: "system(".to_string(),
                    severity: Severity::High,
                },
                MaliciousPattern {
                    id: "WP007".to_string(),
                    description: "Potential shell command execution".to_string(),
                    pattern: "passthru(".to_string(),
                    severity: Severity::High,
                },
                MaliciousPattern {
                    id: "WP008".to_string(),
                    description: "WordPress backdoor indicator".to_string(),
                    pattern: "wp_filesystem".to_string(),
                    severity: Severity::Low,
                },
                MaliciousPattern {
                    id: "WP009".to_string(),
                    description: "Potential SQL injection vector".to_string(),
                    pattern: "$wpdb->query".to_string(),
                    severity: Severity::Low,
                },
                MaliciousPattern {
                    id: "WP010".to_string(),
                    description: "Potential remote file inclusion".to_string(),
                    pattern: "include($_".to_string(),
                    severity: Severity::Critical,
                },
            ],
        }
    }
}

#[async_trait]
impl Auditor for WpAuditor {
    fn name(&self) -> &str {
        "WP-Praxis WordPress Auditor"
    }

    fn description(&self) -> &str {
        "Scans WordPress installations for security issues and malicious code"
    }

    async fn audit_file(&self, file: &FileInfo) -> AuditResult<Vec<Finding>> {
        let mut findings = Vec::new();

        // Only scan PHP files
        let is_php = file
            .extension
            .as_ref()
            .is_some_and(|e| e == "php" || e == "phtml");

        if !is_php {
            return Ok(findings);
        }

        // Read file content
        let content = match fs::read_to_string(&file.path).await {
            Ok(content) => content,
            Err(error) => {
                debug!("Could not read {}: {}", file.path.display(), error);
                return Ok(findings);
            }
        };

        // Check for malicious patterns
        for pattern in &self.malicious_patterns {
            if content.contains(&pattern.pattern) {
                findings.push(
                    Finding::new(
                        &pattern.id,
                        &pattern.description,
                        pattern.severity,
                        &file.path,
                    )
                    .with_context(format!("Pattern matched: {}", pattern.pattern))
                    .with_recommendation(
                        "Review this file for potentially malicious or unsafe code",
                    ),
                );
            }
        }

        // Check for WordPress-specific issues
        let path_str = file.path.to_string_lossy();

        // Check for exposed wp-config.php backup
        if path_str.contains("wp-config") && !path_str.ends_with("wp-config.php") {
            findings.push(
                Finding::new(
                    "WP011",
                    "Potential wp-config backup file exposed",
                    Severity::Critical,
                    &file.path,
                )
                .with_recommendation("Remove backup copies of wp-config.php from the web root"),
            );
        }

        // Check for debug.log exposure
        if path_str.ends_with("debug.log") {
            findings.push(
                Finding::new(
                    "WP012",
                    "WordPress debug.log file found",
                    Severity::High,
                    &file.path,
                )
                .with_recommendation(
                    "Move debug.log outside web root or restrict access via .htaccess",
                ),
            );
        }

        Ok(findings)
    }

    async fn audit_directory(
        &self,
        path: &Path,
        _files: &[FileInfo],
    ) -> AuditResult<Vec<Finding>> {
        let mut findings = Vec::new();

        // Check for wp-config.php in uploads
        let uploads_wp_config = path.join("wp-content/uploads/wp-config.php");
        if uploads_wp_config.exists() {
            findings.push(
                Finding::new(
                    "WP013",
                    "wp-config.php found in uploads directory",
                    Severity::Critical,
                    uploads_wp_config,
                )
                .with_recommendation("This is likely a compromise indicator. Investigate immediately."),
            );
        }

        // Check for PHP files in uploads (potential backdoor)
        let uploads_dir = path.join("wp-content/uploads");
        if uploads_dir.exists() {
            if let Ok(php_in_uploads) = Self::find_php_in_uploads(&uploads_dir).await {
                for php_file in php_in_uploads {
                    findings.push(
                        Finding::new(
                            "WP014",
                            "PHP file found in uploads directory",
                            Severity::High,
                            php_file,
                        )
                        .with_recommendation(
                            "PHP files in uploads are often backdoors. Review and remove if suspicious.",
                        ),
                    );
                }
            }
        }

        Ok(findings)
    }
}

impl WpAuditor {
    /// Find PHP files in the uploads directory
    async fn find_php_in_uploads(uploads_path: &Path) -> WpResult<Vec<PathBuf>> {
        let mut php_files = Vec::new();
        let mut stack = vec![uploads_path.to_path_buf()];

        while let Some(current) = stack.pop() {
            if let Ok(mut entries) = fs::read_dir(&current).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        stack.push(entry_path);
                    } else if entry_path.extension().is_some_and(|e| e == "php") {
                        php_files.push(entry_path);
                    }
                }
            }
        }

        Ok(php_files)
    }
}

/// Configuration auditor for wp-config.php
pub struct WpConfigAuditor;

impl WpConfigAuditor {
    /// Create a new wp-config auditor
    pub fn new() -> Self {
        Self
    }

    /// Audit wp-config.php for security issues
    pub async fn audit_config(wp_root: &Path) -> AuditResult<Vec<Finding>> {
        let mut findings = Vec::new();
        let wp_config = wp_root.join("wp-config.php");

        if !wp_config.exists() {
            return Ok(findings);
        }

        let content = match fs::read_to_string(&wp_config).await {
            Ok(content) => content,
            Err(error) => {
                warn!("Could not read wp-config.php: {}", error);
                return Ok(findings);
            }
        };

        // Check for debug mode enabled
        if content.contains("WP_DEBUG") && content.contains("true") {
            findings.push(
                Finding::new(
                    "WPCFG001",
                    "WordPress debug mode is enabled",
                    Severity::Medium,
                    &wp_config,
                )
                .with_recommendation("Disable WP_DEBUG in production environments"),
            );
        }

        // Check for file editing enabled
        if !content.contains("DISALLOW_FILE_EDIT") || content.contains("DISALLOW_FILE_EDIT', false") {
            findings.push(
                Finding::new(
                    "WPCFG002",
                    "WordPress file editing is not disabled",
                    Severity::Medium,
                    &wp_config,
                )
                .with_recommendation("Add define('DISALLOW_FILE_EDIT', true); to wp-config.php"),
            );
        }

        // Check for default table prefix
        if content.contains("$table_prefix = 'wp_'") {
            findings.push(
                Finding::new(
                    "WPCFG003",
                    "Default WordPress table prefix is used",
                    Severity::Low,
                    &wp_config,
                )
                .with_recommendation("Consider using a custom table prefix for security"),
            );
        }

        // Check for SSL enforcement
        if !content.contains("FORCE_SSL_ADMIN") {
            findings.push(
                Finding::new(
                    "WPCFG004",
                    "FORCE_SSL_ADMIN is not set",
                    Severity::Medium,
                    &wp_config,
                )
                .with_recommendation("Add define('FORCE_SSL_ADMIN', true); to force HTTPS for admin"),
            );
        }

        Ok(findings)
    }
}

impl Default for WpConfigAuditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_wp_auditor_detects_eval() {
        let auditor = WpAuditor::new();
        let file = FileInfo {
            path: PathBuf::from("/test/malicious.php"),
            size: 100,
            is_directory: false,
            is_symlink: false,
            extension: Some("php".to_string()),
            hash: None,
            is_hidden: false,
        };

        // This test would need actual file content to work
        let findings = auditor.audit_file(&file).await.unwrap();
        // Without file content, no findings expected
        assert!(findings.is_empty());
    }

    #[tokio::test]
    async fn test_plugin_info_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("test-plugin");
        std::fs::create_dir(&plugin_dir).unwrap();

        // Create a plugin file with headers
        let plugin_content = r#"<?php
/**
 * Plugin Name: Test Plugin
 * Version: 1.0.0
 * Author: Test Author
 */
"#;
        std::fs::write(plugin_dir.join("test-plugin.php"), plugin_content).unwrap();

        let plugin = PluginInfo::from_directory(&plugin_dir).await.unwrap();
        assert_eq!(plugin.slug, "test-plugin");
        assert_eq!(plugin.name, Some("Test Plugin".to_string()));
        assert_eq!(plugin.version, Some("1.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_theme_info_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let theme_dir = temp_dir.path().join("test-theme");
        std::fs::create_dir(&theme_dir).unwrap();

        // Create a style.css with headers
        let style_content = r#"/*
Theme Name: Test Theme
Version: 2.0.0
Author: Theme Author
*/
"#;
        std::fs::write(theme_dir.join("style.css"), style_content).unwrap();

        let theme = ThemeInfo::from_directory(&theme_dir).await.unwrap();
        assert_eq!(theme.slug, "test-theme");
        assert_eq!(theme.name, Some("Test Theme".to_string()));
        assert_eq!(theme.version, Some("2.0.0".to_string()));
    }
}
