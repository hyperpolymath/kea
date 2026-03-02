;; SPDX-License-Identifier: PMPL-1.0-or-later
;; Kea-Mandible Testing Report
;; Machine-readable Guile Scheme format
;; Generated: 2025-12-29

(testing-report
  (metadata
    (project "kea-mandible")
    (version "0.1.0")
    (report-date "2025-12-29")
    (report-generator "Claude Code (Automated)")
    (schema-version "1.0.0"))

  (summary
    (overall-status 'pass)
    (build-status 'success)
    (test-status 'success)
    (lint-status 'clean))

  (build-results
    (debug-build
      (status 'success)
      (duration-seconds 165)
      (warnings 0)
      (errors 0))
    (release-build
      (status 'success)
      (duration-seconds 267)
      (warnings 0)
      (errors 0))
    (incremental-build
      (status 'success)
      (duration-seconds 25)
      (warnings 0)
      (errors 0)))

  (test-results
    (total-tests 13)
    (passed 13)
    (failed 0)
    (ignored 0)
    (test-suites
      ((name "kea-beak")
       (tests 5)
       (passed 5)
       (test-cases
         ("test_audit_empty_directory" 'pass)
         ("test_audit_with_files" 'pass)
         ("test_finding_builder" 'pass)
         ("test_security_auditor" 'pass)
         ("test_severity_display" 'pass)))
      ((name "wp-praxis")
       (tests 3)
       (passed 3)
       (test-cases
         ("test_theme_info_parsing" 'pass)
         ("test_wp_auditor_detects_eval" 'pass)
         ("test_plugin_info_parsing" 'pass)))
      ((name "slop-gate")
       (tests 5)
       (passed 5)
       (test-cases
         ("test_bloat_report" 'pass)
         ("test_format_bytes" 'pass)
         ("test_slop_gate_detects_backup_files" 'pass)
         ("test_slop_gate_detects_source_maps" 'pass)
         ("test_slop_gate_detects_temp_files" 'pass)))))

  (lint-results
    (clippy
      (status 'clean)
      (warnings 0)
      (errors 0)
      (fixes-applied
        ("derivable_impls" "HashAlgorithm default derived")
        ("for_kv_map" "Use .values() iterator")
        ("option_as_ref_deref" "Use .as_deref()")
        ("manual_strip" "Use strip_prefix/strip_suffix")
        ("unnecessary_map_or" "Use is_some_and")
        ("ptr_arg" "Use &Path instead of &PathBuf"))))

  (components
    ((name "kea-beak")
     (description "General-purpose filesystem auditor")
     (type 'library)
     (source-file "crates/kea-beak/src/lib.rs")
     (lines-of-code 450)
     (features
       ("High-throughput file scanning")
       ("Configurable hash calculation")
       ("Extensible auditor plugin system")
       ("Structured finding reporting"))
     (performance
       (measured-throughput-files-per-sec 7918)))

    ((name "wp-praxis")
     (description "WordPress environment auditor")
     (type 'library)
     (source-file "crates/wp-praxis/src/lib.rs")
     (lines-of-code 700)
     (features
       ("WordPress installation detection")
       ("Plugin and theme enumeration")
       ("Malicious code pattern detection")
       ("wp-config.php security analysis"))
     (detection-patterns
       ((id "WP001") (severity 'high) (description "Obfuscated PHP code"))
       ((id "WP002") (severity 'medium) (description "Suspicious eval usage"))
       ((id "WP005") (severity 'high) (description "shell_exec detection"))
       ((id "WP006") (severity 'high) (description "system() detection"))
       ((id "WP007") (severity 'high) (description "passthru() detection"))
       ((id "WP010") (severity 'critical) (description "Remote file inclusion"))
       ((id "WP011") (severity 'critical) (description "Exposed wp-config backup"))
       ((id "WP012") (severity 'high) (description "debug.log exposure"))))

    ((name "slop-gate")
     (description "Runtime bloat rejection filter")
     (type 'library)
     (source-file "crates/slop-gate/src/lib.rs")
     (lines-of-code 650)
     (features
       ("Duplicate file detection")
       ("Development artifact identification")
       ("Temporary file detection")
       ("Large binary flagging")
       ("Source map detection")
       ("Test file in production detection"))
     (bloat-categories
       ('duplicate "Files with identical content")
       ('dev-artifact "Development-only files")
       ('orphaned-dependency "Unused dependencies")
       ('temporary-file "Temp and cache files")
       ('large-binary "Oversized binary files")
       ('backup-file "Unnecessary backups")
       ('source-map "Source maps in production")
       ('bundled-docs "Documentation in node_modules")
       ('test-file "Test files in production")
       ('example-file "Example files in dependencies")))

    ((name "kea-mandible")
     (description "Main CLI binary")
     (type 'binary)
     (source-file "crates/kea-mandible/src/main.rs")
     (lines-of-code 450)
     (commands
       ((name "pry")
        (description "Deep audit of a target path")
        (options
          ("--target" "Target path to audit" 'required)
          ("--depth" "Maximum depth to traverse" 'optional)
          ("--follow-symlinks" "Follow symbolic links" 'flag)
          ("--include-hidden" "Include hidden files" 'flag)
          ("--hashes" "Calculate file hashes" 'flag)
          ("--hash-algorithm" "Hash algorithm (sha256, blake3)" 'optional)))
       ((name "word-press")
        (description "WordPress-specific audit")
        (options
          ("--path" "Path to WordPress installation" 'required)
          ("--audit-config" "Audit wp-config.php settings" 'flag)))
       ((name "slop")
        (description "Detect and report bloat")
        (options
          ("--target" "Target path to analyze" 'required)
          ("--duplicates" "Detect duplicate files" 'flag)
          ("--large-binary-mb" "Large binary threshold in MB" 'optional)))
       ((name "version")
        (description "Show version information")
        (options)))))

  (dependencies
    ((name "tokio") (version "1.43") (purpose "Async runtime"))
    ((name "clap") (version "4.5") (purpose "CLI argument parsing"))
    ((name "walkdir") (version "2.5") (purpose "Directory traversal"))
    ((name "blake3") (version "1.5") (purpose "Fast hashing"))
    ((name "sha2") (version "0.10") (purpose "SHA-256 hashing"))
    ((name "serde") (version "1.0") (purpose "Serialization"))
    ((name "serde_json") (version "1.0") (purpose "JSON serialization"))
    ((name "tracing") (version "0.1") (purpose "Structured logging"))
    ((name "tracing-subscriber") (version "0.3") (purpose "Log subscriber"))
    ((name "ignore") (version "0.4") (purpose "Gitignore-style filtering"))
    ((name "globset") (version "0.4") (purpose "Glob pattern matching"))
    ((name "async-trait") (version "0.1") (purpose "Async trait support"))
    ((name "thiserror") (version "2.0") (purpose "Error handling"))
    ((name "anyhow") (version "1.0") (purpose "Error context")))

  (known-limitations
    ((id "LIM001")
     (description "Unix timestamps instead of ISO 8601")
     (impact 'low)
     (mitigation "Add chrono crate for proper datetime formatting"))
    ((id "LIM002")
     (description "WordPress detection requires file read access")
     (impact 'medium)
     (mitigation "Pattern matching works only on readable files"))
    ((id "LIM003")
     (description "Duplicate detection memory intensive for large sets")
     (impact 'medium)
     (mitigation "Consider streaming hash comparison"))
    ((id "LIM004")
     (description "No Cap'n Proto output format")
     (impact 'medium)
     (mitigation "Implement capnp crate integration")))

  (recommendations
    ((priority 'high)
     (items
       ("Add integration tests with sample WordPress installations")
       ("Implement Cap'n Proto serialization for Kea-Call format")))
    ((priority 'medium)
     (items
       ("Add file watcher mode for continuous monitoring")
       ("Implement parallel directory scanning")
       ("Add network configuration auditing")))
    ((priority 'low)
     (items
       ("Add progress bar for long-running scans")
       ("Implement config file support")
       ("Add HTML report output format"))))

  (conclusion
    (status 'success)
    (summary "Kea-Mandible project successfully implemented with all core components functional and tested. The codebase passes all unit tests, has zero clippy warnings, and builds successfully in both debug and release modes.")))
