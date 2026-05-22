// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Julia the Viper - Conformance Suite Runner
// Loads .jtv files from conformance/valid/ and conformance/invalid/
// and asserts that valid files parse successfully and invalid files
// produce errors (either parse errors OR semantic errors like purity violations).

use jtv_core::parse_program;
use jtv_core::purity::PurityChecker;
use std::path::PathBuf;

/// Find the conformance directory relative to the workspace root.
fn conformance_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/jtv-core -> workspace root
    manifest_dir.join("../../conformance")
}

/// Read all .jtv files from a directory.
fn collect_jtv_files(dir: &std::path::Path) -> Vec<PathBuf> {
    if !dir.exists() {
        return vec![];
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("Failed to read conformance directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("jtv") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    files.sort();
    files
}

#[test]
fn conformance_valid_files_parse() {
    let valid_dir = conformance_dir().join("valid");
    let files = collect_jtv_files(&valid_dir);

    assert!(
        !files.is_empty(),
        "No valid conformance files found in {:?}",
        valid_dir
    );

    let mut failures = Vec::new();

    for file in &files {
        let source = std::fs::read_to_string(file).expect("Failed to read conformance file");
        match parse_program(&source) {
            Ok(_) => {} // Expected: valid files should parse
            Err(e) => {
                failures.push(format!(
                    "  FAIL: {} -- {}",
                    file.file_name().unwrap().to_str().unwrap(),
                    e
                ));
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "Valid conformance files that failed to parse:\n{}",
            failures.join("\n")
        );
    }
}

#[test]
fn conformance_invalid_files_reject() {
    let invalid_dir = conformance_dir().join("invalid");
    let files = collect_jtv_files(&invalid_dir);

    assert!(
        !files.is_empty(),
        "No invalid conformance files found in {:?}",
        invalid_dir
    );

    let mut failures = Vec::new();

    for file in &files {
        let source = std::fs::read_to_string(file).expect("Failed to read conformance file");

        // An invalid file should fail either at parse or semantic check
        let rejected = match parse_program(&source) {
            Err(_) => true, // Parse error — correctly rejected
            Ok(program) => {
                // Parsed successfully — check if it fails semantic analysis
                // (purity/totality violations, etc.)
                let mut checker = PurityChecker::new();
                checker.check_program(&program).is_err()
            }
        };

        if !rejected {
            failures.push(format!(
                "  FAIL: {} -- passed both parsing and semantic checks but should have been rejected",
                file.file_name().unwrap().to_str().unwrap()
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "Invalid conformance files that incorrectly passed:\n{}",
            failures.join("\n")
        );
    }
}
