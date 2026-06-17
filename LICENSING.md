<!--
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# Licensing

Julia the Viper is **dual-licensed by artifact type**:

| Artifact | License | SPDX |
|----------|---------|------|
| **Source code** (`.rs`, `.lean`, `.idr`, `.yml`, `.toml`, `.sh`, `.a2ml`, `.scm`, `.ncl`, `.pest`, `.ebnf`, `Justfile`, build/CI config) | Mozilla Public License 2.0 | `MPL-2.0` |
| **Documentation & prose** (`.adoc`, `.md`, wiki, specs, ADRs, tutorials) | Creative Commons Attribution-ShareAlike 4.0 | `CC-BY-SA-4.0` |

This supersedes the earlier PMPL-1.0 / Palimpsest, `MIT OR GPL-3.0-or-later OR Palimpsest`, and `MPL-2.0-or-later` arrangements (see *History*).

## Per-file SPDX headers

Every file declares its license in an SPDX header:

- Code: `// SPDX-License-Identifier: MPL-2.0` (or `# …` for shell/YAML/TOML).
- Docs (Markdown): `<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->`
- Docs (AsciiDoc): `// SPDX-License-Identifier: CC-BY-SA-4.0`

Full license texts live in `LICENSES/` (REUSE layout); the repository-root `LICENSE` is the MPL-2.0 code license.

## Why this split

- **MPL-2.0 for code** — file-level copyleft that stays permissive enough to embed JtV as an "aspect" in other languages (the AOLD goal), while keeping modifications open.
- **CC-BY-SA-4.0 for docs** — the natural license for prose and specs: share and adapt with attribution, share-alike.

## Third-party dependencies

| Crate | License |
|-------|---------|
| `pest`, `pest_derive` | MIT/Apache-2.0 |
| `num-rational`, `num-complex`, `num-traits` | MIT/Apache-2.0 |
| `thiserror`, `serde`, `serde_json` | MIT/Apache-2.0 |
| `wasm-bindgen`, `clap`, `criterion` | MIT/Apache-2.0 |
| `colored` | MPL-2.0 |

All are compatible with MPL-2.0.

## History

Earlier revisions experimented with a bespoke **Palimpsest (PMPL-1.0)** license and, in places, an `MIT OR GPL-3.0-or-later OR Palimpsest` tri-license and `MPL-2.0-or-later`. As of 2026-06 the project standardises on **MPL-2.0 (code) + CC-BY-SA-4.0 (docs)**; `PALIMPSEST.adoc` and the PMPL badges are retired. Already-released versions remain under whatever terms they shipped with.

## Questions

Open a GitHub discussion tagged `licensing`.
