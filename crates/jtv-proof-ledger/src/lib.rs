// SPDX-License-Identifier: MPL-2.0
//! Proof Witness Grades for JtV, derived from receipts.
//!
//! `hyperpolymath/standards` → `3-practice/PROOF-POLICY.adoc` §8 rules that a
//! Proof Witness Grade (PWG) is *computed* from checker receipts and the §2.2
//! correspondence records, never written by hand. This crate is that
//! computation for JtV's theory theorems:
//!
//! * [`Records`] — `verification/ledger/theorems.json`: one record per theory
//!   theorem, naming its Agda source and the Lean and Rocq legs, with each
//!   leg's route and §2.2 correspondence state.
//! * [`Receipt`] — `verification/receipts/{agda,lean}.json`: what a pinned
//!   checker observed, with a sha256 digest of every input file.
//! * [`derive`] — the grade rules. A receipt counts only while its digests
//!   match the files on disk (§8.4, "regression drops the grade"). The two
//!   provers go stale independently: an Agda edit leaves no Agda leg, so every
//!   theorem is `ungraded`; a Lean edit stops the Lean legs counting, and
//!   theorems with a current Agda receipt stay `PWG-3`.
//!
//! The crate never awards `PWG-1`: that needs per-commit receipts from a
//! headless, isolated checker for every witness, which this repository does
//! not produce yet. It never treats a leg as counting unless its record says
//! `RECONCILED`, and only the owner moves a record to that state.

pub mod receipt;
pub mod render;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The flags PROOF-POLICY §2.1 requires on every Agda theory entrypoint.
pub const CANONICAL_AGDA_FLAGS: [&str; 3] = ["--safe", "--without-K", "--cubical-compatible"];

/// §2.2 correspondence states. Only `RECONCILED` lets a recheck leg count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Correspondence {
    #[serde(rename = "AGDA-PENDING")]
    AgdaPending,
    #[serde(rename = "BRIDGE-PENDING")]
    BridgePending,
    #[serde(rename = "RECONCILED")]
    Reconciled,
    #[serde(rename = "COUNTEREXAMPLE")]
    Counterexample,
    #[serde(rename = "QUARANTINED")]
    Quarantined,
}

/// §2 routes for a recheck leg.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Route {
    Transported,
    ReProved,
    Waived,
    Unavailable,
}

/// The Agda source of a theory theorem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgdaSource {
    pub module: String,
    pub name: String,
}

/// A recheck leg (Lean or Rocq) as the owner has recorded it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leg {
    pub route: Route,
    pub state: Correspondence,
    /// The declaration in the target prover, when one exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decl: Option<String>,
    /// How the target statement relates to the Agda statement, and what was
    /// compared. Required before a leg can be `RECONCILED`.
    #[serde(default)]
    pub comparison: String,
    /// Residual assumptions, e.g. a different variable encoding.
    #[serde(default)]
    pub assumptions: Vec<String>,
    /// Whether the owner has ruled the leg's axioms admissible under
    /// TRUSTED-BASE-REDUCTION-POLICY disposition (c). A leg resting on axioms
    /// counts only when this is true.
    #[serde(default)]
    pub axioms_admissible: bool,
}

/// One theory theorem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theorem {
    pub id: String,
    pub statement: String,
    /// `None` for a theory statement that so far has only target-prover
    /// evidence: it is `ungraded` (§8.1).
    #[serde(default)]
    pub agda: Option<AgdaSource>,
    #[serde(default)]
    pub lean: Option<Leg>,
    #[serde(default)]
    pub rocq: Option<Leg>,
}

/// `verification/ledger/theorems.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Records {
    pub schema: u32,
    pub theorems: Vec<Theorem>,
}

/// The provers a receipt can come from. A closed set: a typo in a caller is a
/// compile error, not a run-time one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Prover {
    Agda,
    Lean,
}

impl Prover {
    /// The lowercase name used in receipt file names and in the JSON.
    pub fn name(self) -> &'static str {
        match self {
            Prover::Agda => "agda",
            Prover::Lean => "lean",
        }
    }
}

/// What a checker run observed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Receipt {
    pub schema: u32,
    pub prover: Prover,
    /// The checker's own version string.
    pub checker: String,
    /// Library pins, e.g. `standard-library-2.4@2a5a0dec…`.
    pub libraries: Vec<String>,
    pub invocation: String,
    /// Flags actually in force.
    pub flags: Vec<String>,
    /// The commit the inputs were checked out from, and whether the worktree
    /// held uncommitted changes. Currency is decided by `inputs`, not by this.
    pub base_commit: String,
    pub worktree_clean: bool,
    pub date: String,
    /// Repository-relative path → sha256 of every input file.
    pub inputs: BTreeMap<String, String>,
    /// Agda: module → `typechecked`. Lean: declaration → `checked`.
    pub results: BTreeMap<String, String>,
    /// Lean: declaration → axioms the kernel reports.
    #[serde(default)]
    pub axioms: BTreeMap<String, Vec<String>>,
    /// Agda: the escape-hatch scan outcome (`clean`).
    #[serde(default)]
    pub escape_scan: String,
}

/// sha256 of a file, as lowercase hex.
pub fn file_digest(path: &Path) -> std::io::Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(hex(&Sha256::digest(&bytes)))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The input files a prover's receipt must cover, found on disk now.
pub fn current_inputs(root: &Path, prover: Prover) -> std::io::Result<BTreeMap<String, String>> {
    let mut files: Vec<PathBuf> = Vec::new();
    match prover {
        Prover::Agda => {
            collect(&root.join("theory/agda/src"), "agda", &mut files)?;
            files.push(root.join("theory/agda/jtv-theory.agda-lib"));
            files.push(root.join("theory/agda/check.sh"));
        }
        Prover::Lean => {
            collect(&root.join("jtv_proofs"), "lean", &mut files)?;
            files.push(root.join("jtv_proofs/lean-toolchain"));
        }
    }
    let mut out = BTreeMap::new();
    for f in files {
        let rel = f
            .strip_prefix(root)
            .unwrap_or(&f)
            .to_string_lossy()
            .replace('\\', "/");
        out.insert(rel, file_digest(&f)?);
    }
    Ok(out)
}

/// Source files with the given extension, recursively. Build directories are
/// skipped: `.lake` and `_build` hold outputs, not inputs.
fn collect(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(std::fs::DirEntry::path);
    for e in entries {
        let p = e.path();
        let name = e.file_name();
        let name = name.to_string_lossy();
        if p.is_dir() {
            if name.starts_with('.') || name == "_build" {
                continue;
            }
            collect(&p, ext, out)?;
        } else if p.extension().is_some_and(|x| x == ext) {
            out.push(p);
        }
    }
    Ok(())
}

/// Why a receipt does or does not describe the files on disk now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Currency {
    Current,
    Missing,
    /// The digests differ: these files changed, appeared or disappeared.
    Stale(Vec<String>),
}

/// Compare a receipt's recorded inputs with the files on disk.
pub fn currency(receipt: Option<&Receipt>, now: &BTreeMap<String, String>) -> Currency {
    let Some(r) = receipt else {
        return Currency::Missing;
    };
    let mut changed: Vec<String> = Vec::new();
    for (path, digest) in now {
        if r.inputs.get(path) != Some(digest) {
            changed.push(path.clone());
        }
    }
    for path in r.inputs.keys() {
        if !now.contains_key(path) {
            changed.push(path.clone());
        }
    }
    if changed.is_empty() {
        Currency::Current
    } else {
        changed.sort();
        changed.dedup();
        Currency::Stale(changed)
    }
}

/// §5 tokens, as the ledger records them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Proven,
    /// Proven, resting on the listed axioms.
    ProvenWithAxioms(Vec<String>),
    Open,
}

impl Token {
    pub fn label(&self) -> String {
        match self {
            Token::Proven => "`proven`".into(),
            Token::ProvenWithAxioms(a) => format!("`proven-with-postulates` ({})", a.join(", ")),
            Token::Open => "`open`".into(),
        }
    }
}

/// The grade of one theorem and the reasons behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graded {
    pub pwg: Pwg,
    pub agda: Token,
    pub lean: Option<(Token, bool)>,
    pub rocq: Option<(Token, bool)>,
}

/// Proof Witness Grades. Lower is stronger; `PWG-1` is never derived here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pwg {
    Ungraded,
    Three,
    TwoI,
    TwoIi,
}

impl Pwg {
    pub fn label(self) -> &'static str {
        match self {
            Pwg::Ungraded => "ungraded",
            Pwg::Three => "PWG-3",
            Pwg::TwoI => "PWG-2i",
            Pwg::TwoIi => "PWG-2ii",
        }
    }
}

/// The receipts a derivation reads, with their currency already decided.
pub struct Evidence<'a> {
    pub agda: Option<&'a Receipt>,
    pub agda_current: bool,
    pub lean: Option<&'a Receipt>,
    pub lean_current: bool,
}

/// The Agda leg's §5 token: `proven` only from a current receipt that ran the
/// canonical flags, typechecked the module, and found no escape hatch.
fn agda_token(src: &AgdaSource, ev: &Evidence) -> Token {
    let Some(r) = ev.agda.filter(|_| ev.agda_current) else {
        return Token::Open;
    };
    let flags_ok = CANONICAL_AGDA_FLAGS
        .iter()
        .all(|f| r.flags.iter().any(|g| g == f));
    let checked = r.results.get(&src.module).map(String::as_str) == Some("typechecked");
    if flags_ok && checked && r.escape_scan == "clean" {
        Token::Proven
    } else {
        Token::Open
    }
}

/// The Lean leg's token, and whether the leg counts towards a grade.
fn lean_leg(leg: &Leg, ev: &Evidence) -> (Token, bool) {
    let found = leg.decl.as_ref().and_then(|d| {
        let r = ev.lean.filter(|_| ev.lean_current)?;
        (r.results.get(d).map(String::as_str) == Some("checked"))
            .then(|| r.axioms.get(d).cloned().unwrap_or_default())
    });
    let token = match found {
        None => Token::Open,
        Some(ax) if ax.is_empty() => Token::Proven,
        Some(ax) => Token::ProvenWithAxioms(ax),
    };
    let counts = leg.state == Correspondence::Reconciled
        && matches!(leg.route, Route::Transported | Route::ReProved)
        && match &token {
            Token::Proven => true,
            Token::ProvenWithAxioms(_) => leg.axioms_admissible,
            Token::Open => false,
        };
    (token, counts)
}

/// Apply PROOF-POLICY §8.2 to one theorem.
pub fn derive(t: &Theorem, ev: &Evidence) -> Graded {
    let agda = t.agda.as_ref().map_or(Token::Open, |s| agda_token(s, ev));
    let lean = t.lean.as_ref().map(|l| lean_leg(l, ev));
    // No Rocq development or receipt exists, so a Rocq leg never counts.
    let rocq = t.rocq.as_ref().map(|_| (Token::Open, false));
    let pwg = if !matches!(agda, Token::Proven | Token::ProvenWithAxioms(_)) {
        Pwg::Ungraded
    } else {
        let legs = [lean.as_ref(), rocq.as_ref()]
            .iter()
            .filter(|l| l.is_some_and(|(_, c)| *c))
            .count();
        match legs {
            0 => Pwg::Three,
            1 => Pwg::TwoI,
            _ => Pwg::TwoIi,
        }
    };
    Graded {
        pwg,
        agda,
        lean,
        rocq,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn receipt(prover: Prover) -> Receipt {
        Receipt {
            schema: 1,
            prover,
            checker: "test".into(),
            libraries: vec![],
            invocation: "test".into(),
            flags: CANONICAL_AGDA_FLAGS.iter().map(|s| s.to_string()).collect(),
            base_commit: "0".repeat(40),
            worktree_clean: true,
            date: "2026-11-09T00:00:00Z".into(),
            inputs: BTreeMap::from([("a".into(), "1".into())]),
            results: BTreeMap::from([
                ("Jtv.M".into(), "typechecked".into()),
                ("thm".into(), "checked".into()),
            ]),
            axioms: BTreeMap::new(),
            escape_scan: "clean".into(),
        }
    }

    fn theorem(state: Correspondence) -> Theorem {
        Theorem {
            id: "t".into(),
            statement: "s".into(),
            agda: Some(AgdaSource {
                module: "Jtv.M".into(),
                name: "t".into(),
            }),
            lean: Some(Leg {
                route: Route::ReProved,
                state,
                decl: Some("thm".into()),
                comparison: "same".into(),
                assumptions: vec![],
                axioms_admissible: false,
            }),
            rocq: None,
        }
    }

    fn ev<'a>(a: &'a Receipt, l: &'a Receipt, ac: bool, lc: bool) -> Evidence<'a> {
        Evidence {
            agda: Some(a),
            agda_current: ac,
            lean: Some(l),
            lean_current: lc,
        }
    }

    #[test]
    fn agda_alone_is_pwg3() {
        let (a, l) = (receipt(Prover::Agda), receipt(Prover::Lean));
        let g = derive(
            &theorem(Correspondence::BridgePending),
            &ev(&a, &l, true, true),
        );
        assert_eq!(g.pwg, Pwg::Three);
        assert_eq!(g.lean, Some((Token::Proven, false)));
    }

    #[test]
    fn reconciled_lean_leg_gives_pwg2i() {
        let (a, l) = (receipt(Prover::Agda), receipt(Prover::Lean));
        let g = derive(
            &theorem(Correspondence::Reconciled),
            &ev(&a, &l, true, true),
        );
        assert_eq!(g.pwg, Pwg::TwoI);
    }

    #[test]
    fn stale_agda_receipt_drops_to_ungraded() {
        let (a, l) = (receipt(Prover::Agda), receipt(Prover::Lean));
        let g = derive(
            &theorem(Correspondence::Reconciled),
            &ev(&a, &l, false, true),
        );
        assert_eq!(g.pwg, Pwg::Ungraded);
        assert_eq!(g.agda, Token::Open);
    }

    #[test]
    fn stale_lean_receipt_stops_the_leg_counting() {
        let (a, l) = (receipt(Prover::Agda), receipt(Prover::Lean));
        let g = derive(
            &theorem(Correspondence::Reconciled),
            &ev(&a, &l, true, false),
        );
        assert_eq!(g.pwg, Pwg::Three);
    }

    #[test]
    fn non_canonical_flags_do_not_count() {
        let (mut a, l) = (receipt(Prover::Agda), receipt(Prover::Lean));
        a.flags = vec!["--safe".into()];
        let g = derive(
            &theorem(Correspondence::Reconciled),
            &ev(&a, &l, true, true),
        );
        assert_eq!(g.pwg, Pwg::Ungraded);
    }

    #[test]
    fn dirty_escape_scan_does_not_count() {
        let (mut a, l) = (receipt(Prover::Agda), receipt(Prover::Lean));
        a.escape_scan = "postulate found".into();
        let g = derive(
            &theorem(Correspondence::Reconciled),
            &ev(&a, &l, true, true),
        );
        assert_eq!(g.pwg, Pwg::Ungraded);
    }

    #[test]
    fn axioms_need_an_admissibility_ruling() {
        let (a, mut l) = (receipt(Prover::Agda), receipt(Prover::Lean));
        l.axioms.insert("thm".into(), vec!["propext".into()]);
        let mut t = theorem(Correspondence::Reconciled);
        let g = derive(&t, &ev(&a, &l, true, true));
        assert_eq!(g.pwg, Pwg::Three);
        assert_eq!(
            g.lean,
            Some((Token::ProvenWithAxioms(vec!["propext".into()]), false))
        );
        t.lean.as_mut().unwrap().axioms_admissible = true;
        assert_eq!(derive(&t, &ev(&a, &l, true, true)).pwg, Pwg::TwoI);
    }

    #[test]
    fn unavailable_or_waived_legs_never_count() {
        let (a, l) = (receipt(Prover::Agda), receipt(Prover::Lean));
        for route in [Route::Unavailable, Route::Waived] {
            let mut t = theorem(Correspondence::Reconciled);
            t.lean.as_mut().unwrap().route = route;
            assert_eq!(derive(&t, &ev(&a, &l, true, true)).pwg, Pwg::Three);
        }
    }

    #[test]
    fn missing_lean_declaration_is_open() {
        let (a, l) = (receipt(Prover::Agda), receipt(Prover::Lean));
        let mut t = theorem(Correspondence::Reconciled);
        t.lean.as_mut().unwrap().decl = Some("absent".into());
        let g = derive(&t, &ev(&a, &l, true, true));
        assert_eq!(g.lean, Some((Token::Open, false)));
        assert_eq!(g.pwg, Pwg::Three);
    }

    #[test]
    fn rocq_never_counts_without_a_development() {
        let (a, l) = (receipt(Prover::Agda), receipt(Prover::Lean));
        let mut t = theorem(Correspondence::Reconciled);
        t.rocq = t.lean.clone();
        assert_eq!(derive(&t, &ev(&a, &l, true, true)).pwg, Pwg::TwoI);
    }

    #[test]
    fn no_agda_source_is_ungraded() {
        let (a, l) = (receipt(Prover::Agda), receipt(Prover::Lean));
        let mut t = theorem(Correspondence::Reconciled);
        t.agda = None;
        assert_eq!(derive(&t, &ev(&a, &l, true, true)).pwg, Pwg::Ungraded);
    }

    #[test]
    fn currency_detects_changed_added_and_removed_files() {
        let mut r = receipt(Prover::Agda);
        r.inputs = BTreeMap::from([("a".into(), "1".into()), ("b".into(), "2".into())]);
        let same = r.inputs.clone();
        assert_eq!(currency(Some(&r), &same), Currency::Current);
        let changed = BTreeMap::from([("a".into(), "9".into()), ("b".into(), "2".into())]);
        assert_eq!(
            currency(Some(&r), &changed),
            Currency::Stale(vec!["a".into()])
        );
        let added = BTreeMap::from([
            ("a".into(), "1".into()),
            ("b".into(), "2".into()),
            ("c".into(), "3".into()),
        ]);
        assert_eq!(
            currency(Some(&r), &added),
            Currency::Stale(vec!["c".into()])
        );
        let removed = BTreeMap::from([("a".into(), "1".into())]);
        assert_eq!(
            currency(Some(&r), &removed),
            Currency::Stale(vec!["b".into()])
        );
        assert_eq!(currency(None, &same), Currency::Missing);
    }
}
