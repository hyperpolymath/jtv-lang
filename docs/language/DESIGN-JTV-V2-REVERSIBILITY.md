<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk> -->

# JTV v2 Reversibility: Canonical Design

Date: 2026-04-12

## Status

Accepted — canonical model for JTV v2 (Gamma). See also the designed fork
with 007's Phase 1 implementation documented at the end of this file.

---

## The Philosophical Commitment

JTV v2's central claim is: **subtraction is not a primitive.** It is the
inverse of addition, accessed via reversal. This is not a convenience — it
is the foundational identity of the Gamma stage of JTV's evolution.

`x - v` in JTV v2 does not exist as a grammar production. Instead:

```
reverse { x += v }
```

This is not approximate. It is not "semantically equivalent in the simple
case." It IS subtraction, because subtraction IS the reversal of addition.
The arithmetic identity is preserved at the operation level, not recovered
by a snapshot.

This commitment places JTV v2 in the tradition of Janus (Lutz & Derby 1982,
Yokoyama & Glück 2007) — per-operation reversal, step-by-step invertibility,
genuine connection to Landauer's principle and Bennett's theorem on reversible
computation.

---

## The Three Constructs

### `reversible { }`

Marks a block whose operations are reversible. Maintains a reversal log:

```
R = [(op₁, inv(op₁)), (op₂, inv(op₂)), ...]
```

Each operation inside records its inverse at execution time. The inverses
are:

| Operation | Inverse |
|-----------|---------|
| `x += v` | `x -= v` (i.e. `reverse { x += v }`) |
| `append(xs, v)` | `remove_last(xs)` |
| `send_local(h, v)` | handle reverts via snapshot of pre-send state |

### `reverse { }`

Applies the reversal log in reverse order. This is step-by-step inversion —
not snapshot restore. The log is walked backwards, each inverse applied in
sequence. The programmer observes intermediate states being undone.

### `irreversible { }`

Explicitly marks operations with no defined inverse: I/O, cross-agent sends,
external API calls. Attempting to `reverse` an `irreversible` block is a
static error.

---

## The Linear Token (Shared with 007)

Both JTV v2 and 007 use the linear `ReversalToken` mechanism for pairing.
Entering `reversible { }` produces a linear value `tok` of type
`ReversalToken<S>`. The token must be consumed by `reverse tok` or
`abandon tok` (explicit commit). Linearity guarantees exactly one
`reverse`/`abandon` per token — no double-reversal, no lost log.

```
reversible {
  x += amount
  append(log, entry)
} -> tok            -- tok : ReversalToken<{ x: Int, log: List<Entry> }>

reverse tok         -- replays: remove_last(log), then x -= amount
-- OR
abandon tok         -- commits: discards log, operations stand
```

In JTV v2, the token's type parameter `S` records the *types* of captured
variables whose inverses are logged — not their pre-operation values (that
is the snapshot approach; JTV v2 carries the operation log instead).

### Structural sugar

```
reversible { ... } reverse { ... }
-- desugars to: reversible { ... } -> _tok ; reverse _tok
```

---

## Branch Interaction

At `branch` join points, path-sensitive typing applies:

- Token on **all** arms → `ReversalToken<S>` (must consume)
- Token on **some** arms → `Option<ReversalToken<S>>` (auto-promoted; `match` forces handling)
- Token on **no** arms → nothing in scope

The `Option<ReversalToken<S>>` type at asymmetric joins IS the signal.
`match` forces exhaustive `Some`/`None` handling. No separate lint needed.

---

## The ExternalHandle Boundary

`send` inside a `reversible` block targeting an `ExternalHandle` (crossing
an agent boundary) is a **static error** unless inside `irreversible { }`.

This enforces that JTV v2's reversibility is **local reversibility**. True
distributed reversibility (saga patterns, two-phase commit) is implemented
at the agent protocol level, not via `reverse`. This is correct: distributed
reversibility is a distributed systems problem.

---

## What JTV v2 Does NOT Do

JTV v2 does **not** use whole-block snapshot restore as its reversal
mechanism. This is the key distinction from 007's Phase 1 implementation.

- **Snapshot restore** (007 Phase 1): captures pre-block `@state` values,
  restores them on `reverse`. Semantically correct at block granularity.
  Not per-operation. Not at the Landauer limit.

- **Operation log + replay** (JTV v2): records `(op, inv(op))` pairs as
  operations execute, replays inverses in reverse order on `reverse`.
  Per-operation. Philosophically aligned with the subtraction-as-inverse claim.
  Closer to the Landauer limit (each operation's energy cost can in principle
  be recouped).

---

## The Designed Fork with 007

This divergence from 007's Phase 1 is **deliberate and documented** as a
designed experiment. See:

- `007/docs/session-2026-04-12-jtv-v2-reversibility-design.adoc` — 007's
  five closed design decisions and Phase 1/Phase 2 distinction
- `nextgen-languages/docs/design/jtv-007-reversibility-fork.adoc` — the
  portfolio-level experiment design

### Shared foundations (both approaches)

- Linear `ReversalToken` as pairing mechanism
- `Option<ReversalToken>` at branch joins (path-sensitive typing)
- `ExternalHandle` static error for agent boundary sends
- `irreversible { }` escape hatch
- `reversible { } reverse { }` structural sugar

### What differs

| | JTV v2 | 007 Phase 1 | 007 Phase 2 |
|--|--------|-------------|-------------|
| Token carries | Operation log | `@state` field values (snapshot) | Operation log (ref T) |
| `reverse tok` does | Replay inverse ops in order | Restore snapshot | Replay inverse ops |
| Subtraction model | `reverse { x += v }` = true inverse | State restore predating `+=` | `reverse ref_x += v` = true inverse |
| Landauer proximity | Close | Distant (snapshot has memory cost) | Close |

### Convergence point

007 Phase 2 (`ref T` + fine-grained log) converges toward JTV v2's
per-operation model. At that point the experiment concludes:

- If snapshot proves sufficient for 95% of agent programming use cases,
  JTV v2 may consider a two-tier model (snapshot for simple cases, log for
  complex ones).
- If per-operation inversion proves essential in 007 Phase 2, the Phase 1
  snapshot approach is confirmed as a stepping stone only.

Neither outcome compromises the other. The shared token foundation ensures
findings transfer immediately in either direction.
