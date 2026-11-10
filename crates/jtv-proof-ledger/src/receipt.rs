// SPDX-License-Identifier: MPL-2.0
//! Running a checker and recording what it observed.
//!
//! A receipt is written only when the run succeeds and the input files did
//! not change while it ran. Anything else is an error, never a partial
//! receipt: a receipt that overstates what was checked is worse than none.

use crate::{current_inputs, Prover, Receipt, Records};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

type Result<T> = std::result::Result<T, String>;

fn run(cmd: &mut Command) -> Result<(bool, String)> {
    let out = cmd
        .output()
        .map_err(|e| format!("could not start {cmd:?}: {e}"))?;
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    Ok((out.status.success(), text))
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let (ok, out) = run(Command::new("git").arg("-C").arg(root).args(args))?;
    if !ok {
        return Err(format!("git {args:?} failed: {out}"));
    }
    Ok(out.trim().to_string())
}

fn now_utc() -> Result<String> {
    let (ok, out) = run(Command::new("date").args(["-u", "+%Y-%m-%dT%H:%M:%SZ"]))?;
    if !ok {
        return Err("date failed".into());
    }
    Ok(out.trim().to_string())
}

/// A full 40-character hexadecimal commit hash.
fn is_full_commit(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn commit_info(root: &Path) -> Result<(String, bool)> {
    let head = git(root, &["rev-parse", "HEAD"])?;
    if !is_full_commit(&head) {
        return Err(format!("HEAD is not a full commit hash: {head}"));
    }
    let clean = git(root, &["status", "--porcelain"])?.is_empty();
    Ok((head, clean))
}

/// The standard library as `name@commit`. The commit comes from Git run in the
/// library file's parent directory, or from `AGDA_STDLIB_COMMIT` if Git fails
/// or does not return a full hash. File, name and fallback pin errors propagate.
fn stdlib_pin(lib_file: &Path) -> Result<String> {
    let text = std::fs::read_to_string(lib_file)
        .map_err(|e| format!("cannot read {}: {e}", lib_file.display()))?;
    let name = text
        .lines()
        .find_map(|l| l.strip_prefix("name:"))
        .map(str::trim)
        .ok_or_else(|| format!("no name: line in {}", lib_file.display()))?;
    let dir = lib_file.parent().unwrap_or(Path::new("."));
    let commit = match git(dir, &["rev-parse", "HEAD"]) {
        Ok(c) if is_full_commit(&c) => c,
        _ => {
            let c = std::env::var("AGDA_STDLIB_COMMIT").map_err(|_| {
                "the stdlib is not a git checkout; set AGDA_STDLIB_COMMIT to its pinned commit"
                    .to_string()
            })?;
            if !is_full_commit(&c) {
                return Err(format!("AGDA_STDLIB_COMMIT is not a full commit hash: {c}"));
            }
            c
        }
    };
    Ok(format!("{name}@{commit}"))
}

/// Run `theory/agda/check.sh` and return a receipt; the caller writes it.
///
/// `AGDA_STDLIB` must name the standard library file. The check script may
/// write build outputs and a log. An error is returned for a failed check,
/// missing required output, a non-clean escape-hatch scan or changed inputs,
/// as well as for input, pin, Git, date and process errors.
pub fn agda(root: &Path) -> Result<Receipt> {
    let lib = std::env::var("AGDA_STDLIB")
        .map_err(|_| "set AGDA_STDLIB to the pinned standard-library.agda-lib".to_string())?;
    let stdlib = stdlib_pin(Path::new(&lib))?;
    let before = current_inputs(root, Prover::Agda).map_err(|e| e.to_string())?;
    let (ok, out) = run(&mut Command::new(root.join("theory/agda/check.sh")))?;
    if !ok {
        return Err(format!("check.sh failed; no receipt written:\n{out}"));
    }
    let after = current_inputs(root, Prover::Agda).map_err(|e| e.to_string())?;
    if before != after {
        return Err("the Agda sources changed during the run; no receipt written".into());
    }
    let field = |key: &str| {
        out.lines()
            .find_map(|l| l.strip_prefix(key))
            .map(|v| v.trim().to_string())
            .ok_or_else(|| format!("check.sh output has no `{key}` line"))
    };
    let checker = field("agda:")?;
    let invocation = field("invocation:")?;
    let flags: Vec<String> = invocation
        .split_whitespace()
        .filter(|w| w.starts_with("--") && *w != "--library-file" && *w != "--no-default-libraries")
        .map(str::to_string)
        .collect();
    let mut results = BTreeMap::new();
    for line in out.lines() {
        let mut parts = line.split_whitespace();
        if let (Some(m), Some("TYPECHECKED"), None) = (parts.next(), parts.next(), parts.next()) {
            results.insert(m.replace('/', "."), "typechecked".to_string());
        }
    }
    if results.is_empty() {
        return Err("check.sh reported no typechecked modules".into());
    }
    let escape_scan = if out
        .lines()
        .any(|l| l.trim() == "hole/postulate scan: clean")
    {
        "clean".to_string()
    } else {
        return Err("check.sh did not report a clean escape-hatch scan".into());
    };
    let (base_commit, worktree_clean) = commit_info(root)?;
    Ok(Receipt {
        schema: 1,
        prover: Prover::Agda,
        checker,
        libraries: vec![stdlib],
        invocation: "theory/agda/check.sh (agda <flags> <module>.agda per module)".into(),
        flags,
        base_commit,
        worktree_clean,
        date: now_utc()?,
        inputs: after,
        results,
        axioms: BTreeMap::new(),
        escape_scan,
    })
}

/// Declarations whose axioms include an escape hatch.
fn escape_hatches(axioms: &BTreeMap<String, Vec<String>>) -> Vec<String> {
    axioms
        .iter()
        .filter(|(_, ax)| {
            ax.iter()
                .any(|a| a == "sorryAx" || a == "Lean.ofReduceBool")
        })
        .map(|(name, _)| name.clone())
        .collect()
}

/// Parse one line of `#print axioms` output.
/// Returns `None` for an unrecognised line and sorts any reported axioms.
fn parse_axioms_line(line: &str) -> Option<(String, Vec<String>)> {
    let rest = line.strip_prefix('\'')?;
    let (name, tail) = rest.split_once('\'')?;
    let tail = tail.trim();
    if tail.starts_with("does not depend on any axioms") {
        return Some((name.to_string(), vec![]));
    }
    let list = tail.strip_prefix("depends on axioms:")?.trim();
    let list = list.strip_prefix('[')?.strip_suffix(']')?;
    let mut axioms: Vec<String> = list.split(',').map(|s| s.trim().to_string()).collect();
    axioms.retain(|s| !s.is_empty());
    axioms.sort();
    Some((name.to_string(), axioms))
}

/// Build the Lean project, then ask the kernel for the axioms of every Lean
/// declaration named by the records and return a receipt; the caller writes it.
///
/// The build writes project outputs, and the axiom probe is written under
/// `.lake`; its removal is attempted after the probe returns output. An error
/// is returned if the build or probe fails, a declaration has no axiom report,
/// a report contains `sorryAx` or `Lean.ofReduceBool`, or the inputs change.
/// Input, process, probe-write, Git and date errors also propagate.
pub fn lean(root: &Path, records: &Records) -> Result<Receipt> {
    let proj = root.join("jtv_proofs");
    let before = current_inputs(root, Prover::Lean).map_err(|e| e.to_string())?;
    let (ok, out) = run(Command::new("lake").arg("build").current_dir(&proj))?;
    if !ok {
        return Err(format!("lake build failed; no receipt written:\n{out}"));
    }
    let mut decls: Vec<String> = records
        .theorems
        .iter()
        .filter_map(|t| t.lean.as_ref().and_then(|l| l.decl.clone()))
        .collect();
    decls.sort();
    decls.dedup();
    let mut modules: Vec<String> = before
        .keys()
        .filter_map(|p| p.strip_prefix("jtv_proofs/"))
        .filter_map(|p| p.strip_suffix(".lean"))
        .filter(|m| *m != "lakefile" && !m.contains('/'))
        .map(str::to_string)
        .collect();
    modules.sort();
    let mut src = String::new();
    for m in &modules {
        src.push_str(&format!("import {m}\n"));
    }
    for d in &decls {
        src.push_str(&format!("#print axioms {d}\n"));
    }
    let probe = proj.join(".lake/jtv-ledger-axioms.lean");
    std::fs::write(&probe, src).map_err(|e| e.to_string())?;
    let (ok, out) = run(Command::new("lake")
        .args(["env", "lean"])
        .arg(&probe)
        .current_dir(&proj))?;
    let _ = std::fs::remove_file(&probe);
    if !ok {
        return Err(format!(
            "#print axioms failed (a declaration named in theorems.json may not exist):\n{out}"
        ));
    }
    let mut axioms = BTreeMap::new();
    for line in out.lines() {
        if let Some((name, ax)) = parse_axioms_line(line) {
            axioms.insert(name, ax);
        }
    }
    let missing: Vec<&String> = decls.iter().filter(|d| !axioms.contains_key(*d)).collect();
    if !missing.is_empty() {
        return Err(format!("no axiom report for {missing:?}"));
    }
    // An escape hatch is never recorded as a checked proof: `sorryAx` is an
    // unfinished proof, and `Lean.ofReduceBool` is `native_decide` trusting the
    // compiler. Either one means no receipt at all, not a leg with an axiom.
    let escaped = escape_hatches(&axioms);
    if !escaped.is_empty() {
        return Err(format!(
            "escape hatches found; no receipt written: {escaped:?}"
        ));
    }
    let after = current_inputs(root, Prover::Lean).map_err(|e| e.to_string())?;
    if before != after {
        return Err("the Lean sources changed during the run; no receipt written".into());
    }
    let (ok, version) = run(Command::new("lake")
        .args(["env", "lean", "--version"])
        .current_dir(&proj))?;
    if !ok {
        return Err("lean --version failed".into());
    }
    let (base_commit, worktree_clean) = commit_info(root)?;
    let results = decls
        .iter()
        .map(|d| (d.clone(), "checked".to_string()))
        .collect();
    Ok(Receipt {
        schema: 1,
        prover: Prover::Lean,
        checker: version.trim().to_string(),
        libraries: vec!["Lean core only (no Mathlib; empty lake-manifest packages)".into()],
        invocation: "lake build; lake env lean <#print axioms probe>".into(),
        flags: vec!["autoImplicit=false".into()],
        base_commit,
        worktree_clean,
        date: now_utc()?,
        inputs: after,
        results,
        axioms,
        escape_scan: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::{escape_hatches, is_full_commit, parse_axioms_line};
    use std::collections::BTreeMap;

    #[test]
    fn rejects_escape_hatches() {
        let ax = BTreeMap::from([
            ("ok".to_string(), vec!["propext".to_string()]),
            ("sorried".to_string(), vec!["sorryAx".to_string()]),
            ("native".to_string(), vec!["Lean.ofReduceBool".to_string()]),
        ]);
        assert_eq!(
            escape_hatches(&ax),
            vec!["native".to_string(), "sorried".to_string()]
        );
    }

    #[test]
    fn full_commit_hashes_only() {
        assert!(is_full_commit("2a5a0dec0a3cb5c87468ce101388a513d39dbf10"));
        assert!(!is_full_commit("2a5a0dec"));
        assert!(!is_full_commit("zz5a0dec0a3cb5c87468ce101388a513d39dbf10"));
    }

    #[test]
    fn parses_axiom_lists() {
        assert_eq!(
            parse_axioms_line("'Jtv.Echo.join_comm' does not depend on any axioms"),
            Some(("Jtv.Echo.join_comm".into(), vec![]))
        );
        assert_eq!(
            parse_axioms_line("'rev_composition' depends on axioms: [Quot.sound, propext]"),
            Some((
                "rev_composition".into(),
                vec!["Quot.sound".into(), "propext".into()]
            ))
        );
        assert_eq!(parse_axioms_line("Build completed successfully."), None);
    }
}
