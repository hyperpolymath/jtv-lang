{-# OPTIONS --cubical-compatible --safe #-}

------------------------------------------------------------------------
-- Jtv.Reversal — JtV v2's reversal discipline, in Agda.
--
-- Port of the `RevOp` section of `jtv_proofs/JtvTheorems.lean` (Lean 4).
-- These are the statements behind ADR-0007 D2 ("subtraction is REVERSE
-- addition") and D5 (the Safe / Neutral ladder):
--
--   * rev-forward-backward-add / -sub
--       the Safe tier: reversing `x += e` (resp. `x -= e`) restores the
--       whole state when x is not free in e.
--   * rev-forward-backward-with-token
--       the Neutral tier: with the residue (the value x held before the
--       step) recovery is unconditional, even for self-referential
--       `x += x`.
--   * naive-backward-fails-self-ref
--       an *existential* counterexample: for the specified naive inverse
--       (`execBackward`, which recomputes the operand from the current
--       state) there is a self-referential `x += x` at which it fails to
--       recover `σ x`. This shows the residue is necessary *for that
--       inverse in that case*; it is not a proof that no other
--       residue-free strategy could work.
--
-- Scope (see theory/agda/README.adoc): one operation, one token. The
-- multi-operation log, the reversible `if`, and the Rust interpreter are
-- NOT modelled here — see F1-F3 in
-- docs/reports/audit/audit-2026-11-01-jtv-v2-independence.adoc.
--
-- PROOF STRUCTURE (load-bearing, worth keeping): each reversal lemma is
-- proved first over an *abstract* post-state `τ` carrying its two
-- characterising equations (value at x, value off x). The concrete
-- theorems then instantiate τ := execForward ... and discharge those
-- equations with update-same/update-diff. This matters because
-- `update σ x v` does not reduce at an open index — the decision on
-- `x ≟ x` is stuck on a variable — so any proof that unfolds the forward
-- state at x gets stuck in `decIf`. Keeping τ opaque avoids that entirely.
------------------------------------------------------------------------

module Jtv.Reversal where

open import Data.Empty using (⊥; ⊥-elim)
open import Data.Integer using (ℤ; _+_; -_; 1ℤ)
open import Data.Integer.Properties using (+-assoc; +-identityʳ; +-inverseˡ; +-inverseʳ)
open import Data.List.Membership.Propositional using (_∈_; _∉_)
open import Data.List.Relation.Unary.Any using (here)
open import Data.Nat using (ℕ; zero)
open import Data.Nat.Properties using (_≟_)
open import Data.Product using (Σ; _,_; _×_)
open import Jtv.Core using (DataExpr; State; Var; eval; freeVars; update; var;
  update-same; update-diff; eval-independent)
open import Relation.Binary.PropositionalEquality
  using (_≡_; _≢_; refl; sym; trans; cong; subst)
open import Relation.Nullary using (Dec; yes; no; ¬_)

------------------------------------------------------------------------
-- ℤ arithmetic lemmas: the two cancellation identities reversal needs.
-- Proved here rather than imported (PROOF-POLICY §2: a missing lemma is
-- reproved by hand, not pulled in as a library dependency).
-- Note both are stated with `+ (- b)` rather than `- b`: ADR-0007 D2
-- says subtraction is not a primitive, and in this model it literally
-- is not — negation is the only primitive, and reversal is addition of
-- a negation.

+-minus : ∀ a b → (a + b) + (- b) ≡ a
+-minus a b =
  trans (+-assoc a b (- b))
        (trans (cong (λ q → a + q) (+-inverseʳ b)) (+-identityʳ a))

minus+ : ∀ a b → (a + (- b)) + b ≡ a
minus+ a b =
  trans (+-assoc a (- b) b)
        (trans (cong (λ q → a + q) (+-inverseˡ b)) (+-identityʳ a))

------------------------------------------------------------------------
-- Operations.

data RevOp : Set where
  addAssign : Var → DataExpr → RevOp
  subAssign : Var → DataExpr → RevOp

-- Apply the assignment using the operand evaluated in the original state.
execForward : RevOp → State → State
execForward (addAssign x e) σ = update σ x (σ x + eval e σ)
execForward (subAssign x e) σ = update σ x (σ x + (- eval e σ))

-- Apply the opposite arithmetic using the operand evaluated in the current
-- state. The forward round trip restores the state when the assigned variable
-- is not free in the operand; it can fail for self-referential operands.
execBackward : RevOp → State → State
execBackward (addAssign x e) σ = update σ x (σ x + (- eval e σ))
execBackward (subAssign x e) σ = update σ x (σ x + eval e σ)

-- Restore the assigned variable from the retained residue, the value it held
-- before the forward step. Other variables retain their current values.
execBackwardWithToken : RevOp → State → ℤ → State
execBackwardWithToken (addAssign x _) σ tok = update σ x tok
execBackwardWithToken (subAssign x _) σ tok = update σ x tok

------------------------------------------------------------------------
-- Helpers: a `y ≟ x` decision must NOT be `with`-abstracted around a goal
-- that calls `update`, because `update` re-evaluates its own decision and
-- the two do not convert. These two lemmas carry the decision *inside*,
-- so every `update` occurrence stays syntactically identical between the
-- goal and the lemmas.

update-at : ∀ σ z v y → y ≡ z → update σ z v y ≡ v
update-at σ z v y p = subst (λ q → update σ z v q ≡ v) (sym p) (update-same σ z v)

update-away : ∀ σ z v y → y ≢ z → update σ z v y ≡ σ y
update-away = update-diff

------------------------------------------------------------------------
-- Abstract reversal lemmas.

-- Safe tier, `x += e`: given the forward state's value at x and off x.
rev-add-general :
  ∀ x e σ τ
  → τ x ≡ σ x + eval e σ
  → eval e τ ≡ eval e σ
  → (∀ y → y ≢ x → τ y ≡ σ y)
  → ∀ y → update τ x (τ x + (- eval e τ)) y ≡ σ y
rev-add-general x e σ τ at-x ev-agree off-x = go
  where
    V : ℤ
    V = τ x + (- eval e τ)

    case : ∀ y → Dec (y ≡ x) → update τ x V y ≡ σ y
    case y (yes p) =
      trans (update-at τ x V y p)
        (trans (cong (λ q → q + (- eval e τ)) at-x)
          (trans (cong (λ q → (σ x + eval e σ) + (- q)) ev-agree)
            (trans (+-minus (σ x) (eval e σ)) (cong σ (sym p)))))
    case y (no p) = trans (update-away τ x V y p) (off-x y p)

    go : ∀ y → update τ x V y ≡ σ y
    go y = case y (y ≟ x)

-- Safe tier, `x -= e`.
rev-sub-general :
  ∀ x e σ τ
  → τ x ≡ σ x + (- eval e σ)
  → eval e τ ≡ eval e σ
  → (∀ y → y ≢ x → τ y ≡ σ y)
  → ∀ y → update τ x (τ x + eval e τ) y ≡ σ y
rev-sub-general x e σ τ at-x ev-agree off-x = go
  where
    V : ℤ
    V = τ x + eval e τ

    case : ∀ y → Dec (y ≡ x) → update τ x V y ≡ σ y
    case y (yes p) =
      trans (update-at τ x V y p)
        (trans (cong (λ q → q + eval e τ) at-x)
          (trans (cong (λ q → (σ x + (- eval e σ)) + q) ev-agree)
            (trans (minus+ (σ x) (eval e σ)) (cong σ (sym p)))))
    case y (no p) = trans (update-away τ x V y p) (off-x y p)

    go : ∀ y → update τ x V y ≡ σ y
    go y = case y (y ≟ x)

-- Neutral tier: restore from the residue; no arithmetic step at all, and
-- no need for the expression to be readable a second time.
rev-token-general :
  ∀ x σ τ
  → (∀ y → y ≢ x → τ y ≡ σ y)
  → ∀ y → update τ x (σ x) y ≡ σ y
rev-token-general x σ τ off-x = go
  where
    case : ∀ y → Dec (y ≡ x) → update τ x (σ x) y ≡ σ y
    case y (yes p) = trans (update-at τ x (σ x) y p) (cong σ (sym p))
    case y (no p)  = trans (update-away τ x (σ x) y p) (off-x y p)

    go : ∀ y → update τ x (σ x) y ≡ σ y
    go y = case y (y ≟ x)

------------------------------------------------------------------------
-- The concrete theorems: instantiate τ with the forward execution.

rev-forward-backward-add :
  ∀ x e σ → x ∉ freeVars e
  → ∀ y → execBackward (addAssign x e) (execForward (addAssign x e) σ) y ≡ σ y
rev-forward-backward-add x e σ x∉ =
  rev-add-general x e σ (update σ x (σ x + eval e σ))
    (update-same σ x (σ x + eval e σ))
    (eval-independent e x (σ x + eval e σ) σ x∉)
    (λ y p → update-diff σ x (σ x + eval e σ) y p)

rev-forward-backward-sub :
  ∀ x e σ → x ∉ freeVars e
  → ∀ y → execBackward (subAssign x e) (execForward (subAssign x e) σ) y ≡ σ y
rev-forward-backward-sub x e σ x∉ =
  rev-sub-general x e σ (update σ x (σ x + (- eval e σ)))
    (update-same σ x (σ x + (- eval e σ)))
    (eval-independent e x (σ x + (- eval e σ)) σ x∉)
    (λ y p → update-diff σ x (σ x + (- eval e σ)) y p)

rev-forward-backward-with-token :
  ∀ x e σ → ∀ y
  → execBackwardWithToken (addAssign x e) (execForward (addAssign x e) σ) (σ x) y ≡ σ y
rev-forward-backward-with-token x e σ =
  rev-token-general x σ (update σ x (σ x + eval e σ))
    (λ y p → update-diff σ x (σ x + eval e σ) y p)

------------------------------------------------------------------------
-- And the residue is necessary: the naive inverse fails on `x += x`
-- (σ x = 1 gives 1 + 1 = 2, and 2 + (- 2) = 0 ≠ 1).

naive-backward-fails-self-ref :
  Σ State (λ σ → Σ Var (λ x → Σ DataExpr (λ e →
    (x ∈ freeVars e) ×
    (execBackward (addAssign x e) (execForward (addAssign x e) σ) x ≢ σ x))))
naive-backward-fails-self-ref = σ , zero , var zero , self-ref , counter
  where
    σ : State
    σ _ = 1ℤ

    σ₁ : State
    σ₁ = update σ zero (1ℤ + eval (var zero) σ)

    self-ref : zero ∈ freeVars (var zero)
    self-ref = here refl

    -- The naive inverse computes (σ₁ 0) + (- (σ₁ 0)) = 0, not 1.
    counter : update σ₁ zero (σ₁ zero + (- eval (var zero) σ₁)) zero ≢ σ zero
    counter ()
