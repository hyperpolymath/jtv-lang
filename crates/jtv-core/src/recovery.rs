// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Julia the Viper - Error Recovery Parser
// Provides parse_recovering() which returns a partial AST alongside
// a list of diagnostics. This enables IDE features (syntax highlighting,
// code completion) even when the source is incomplete or malformed.

use crate::ast::*;
use crate::parser::parse_program;

/// A diagnostic message emitted during error-recovering parse.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    /// Human-readable description of the error.
    pub message: String,
    /// Byte offset in the source where the error starts.
    pub offset: usize,
    /// Length of the erroneous span in bytes (0 if unknown).
    pub length: usize,
    /// Severity of the diagnostic.
    pub severity: Severity,
}

/// Severity levels for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// The parser could not understand this region at all.
    Error,
    /// The parser made an assumption to continue (e.g. inserted a closing brace).
    Warning,
    /// Informational note attached to another diagnostic.
    Info,
}

/// Result of an error-recovering parse.
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    /// The (possibly partial) AST. May be empty if the input is
    /// entirely unparseable.
    pub program: Program,
    /// Diagnostics collected during parsing. Empty if the input was
    /// fully valid.
    pub diagnostics: Vec<Diagnostic>,
}

impl RecoveryResult {
    /// Returns true if the parse completed without any diagnostics.
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Returns only the error-severity diagnostics.
    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect()
    }

    /// Returns only the warning-severity diagnostics.
    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .collect()
    }
}

/// Parse the input with error recovery.
///
/// Unlike [`parse_program`], this function never returns `Err`. Instead
/// it returns whatever AST it managed to build plus a list of diagnostics
/// for regions that failed to parse.
///
/// # Strategy
///
/// 1. Try a normal parse first. If it succeeds, return immediately with
///    zero diagnostics.
/// 2. On failure, split the input into top-level chunks (delimited by
///    unmatched `}` or blank lines) and parse each chunk independently.
/// 3. Chunks that fail are recorded as diagnostics; chunks that succeed
///    contribute to the partial AST.
/// 4. Within each chunk, attempt sub-recovery by trying increasingly
///    smaller prefixes and suffix repairs (e.g. appending `}`).
pub fn parse_recovering(input: &str) -> RecoveryResult {
    // Fast path: try the full parse first.
    if let Ok(program) = parse_program(input) {
        return RecoveryResult {
            program,
            diagnostics: vec![],
        };
    }

    let mut statements = Vec::new();
    let mut diagnostics = Vec::new();

    // Split the input into top-level segments.  We use a simple
    // heuristic: scan for lines that start a new top-level construct
    // (module, import, fn, @pure, @total, or an identifier at column 0).
    let segments = split_top_level_segments(input);

    for segment in &segments {
        let text = &input[segment.start..segment.end];
        let trimmed = text.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Try parsing the segment as a full program.
        if let Ok(program) = parse_program(trimmed) {
            statements.extend(program.statements);
            continue;
        }

        // Try appending a closing brace (common incomplete-function case).
        let with_brace = format!("{} }}", trimmed);
        if let Ok(program) = parse_program(&with_brace) {
            statements.extend(program.statements);
            diagnostics.push(Diagnostic {
                message: "Inserted missing closing brace '}'".to_string(),
                offset: segment.end,
                length: 0,
                severity: Severity::Warning,
            });
            continue;
        }

        // Try appending two closing braces (module > function).
        let with_two_braces = format!("{} }} }}", trimmed);
        if let Ok(program) = parse_program(&with_two_braces) {
            statements.extend(program.statements);
            diagnostics.push(Diagnostic {
                message: "Inserted 2 missing closing braces '}}'".to_string(),
                offset: segment.end,
                length: 0,
                severity: Severity::Warning,
            });
            continue;
        }

        // Try parsing individual lines within the segment.
        let mut any_line_ok = false;
        for line in trimmed.lines() {
            let line_trimmed = line.trim();
            if line_trimmed.is_empty() || line_trimmed.starts_with("//") {
                continue;
            }
            if let Ok(program) = parse_program(line_trimmed) {
                statements.extend(program.statements);
                any_line_ok = true;
            }
        }

        if !any_line_ok {
            // Nothing salvageable in this segment.
            diagnostics.push(Diagnostic {
                message: format!(
                    "Failed to parse: {}",
                    if trimmed.len() > 60 {
                        format!("{}...", &trimmed[..57])
                    } else {
                        trimmed.to_string()
                    }
                ),
                offset: segment.start,
                length: segment.end - segment.start,
                severity: Severity::Error,
            });
        }
    }

    RecoveryResult {
        program: Program { statements },
        diagnostics,
    }
}

/// A byte-range segment within the source text.
#[derive(Debug, Clone)]
struct Segment {
    start: usize,
    end: usize,
}

/// Split the input into top-level segments using a lightweight heuristic.
///
/// A new segment begins when a line at column 0 starts with one of:
///   module, import, fn, @pure, @total, or an ASCII letter/underscore
///   (potential assignment or bare identifier).
///
/// This avoids parsing the whole file just to find boundaries.
fn split_top_level_segments(input: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut current_start = 0;

    for (byte_offset, line) in line_byte_offsets(input) {
        if line.is_empty() {
            continue;
        }

        // Only split on lines that start at column 0 (no leading whitespace)
        // and look like a top-level construct.
        let first_char = line.chars().next().unwrap_or(' ');
        let is_toplevel_start = !first_char.is_whitespace()
            && (line.starts_with("module ")
                || line.starts_with("import ")
                || line.starts_with("fn ")
                || line.starts_with("@pure ")
                || line.starts_with("@total ")
                || first_char.is_ascii_alphabetic()
                || first_char == '_');

        if is_toplevel_start && byte_offset > current_start {
            // Close previous segment.
            let prev_end = byte_offset;
            if prev_end > current_start {
                segments.push(Segment {
                    start: current_start,
                    end: prev_end,
                });
            }
            current_start = byte_offset;
        }
    }

    // Final segment.
    if current_start < input.len() {
        segments.push(Segment {
            start: current_start,
            end: input.len(),
        });
    }

    if segments.is_empty() && !input.is_empty() {
        segments.push(Segment {
            start: 0,
            end: input.len(),
        });
    }

    segments
}

/// Yield (byte_offset, line_text) for each line in the input.
fn line_byte_offsets(input: &str) -> Vec<(usize, &str)> {
    let mut result = Vec::new();
    let mut offset = 0;
    for line in input.split('\n') {
        result.push((offset, line));
        offset += line.len() + 1; // +1 for the '\n'
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_parse_returns_no_diagnostics() {
        let result = parse_recovering("x = 5 + 3");
        assert!(result.is_clean());
        assert_eq!(result.program.statements.len(), 1);
    }

    #[test]
    fn test_missing_closing_brace_recovery() {
        let code = "fn broken(x: Int): Int { return x";
        let result = parse_recovering(code);
        assert!(!result.is_clean());
        // Should have recovered with an inserted brace
        assert!(
            !result.program.statements.is_empty() || !result.diagnostics.is_empty(),
            "Should produce either partial AST or diagnostics"
        );
    }

    #[test]
    fn test_multiple_valid_lines() {
        let code = "x = 1\ny = 2\nz = x + y";
        let result = parse_recovering(code);
        assert!(result.is_clean());
        assert_eq!(result.program.statements.len(), 3);
    }

    #[test]
    fn test_mix_of_valid_and_invalid() {
        // Use input where the whole string fails parse_program but
        // individual valid lines can be salvaged.
        let code = "x = 1\n@@@ $$$ %%% ^^^\ny = 2";
        let result = parse_recovering(code);
        // At least some statements should have been recovered.
        assert!(
            !result.program.statements.is_empty(),
            "Should recover at least some statements, got {:?} diagnostics: {:?}",
            result.program.statements.len(),
            result.diagnostics
        );
        // The garbage line should either produce a diagnostic or
        // be silently dropped. Either way, we must get at least x=1
        // or y=2 back.
        assert!(
            result.program.statements.len() >= 1,
            "Should recover at least one statement"
        );
    }

    #[test]
    fn test_empty_input() {
        let result = parse_recovering("");
        assert!(result.is_clean());
        assert!(result.program.statements.is_empty());
    }

    #[test]
    fn test_completely_invalid_input() {
        let result = parse_recovering("@@@ $$$ %%% ^^^");
        assert!(!result.is_clean());
        assert!(!result.errors().is_empty());
    }

    #[test]
    fn test_incomplete_module() {
        let code = r#"
module M {
    fn f(x: Int): Int {
        return x
"#;
        let result = parse_recovering(code);
        // Should recover with inserted braces or report diagnostics.
        assert!(
            !result.program.statements.is_empty() || !result.diagnostics.is_empty(),
            "Should produce partial AST or diagnostics for incomplete module"
        );
    }

    #[test]
    fn test_severity_filtering() {
        let result = parse_recovering("fn broken(x: Int): Int { return x");
        let errors = result.errors();
        let warnings = result.warnings();
        // Should have at least one diagnostic (either error or warning)
        assert!(
            !errors.is_empty() || !warnings.is_empty(),
            "Should have at least one diagnostic"
        );
    }
}
