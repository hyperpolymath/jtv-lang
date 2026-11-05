{-# OPTIONS --cubical-compatible --safe #-}

------------------------------------------------------------------------
-- Jtv.Conditional — the reversible `if`, and what goes wrong without a
-- residue, in Agda.
--
-- This module mechanises finding F3 of
-- docs/reports/audit/audit-2026-11-01-jtv-v2-independence.adoc §8.1:
-- the *tokenless* path (`reverse { if g { x += v } }`) re-evaluates the
-- guard on the reversed state (`crates/jtv-core/src/reversible.rs`
-- `execute_inverted_stmt`), and that is not an identity in general.
-- Janus avoids this by requiring the guard to hold in the reverse
-- direction; JtV has no such check.
--
-- The point of proving it rather than asserting it: the failure is not a
-- coding slip but a semantic gap, so the model has to *exhibit* a state
-- at which forward-then-reverse differs. That is what
-- `tokenless-conditional-not-identity` below does.
--
-- The repair is proved, not merely modelled (`reversible-if-with-residue`):
-- record which branch was taken, exactly as `RecordedOp.If
-- { condition_was_true }` does at runtime, and the round trip *is* the
-- identity — for any guard, not only one that stays true.
--
-- SCOPE of the repair theorem — read it before citing it. `forward-if` adds
-- a *known* `v`, so the recorded bit is the whole residue: undoing `x += v`
-- needs nothing but the branch decision and `v`. For an overwriting
-- assignment (`x = 5`, the form a reversible-`if` branch actually parses —
-- see `crates/jtv-core/src/grammar.pest:48`, where the branch body is a
-- plain control block) the branch bit is *not* sufficient: the old value of
-- `x` is gone, so no branch record can restore it. That case needs the
-- saved value — the `Snapshot` residue — and is not what this theorem
-- proves. This is the model's counterpart of the runtime's division between
-- `RecordedOp::If` (branch bookkeeping) and `RecordedOp::Snapshot` (value
-- bookkeeping).
------------------------------------------------------------------------

module Jtv.Conditional where

open import Data.Bool using (Bool; true; false; if_then_else_; not; _∧_)
open import Data.Empty using (⊥; ⊥-elim)
open import Data.Integer using (ℤ; _+_; -_; +_)
open import Data.Integer.Properties using (_≟_; +-injective)
open import Data.Nat using (ℕ; zero)
open import Data.Nat.Properties using () renaming (_≟_ to _≟ℕ_)
open import Data.Product using (Σ; _,_; _×_)
open import Jtv.Core using (State; Var; update; update-same; update-diff)
open import Jtv.Reversal using (+-minus)
open import Relation.Binary.PropositionalEquality
  using (_≡_; _≢_; refl; sym; trans; cong; subst)
open import Relation.Nullary using (Dec; yes; no; ¬_)
open import Relation.Nullary.Decidable.Core using (does)

zeroℤ : ℤ
zeroℤ = + 0

oneℤ : ℤ
oneℤ = + 1

------------------------------------------------------------------------
-- The guard. A guard is any Bool-valued predicate on states; the
-- counterexample uses the simplest interesting one, "x is still zero".

guard : State → Bool
guard σ = does (σ zero ≟ zeroℤ)

------------------------------------------------------------------------
-- Two reversal strategies for `reverse { if g { x += 1 } }`.

-- (a) tokenless: re-evaluate the guard on the reversed state.
reverse-if-recheck : (State → Bool) → Var → ℤ → State → State
reverse-if-recheck g x v σ = if g σ then update σ x (σ x + (- v)) else σ

-- (b) residue: replay the branch that was actually taken. This is the
-- `RecordedOp.If { condition_was_true }` discipline at runtime — the
-- Bool *is* the residue.
reverse-if-recorded : Bool → Var → ℤ → State → State
reverse-if-recorded true  x v σ = update σ x (σ x + (- v))
reverse-if-recorded false _ _ σ = σ

-- Add the given value to the variable when the guard holds in the input
-- state; otherwise return that state unchanged.
forward-if : (State → Bool) → Var → ℤ → State → State
forward-if g x v σ = if g σ then update σ x (σ x + v) else σ

------------------------------------------------------------------------
-- The counterexample: at σ ≡ 0, forward takes the branch (guard true)
-- and sets x to 1; the reversed state no longer satisfies the guard, so
-- the re-checking strategy skips the undo and the value stays 1.

-- Named as the documents cite it (`theory/agda/README.adoc` §1, §4;
-- PROOF-STATUS §4; audit §V3): an earlier revision declared this
-- `counterexample`, so every reference was uncheckable.
tokenless-conditional-not-identity :
  Σ State (λ σ → reverse-if-recheck guard zero oneℤ (forward-if guard zero oneℤ σ) ≢ σ)
tokenless-conditional-not-identity = σ , not-identity
  where
    σ : State
    σ _ = zeroℤ

    -- Both guards reduce: `guard σ` is true (0 ≟ 0) and, after the step,
    -- `guard (forward-if ...)` is false (1 ≟ 0). So the composite is the
    -- forward state, whose value at 0 is 1ℤ while σ's is 0ℤ.
    not-identity :
      reverse-if-recheck guard zero oneℤ (forward-if guard zero oneℤ σ) ≢ σ
    not-identity eq =
      -- Both guards are closed, so the composite computes to the forward
      -- state; its value at 0 is `+ 1` while σ's is `+ 0`.
      nat-absurd (+-injective (cong (λ τ → τ zero) eq))
      where
        nat-absurd : 1 ≡ 0 → ⊥
        nat-absurd = λ ()

------------------------------------------------------------------------
-- The repair, proved: recording which branch was taken restores the
-- state for *any* guard, including one whose truth changes under the
-- step. (This is the theorem the module's header advertises; it was
-- missing in the first revision of this file and is the reason
-- `reverse-if-recorded` takes a Bool rather than re-running the guard.)

update-at : ∀ σ z v y → y ≡ z → update σ z v y ≡ v
update-at σ z v y p = subst (λ q → update σ z v q ≡ v) (sym p) (update-same σ z v)

update-away : ∀ σ z v y → y ≢ z → update σ z v y ≡ σ y
update-away = Jtv.Core.update-diff


-- Generic full-state undo of one addition, stated over an abstract
-- post-state so that no `update` needs to reduce at an open index.
-- (Same technique as Jtv.Reversal; kept local to avoid a module cycle.)
undo-add-general :
  ∀ (x : Var) (v : ℤ) (σ τ : State)
  → τ x ≡ σ x + v
  → (∀ y → y ≢ x → τ y ≡ σ y)
  → ∀ y → update τ x (τ x + (- v)) y ≡ σ y
undo-add-general x v σ τ at-x off-x = go
  where
    case : ∀ y → Dec (y ≡ x) → update τ x (τ x + (- v)) y ≡ σ y
    case y (yes p) =
      trans (update-at τ x (τ x + (- v)) y p)
        (trans (cong (λ q → q + (- v)) at-x)
          (trans (+-minus (σ x) v) (cong σ (sym p))))
    case y (no p) = trans (update-away τ x (τ x + (- v)) y p) (off-x y p)

    go : ∀ y → update τ x (τ x + (- v)) y ≡ σ y
    go y = case y (y ≟ℕ x)

-- **The repair is correct.** Recording the branch taken by the forward pass
-- and replaying it makes forward-then-reverse the identity, for an arbitrary
-- guard g — not merely one that stays true. Stated pointwise (∀ y), matching
-- Jtv.Reversal: a `State`-level equality would need function extensionality,
-- which the stdlib supplies only as an axiom, and postulating an axiom to
-- prove a reversibility lemma would put an escape hatch in a Grade 3 module.
reversible-if-with-residue :
  ∀ (g : State → Bool) (x : Var) (v : ℤ) (σ : State) → ∀ y
  → reverse-if-recorded (g σ) x v (forward-if g x v σ) y ≡ σ y
reversible-if-with-residue g x v σ = go
  where
    go : ∀ y → reverse-if-recorded (g σ) x v (forward-if g x v σ) y ≡ σ y
    go y with g σ
    ... | true  = undo-add-general x v σ (update σ x (σ x + v))
                    (update-same σ x (σ x + v))
                    (λ z p → update-diff σ x (σ x + v) z p) y
    ... | false = refl
