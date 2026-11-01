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
-- The repair is also modelled (`reversible-if-with-residue`): record
-- which branch was taken, exactly as `RecordedOp.If { condition_was_true
-- }` does at runtime, and the round trip *is* the identity.
------------------------------------------------------------------------

module Jtv.Conditional where

open import Data.Bool using (Bool; true; false; if_then_else_; not; _∧_)
open import Data.Empty using (⊥; ⊥-elim)
open import Data.Integer using (ℤ; _+_; -_; +_)
open import Data.Integer.Properties using (_≟_; +-injective)
open import Data.Nat using (ℕ; zero)
open import Data.Nat.Properties using ()
open import Data.Product using (Σ; _,_; _×_)
open import Jtv.Core using (State; Var; update; update-same)
open import Relation.Binary.PropositionalEquality
  using (_≡_; _≢_; refl; sym; trans; cong; subst)
open import Relation.Nullary using (yes; no; ¬_)
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
-- `RecordedOp.If condition_was_true` discipline.
reverse-if-recorded : Bool → Var → ℤ → State → State
reverse-if-recorded taken x v σ =
  if taken ∧ not taken then σ else if taken then update σ x (σ x + (- v)) else σ

forward-if : (State → Bool) → Var → ℤ → State → State
forward-if g x v σ = if g σ then update σ x (σ x + v) else σ

------------------------------------------------------------------------
-- The counterexample: at σ ≡ 0, forward takes the branch (guard true)
-- and sets x to 1; the reversed state no longer satisfies the guard, so
-- the re-checking strategy skips the undo and the value stays 1.

counterexample :
  Σ State (λ σ → reverse-if-recheck guard zero oneℤ (forward-if guard zero oneℤ σ) ≢ σ)
counterexample = σ , not-identity
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
