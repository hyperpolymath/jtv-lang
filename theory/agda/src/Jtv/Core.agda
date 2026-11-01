{-# OPTIONS --cubical-compatible --safe #-}

------------------------------------------------------------------------
-- JtV.Core — the Data fragment of JtV, in Agda.
--
-- Port of the model in `jtv_proofs/JtvCore.lean` (Lean 4), written to be
-- the estate's Theory Path source of truth:
--   * standards/3-practice/PROOF-POLICY.adoc §1   (theory -> Agda)
--   * standards/3-practice/PROOF-POLICY.adoc §2.1 (canonical invocation)
--   * mathesis:estate-mathematical-architecture §5 (Grade 3 basis)
--
-- Statement correspondence with Lean is recorded per theorem in
-- `theory/agda/README.adoc` and `verification/PROOF-STATUS.adoc`.
--
-- Modelled: variables, states, the four Data constructors, evaluation,
-- free variables, and evaluation's independence from unrelated state.
-- `neg` is present because the Lean model and the current AST carry it;
-- ADR-0007 D2 flags its status as open, and porting it faithfully is the
-- point of a faithful port.
------------------------------------------------------------------------

module Jtv.Core where

open import Data.Empty using (⊥; ⊥-elim)
open import Data.Integer using (ℤ; _+_; _-_; -_)
open import Data.List using (List; []; _∷_; _++_)
open import Data.List.Membership.Propositional using (_∈_; _∉_)
open import Data.List.Membership.Propositional.Properties using (∈-++⁺ˡ; ∈-++⁺ʳ)
open import Data.List.Relation.Unary.Any using (here)
open import Data.Nat using (ℕ)
open import Data.Nat.Properties using (_≟_)
open import Relation.Binary.PropositionalEquality
  using (_≡_; _≢_; refl; sym; trans; cong; cong₂; subst)
open import Relation.Nullary using (Dec; yes; no; ¬_)

------------------------------------------------------------------------
-- Domains.

Var : Set
Var = ℕ

State : Set
State = Var → ℤ

data DataExpr : Set where
  lit : ℤ → DataExpr
  var : Var → DataExpr
  add : DataExpr → DataExpr → DataExpr
  neg : DataExpr → DataExpr

eval : DataExpr → State → ℤ
eval (lit n) σ = n
eval (var x) σ = σ x
eval (add e₁ e₂) σ = eval e₁ σ + eval e₂ σ
eval (neg e) σ = - (eval e σ)

freeVars : DataExpr → List Var
freeVars (lit n) = []
freeVars (var x) = x ∷ []
freeVars (add e₁ e₂) = freeVars e₁ ++ freeVars e₂
freeVars (neg e) = freeVars e

------------------------------------------------------------------------
-- State update and its two interaction lemmas.
--
-- `decIf` keeps the decision visible so `update-diff` can case on it; a
-- Bool-valued test would need a separate `does`-lemma.

-- Monomorphic in the result (ℤ) to keep the universe of the decision's
-- index trivially solvable; `update` only ever produces a ℤ.
decIf : ∀ {p : Set} → Dec p → ℤ → ℤ → ℤ
decIf (yes _) t f = t
decIf (no _) t f = f

update : State → Var → ℤ → State
update σ z v = λ x → decIf (x ≟ z) v (σ x)

update-same : ∀ σ z v → update σ z v z ≡ v
update-same σ z v with z ≟ z
... | yes _ = refl
... | no ¬p = ⊥-elim (¬p refl)

update-diff : ∀ σ z v x → x ≢ z → update σ z v x ≡ σ x
update-diff σ z v x x≢z with x ≟ z
... | yes p = ⊥-elim (x≢z p)
... | no _ = refl

------------------------------------------------------------------------
-- Evaluation depends only on free variables.

eval-ext : ∀ e σ τ → (∀ z → z ∈ freeVars e → σ z ≡ τ z) → eval e σ ≡ eval e τ
eval-ext (lit n) σ τ h = refl
eval-ext (var x) σ τ h = h x (here refl)
eval-ext (add e₁ e₂) σ τ h =
  cong₂ _+_
    (eval-ext e₁ σ τ (λ z p → h z (∈-++⁺ˡ p)))
    (eval-ext e₂ σ τ (λ z p → h z (∈-++⁺ʳ (freeVars e₁) p)))
eval-ext (neg e) σ τ h = cong -_ (eval-ext e σ τ h)

eval-independent : ∀ e z v σ → z ∉ freeVars e → eval e (update σ z v) ≡ eval e σ
eval-independent e z v σ z∉ =
  eval-ext e (update σ z v) σ (λ w w∈ → update-diff σ z v w (λ w≡z → z∉ (subst (λ q → q ∈ freeVars e) w≡z w∈)))
