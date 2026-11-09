// SPDX-License-Identifier: MPL-2.0
//! `jtv-proof-ledger` — derive JtV's Proof Witness Grades from receipts.
//!
//! ```text
//! jtv-proof-ledger receipt agda   # run theory/agda/check.sh, write verification/receipts/agda.json
//! jtv-proof-ledger receipt lean   # lake build + #print axioms, write verification/receipts/lean.json
//! jtv-proof-ledger render         # write verification/ledger/PWG.adoc
//! jtv-proof-ledger check          # fail if PWG.adoc differs from a fresh render
//! ```
//!
//! `receipt agda` needs `AGDA` (optional) and `AGDA_STDLIB`; `receipt lean`
//! needs `lake` on `PATH`. `render` and `check` need no prover at all: they
//! read the committed receipts and hash the files on disk.

use jtv_proof_ledger::render::{render, Inputs};
use jtv_proof_ledger::{current_inputs, receipt, Receipt, Records};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const RECORDS: &str = "verification/ledger/theorems.json";
const RENDERED: &str = "verification/ledger/PWG.adoc";

fn root() -> PathBuf {
    std::env::var_os("JTV_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .expect("repository root")
        })
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn read_receipt(root: &Path, prover: &str) -> Result<Option<Receipt>, String> {
    let p = root.join(format!("verification/receipts/{prover}.json"));
    if p.exists() {
        read_json(&p).map(Some)
    } else {
        Ok(None)
    }
}

/// Write JSON with sorted keys and no insignificant whitespace (the JCS shape
/// for the ASCII strings and integers a receipt holds), plus a final newline.
fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let v = serde_json::to_value(value).map_err(|e| e.to_string())?;
    let text = serde_json::to_string(&v).map_err(|e| e.to_string())?;
    std::fs::write(path, text + "\n").map_err(|e| format!("{}: {e}", path.display()))
}

fn rendered(root: &Path) -> Result<String, String> {
    let records: Records = read_json(&root.join(RECORDS))?;
    let agda = read_receipt(root, "agda")?;
    let lean = read_receipt(root, "lean")?;
    let agda_now = current_inputs(root, "agda").map_err(|e| e.to_string())?;
    let lean_now = current_inputs(root, "lean").map_err(|e| e.to_string())?;
    Ok(render(&Inputs {
        records: &records,
        agda: agda.as_ref(),
        agda_now: &agda_now,
        lean: lean.as_ref(),
        lean_now: &lean_now,
    }))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = root();
    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["receipt", "agda"] => receipt::agda(&root)
            .and_then(|r| write_json(&root.join("verification/receipts/agda.json"), &r)),
        ["receipt", "lean"] => read_json::<Records>(&root.join(RECORDS))
            .and_then(|recs| receipt::lean(&root, &recs))
            .and_then(|r| write_json(&root.join("verification/receipts/lean.json"), &r)),
        ["render"] => rendered(&root)
            .and_then(|text| std::fs::write(root.join(RENDERED), text).map_err(|e| e.to_string())),
        ["check"] => rendered(&root).and_then(|fresh| {
            let committed = std::fs::read_to_string(root.join(RENDERED)).unwrap_or_default();
            if committed == fresh {
                println!("{RENDERED} matches the receipts and records");
                Ok(())
            } else {
                Err(format!(
                    "{RENDERED} does not match a fresh render: a proof file, a receipt or \
                     theorems.json changed. Re-run the receipt for the prover whose sources \
                     changed, then `jtv-proof-ledger render`."
                ))
            }
        }),
        _ => Err("usage: jtv-proof-ledger receipt agda|lean | render | check".into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}
