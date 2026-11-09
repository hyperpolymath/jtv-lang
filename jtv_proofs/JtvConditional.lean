-- SPDX-License-Identifier: MPL-2.0
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath)
/-
  JtV — the reversible `if`, and what goes wrong without a residue (Lean 4).

  The Lean counterpart of `theory/agda/src/Jtv/Conditional.agda`, written so the
  Agda statements have a Lean statement to be compared against (PROOF-POLICY
  §2.2). It models one guarded addition, `if g { x += v }`, and two ways to
  reverse it:

  * `reverseIfRecheck` re-evaluates the guard on the reversed state. This is
    the tokenless path of `crates/jtv-core/src/reversible.rs`
    (`execute_inverted_stmt`), audit finding F3.
  * `reverseIfRecorded` replays the branch the forward pass actually took, the
    `RecordedOp::If { condition_was_true }` discipline at runtime.

  `tokenless_conditional_not_identity` exhibits a state at which the first is
  not an identity. `reversible_if_with_residue` proves the second is, for any
  guard, pointwise in every variable.

  Scope: the step adds a *known* `v`, so the recorded branch bit is the whole
  residue. An overwriting branch (`x = 5`) needs the saved old value — the
  `Snapshot` residue — and is not covered here, exactly as in the Agda module.

  Correspondence notes (recorded per theorem in `theory/agda/README.adoc`):
  variables are `String` here and `ℕ` in Agda, the guard tests variable `"x"`
  here and variable `zero` there, and Lean writes `σ x - v` where Agda writes
  `σ x + (- v)`.
-/
import JtvCore

namespace Jtv.Conditional

/-- The guard of the counterexample: "x is still zero". -/
def guard (σ : State) : Bool := σ "x" == 0

/-- Add `v` to `x` when the guard holds in the input state; otherwise return the
    state unchanged. -/
def forwardIf (g : State → Bool) (x : String) (v : Int) (σ : State) : State :=
  if g σ then σ[x ↦ σ x + v] else σ

/-- Tokenless reversal: re-evaluate the guard on the reversed state. -/
def reverseIfRecheck (g : State → Bool) (x : String) (v : Int) (σ : State) : State :=
  if g σ then σ[x ↦ σ x - v] else σ

/-- Residue reversal: replay the branch that was taken; the `Bool` is the residue. -/
def reverseIfRecorded : Bool → String → Int → State → State
  | true,  x, v, σ => σ[x ↦ σ x - v]
  | false, _, _, σ => σ

/-- **The tokenless strategy is not an identity.** At `σ ≡ 0` the forward step
    takes the branch and sets `x` to `1`; the guard is then false, so the
    re-checking reversal skips the undo. -/
theorem tokenless_conditional_not_identity :
    ∃ σ : State, reverseIfRecheck guard "x" 1 (forwardIf guard "x" 1 σ) ≠ σ := by
  refine ⟨fun _ => 0, ?_⟩
  intro h
  have hx := congrFun h "x"
  simp [reverseIfRecheck, forwardIf, guard, State.update] at hx

/-- **The repair is correct.** Recording the branch taken and replaying it makes
    forward-then-reverse the identity at every variable, for an arbitrary guard. -/
theorem reversible_if_with_residue (g : State → Bool) (x : String) (v : Int)
    (σ : State) (y : String) :
    reverseIfRecorded (g σ) x v (forwardIf g x v σ) y = σ y := by
  unfold forwardIf
  cases g σ with
  | false => rfl
  | true =>
    simp only [reverseIfRecorded, if_true]
    by_cases hy : y = x
    · subst hy; simp [State.update]
    · simp [State.update, hy]

end Jtv.Conditional
