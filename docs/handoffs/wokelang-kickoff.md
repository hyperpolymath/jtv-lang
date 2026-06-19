<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->
<!-- Prepared 2026-06-19 as a handoff prompt for a future Claude session on wokelang. -->

# Kickoff prompt — wokelang

Paste the block below to the next Claude session that has the **wokelang** repo
(`hyperpolymath/wokelang`, one of the nextgen-languages portfolio) in scope.

---

You are continuing work on **wokelang**, a language in the hyperpolymath /
nextgen-languages portfolio. Work on the designated `claude/...` feature branch;
commit with clear messages; push; open a **draft** PR when a rung lands.

**First, orient yourself (read before changing anything):**
1. `.claude/CLAUDE.md` and any root `CLAUDE.md` — project-specific rules override defaults.
2. `.machine_readable/6a2/STATE.a2ml` — current state, milestones, blockers, next-actions.
3. `README.adoc` / `README.md` and `docs/` — purpose and design.
4. `contractiles/` — `Mustfile` (invariants that MUST hold), `Intentfile` (declared intent), `Trustfile` (provenance).
5. The build entrypoint — `Justfile` (run `just --list`), `guix.scm` (primary) / `flake.nix` (fallback).

**Then produce a status before diving in:** a short table grouped into **MUST /
INTEND / WISH** (mirror `hyperpolymath/jtv:docs/ecosystem-status.adoc`), grounded
in `STATE.a2ml` + contractiles — and propose the single smallest useful next rung.
Confirm direction if it's ambiguous or architecturally significant.

**Hyperpolymath standards to hold to:**
- **Language policy:** AffineScript (primary app code) · Rust (systems/CLI/WASM) ·
  Deno (runtime/pkg) · Gleam (BEAM services) · Julia (batch/data) · Guile Scheme
  (state/meta `.a2ml`) · Nickel (config) · OCaml/Ada where specified.
  **Banned:** TypeScript, ReScript (→ AffineScript), Node/npm/bun/pnpm/yarn (→ Deno),
  Go (→ Rust), Python (→ Julia/Rust), Java/Kotlin/Swift/RN/Flutter (→ Rust/Tauri/Dioxus).
- **Licensing:** MPL-2.0 (code) + CC-BY-SA-4.0 (docs), full per-file SPDX headers.
  The PMPL-1.0 / Palimpsest *license* is retired across the estate (keep "Palimpsest"
  only as philosophy/name, never as an SPDX id or a separate `LICENSE`).
- **Hygiene (RSR):** `Justfile` not `Makefile`; `Containerfile` not `Dockerfile`;
  no hardcoded developer paths; HTTPS only; SHA256+ (no MD5/SHA1); no hardcoded
  secrets; SHA-pinned deps; Guix primary / Nix fallback.

**Working discipline (mirror the JtV cadence):**
- Every change ends with the relevant build/test command **run**, with captured output.
- Headline results pinned (tests/CI green) — no overclaiming; if something is
  partial or representational, say so explicitly.
- Update `.machine_readable/6a2/STATE.a2ml` (session-history + next-actions) and the
  contractiles when a rung lands.
- One concern per PR; draft PRs; never push to `main`.

Start by reading the orientation files above and reporting wokelang's current
MUST/INTEND/WISH status + your proposed first rung.

---
