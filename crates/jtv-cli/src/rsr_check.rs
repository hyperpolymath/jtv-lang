// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2025 Julia the Viper Contributors
//
// RSR (Rhodium Standard Repository) Compliance Checker

use colored::*;
use std::fs;
use std::path::Path;

pub struct RsrChecker {
    pub score: u32,
    pub max_score: u32,
    pub passed: Vec<String>,
    pub failed: Vec<String>,
    pub warnings: Vec<String>,
}

impl RsrChecker {
    pub fn new() -> Self {
        RsrChecker {
            score: 0,
            max_score: 0,
            passed: Vec::new(),
            failed: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn check_all(&mut self) {
        println!("{}", "RSR Compliance Check".cyan().bold());
        println!("{}", "=".repeat(60));
        println!();

        self.check_documentation();
        self.check_well_known();
        self.check_build_system();
        self.check_licensing();
        self.check_governance();
        self.check_security();
        self.check_ci_cd();
        self.check_code_quality();
        self.check_offline_first();
        self.check_tpcf();

        self.print_summary();
    }

    fn check_file(&mut self, path: &str, category: &str) {
        self.max_score += 1;
        if Path::new(path).exists() {
            self.score += 1;
            self.passed.push(format!("{}: {}", category, path));
        } else {
            self.failed
                .push(format!("{}: {} (missing)", category, path));
        }
    }

    fn check_documentation(&mut self) {
        println!("{}", "📚 Documentation".yellow().bold());

        self.check_file("README_JTV.md", "README");
        self.check_file("CONTRIBUTING.md", "Contributing");
        self.check_file("CODE_OF_CONDUCT.md", "Code of Conduct");
        self.check_file("CHANGELOG.md", "Changelog");
        self.check_file("MAINTAINERS.md", "Maintainers");

        // Check documentation quality
        if let Ok(readme) = fs::read_to_string("README_JTV.md") {
            if readme.len() > 1000 {
                self.score += 1;
                self.max_score += 1;
                self.passed
                    .push("README: Comprehensive (>1000 chars)".to_string());
            } else {
                self.max_score += 1;
                self.warnings
                    .push("README: Could be more comprehensive".to_string());
            }
        }

        println!();
    }

    fn check_well_known(&mut self) {
        println!("{}", "🔍 .well-known/ Directory".yellow().bold());

        self.check_file(".well-known/security.txt", "Security.txt (RFC 9116)");
        self.check_file(".well-known/ai.txt", "AI Training Policy");
        self.check_file(".well-known/humans.txt", "Humans.txt");

        // Check security.txt validity
        if let Ok(security_txt) = fs::read_to_string(".well-known/security.txt") {
            if security_txt.contains("Contact:") && security_txt.contains("Expires:") {
                self.score += 1;
                self.max_score += 1;
                self.passed
                    .push("security.txt: RFC 9116 compliant".to_string());
            } else {
                self.max_score += 1;
                self.warnings
                    .push("security.txt: May not be fully RFC 9116 compliant".to_string());
            }
        }

        println!();
    }

    fn check_build_system(&mut self) {
        println!("{}", "🔨 Build System".yellow().bold());

        self.check_file("Justfile", "Justfile");
        self.check_file("flake.nix", "Nix Flake");
        self.check_file("Cargo.toml", "Cargo.toml");

        // Check Justfile recipes
        if let Ok(justfile) = fs::read_to_string("Justfile") {
            let recipe_count = justfile.matches(":\n").count();
            if recipe_count >= 10 {
                self.score += 1;
                self.max_score += 1;
                self.passed
                    .push(format!("Justfile: {} recipes (≥10)", recipe_count));
            } else {
                self.max_score += 1;
                self.warnings
                    .push(format!("Justfile: Only {} recipes (<10)", recipe_count));
            }
        }

        println!();
    }

    fn check_licensing(&mut self) {
        println!("{}", "⚖️  Licensing".yellow().bold());

        self.check_file("LICENSE", "Primary License");
        self.check_file("LICENSE-MIT", "MIT License");
        self.check_file("LICENSE-PALIMPSEST", "Palimpsest License");
        self.check_file("LICENSING.md", "Licensing Guide");

        // Check dual licensing
        if Path::new("LICENSE-MIT").exists() && Path::new("LICENSE-PALIMPSEST").exists() {
            self.score += 1;
            self.max_score += 1;
            self.passed
                .push("Dual licensing: MIT + Palimpsest".to_string());
        } else {
            self.max_score += 1;
            self.warnings
                .push("Dual licensing not fully implemented".to_string());
        }

        println!();
    }

    fn check_governance(&mut self) {
        println!("{}", "👥 Governance".yellow().bold());

        self.check_file("MAINTAINERS.md", "Maintainers");
        self.check_file("TPCF.md", "TPCF Perimeters");
        self.check_file("CODE_OF_CONDUCT.md", "Code of Conduct");

        // Check TPCF implementation
        if let Ok(tpcf) = fs::read_to_string("TPCF.md") {
            if tpcf.contains("Perimeter 1")
                && tpcf.contains("Perimeter 2")
                && tpcf.contains("Perimeter 3")
            {
                self.score += 1;
                self.max_score += 1;
                self.passed
                    .push("TPCF: All 3 perimeters defined".to_string());
            }
        } else {
            self.max_score += 1;
        }

        println!();
    }

    fn check_security(&mut self) {
        println!("{}", "🔒 Security".yellow().bold());

        self.check_file("SECURITY.md", "Security Policy");

        // Check for security features
        if let Ok(security) = fs::read_to_string("SECURITY.md") {
            let checks = vec![
                ("Reporting process", "Reporting Process"),
                ("Response timeline", "Response Time"),
                ("Vulnerability classes", "Vulnerability Classes"),
                ("Security guarantees", "Security Guarantees"),
            ];

            for (keyword, label) in checks {
                self.max_score += 1;
                if security.to_lowercase().contains(keyword) {
                    self.score += 1;
                    self.passed.push(format!("SECURITY.md: {}", label));
                }
            }
        }

        println!();
    }

    fn check_ci_cd(&mut self) {
        println!("{}", "🚀 CI/CD".yellow().bold());

        self.check_file(".gitlab-ci.yml", "GitLab CI");

        // Alternative CI systems
        if !Path::new(".gitlab-ci.yml").exists() {
            if Path::new(".github/workflows").exists() {
                self.score += 1;
                self.passed.push("GitHub Actions configured".to_string());
            }
        }

        // Check CI stages
        if let Ok(ci_config) = fs::read_to_string(".gitlab-ci.yml") {
            let stages = ["check", "test", "build", "deploy"];
            let mut found_stages = 0;
            for stage in stages {
                if ci_config.contains(stage) {
                    found_stages += 1;
                }
            }

            self.max_score += 1;
            if found_stages >= 3 {
                self.score += 1;
                self.passed
                    .push(format!("CI/CD: {} stages configured", found_stages));
            } else {
                self.warnings
                    .push(format!("CI/CD: Only {} stages found", found_stages));
            }
        }

        println!();
    }

    fn check_code_quality(&mut self) {
        println!("{}", "✨ Code Quality".yellow().bold());

        // Check for tests
        let test_paths = vec![
            "packages/jtzig/tests",
            "packages/jtzig/benches",
            "tools/cli/tests",
        ];

        for test_path in test_paths {
            if Path::new(test_path).exists() {
                self.score += 1;
                self.max_score += 1;
                self.passed.push(format!("Tests: {} exists", test_path));
            } else {
                self.max_score += 1;
            }
        }

        // Check for no unsafe code (scan Rust files)
        if let Ok(entries) = fs::read_dir("packages/jtzig/src") {
            let mut has_unsafe = false;
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if content.contains("unsafe {") {
                        has_unsafe = true;
                        break;
                    }
                }
            }

            self.max_score += 1;
            if !has_unsafe {
                self.score += 1;
                self.passed
                    .push("Memory safety: No unsafe blocks in core".to_string());
            } else {
                self.warnings
                    .push("Memory safety: Unsafe blocks detected".to_string());
            }
        }

        println!();
    }

    fn check_offline_first(&mut self) {
        println!("{}", "📡 Offline-First".yellow().bold());

        // Check Cargo.toml for network dependencies
        if let Ok(cargo_toml) = fs::read_to_string("packages/jtzig/Cargo.toml") {
            let network_keywords = vec!["reqwest", "hyper", "tokio", "async"];
            let mut has_network = false;

            for keyword in network_keywords {
                if cargo_toml.contains(keyword) {
                    has_network = true;
                    break;
                }
            }

            self.max_score += 1;
            if !has_network {
                self.score += 1;
                self.passed
                    .push("Offline-first: No network dependencies in core".to_string());
            } else {
                self.warnings
                    .push("Offline-first: Network dependencies detected".to_string());
            }
        }

        println!();
    }

    fn check_tpcf(&mut self) {
        println!("{}", "🛡️  TPCF Implementation".yellow().bold());

        self.check_file("TPCF.md", "TPCF Documentation");

        // Check branch protection (would need GitHub API in real implementation)
        self.max_score += 1;
        self.warnings
            .push("TPCF: Branch protection (manual verification required)".to_string());

        println!();
    }

    fn print_summary(&self) {
        println!("{}", "=".repeat(60));
        println!();
        println!("{}", "Summary".cyan().bold());
        println!("{}", "=".repeat(60));

        let percentage = if self.max_score > 0 {
            (self.score as f64 / self.max_score as f64 * 100.0) as u32
        } else {
            0
        };

        let grade = match percentage {
            90..=100 => ("🥇 Platinum", "green"),
            75..=89 => ("🥈 Gold", "yellow"),
            60..=74 => ("🥉 Silver", "blue"),
            50..=59 => ("Bronze", "white"),
            _ => ("Needs Work", "red"),
        };

        println!(
            "Score: {}/{} ({}%)",
            self.score.to_string().green().bold(),
            self.max_score,
            percentage.to_string().bold()
        );
        println!("Grade: {}", grade.0.color(grade.1).bold());
        println!();

        if !self.passed.is_empty() {
            println!("{}", "✅ Passed:".green().bold());
            for item in &self.passed {
                println!("  ✓ {}", item);
            }
            println!();
        }

        if !self.failed.is_empty() {
            println!("{}", "❌ Failed:".red().bold());
            for item in &self.failed {
                println!("  ✗ {}", item);
            }
            println!();
        }

        if !self.warnings.is_empty() {
            println!("{}", "⚠️  Warnings:".yellow().bold());
            for item in &self.warnings {
                println!("  ⚠ {}", item);
            }
            println!();
        }

        println!("{}", "=".repeat(60));
        println!();

        if percentage >= 75 {
            println!(
                "{}",
                "🎉 Congratulations! This repository meets RSR standards."
                    .green()
                    .bold()
            );
        } else if percentage >= 50 {
            println!(
                "{}",
                "⚙️  Good progress! A few improvements needed for full compliance."
                    .yellow()
                    .bold()
            );
        } else {
            println!(
                "{}",
                "📝 Significant work needed to meet RSR standards."
                    .red()
                    .bold()
            );
        }

        println!();
        println!("For more information:");
        println!("  - RSR Documentation: https://rhodium-standard.org");
        println!("  - TPCF Guide: https://rhodium-standard.org/tpcf");
        println!();
    }
}

impl Default for RsrChecker {
    fn default() -> Self {
        Self::new()
    }
}
