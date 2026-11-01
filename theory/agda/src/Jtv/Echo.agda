{-# OPTIONS --cubical-compatible --safe #-}

------------------------------------------------------------------------
-- Jtv.Echo — the structured-loss lattice and the retained-loss fibre,
-- in Agda. Port of `jtv_proofs/JtvEcho.lean` (Lean 4).
--
-- This is the module that carries the estate's *cross-project* theory
-- material. Two things are deliberately separated here:
--
--   1. THE LATTICE AND THE FIBRE (this file): JtV's own mechanisation.
--      The fibre `Echo f y := Σ x, f x ≡ y` is the retained-loss lineage
--      of a map at an output; "Safe" is the case where that fibre is a
--      subsingleton (no loss to retain), "Breaking" the case where the
--      lineage is not retained at all.
--
--   2. THE CORRESPONDENCE to the `echo-types` Agda library: *argued,
--      not bridged*. JtV states the same fibre construction; there is no
--      transported witness and no statement comparison recorded against
--      that library, so under
--      standards/3-practice/PROOF-POLICY.adoc §2.2 this row is `argued`
--      and the `echo-types` side is `open` here. See
--      verification/PROOF-STATUS.adoc.
--
-- Note on scope: nothing in this module imports, transports from, or
-- depends on any other repository. It is a self-contained proof about
-- JtV's own lattice.
------------------------------------------------------------------------

module Jtv.Echo where

open import Data.Bool using (Bool; true; false; _∧_)
open import Data.Empty using (⊥; ⊥-elim)
open import Data.Product using (Σ; _,_; _×_; proj₁; proj₂)
open import Data.Sum using (_⊎_; inj₁; inj₂)
open import Relation.Binary.PropositionalEquality
  using (_≡_; refl; sym; trans; cong; subst)
open import Relation.Nullary using (¬_; yes; no)

------------------------------------------------------------------------
-- SECTION 1: the lattice.

data Echo : Set where
  Safe Neutral Breaking : Echo

-- Order: Safe ⊑ Neutral ⊑ Breaking. Join loses guarantees.
data ⊤′ : Set where tt′ : ⊤′

_⊑_ : Echo → Echo → Set
Safe    ⊑ _        = ⊤′
Neutral ⊑ Safe     = ⊥
Neutral ⊑ Neutral  = ⊤′
Neutral ⊑ Breaking = ⊤′
Breaking ⊑ Safe    = ⊥
Breaking ⊑ Neutral = ⊥
Breaking ⊑ Breaking = ⊤′

-- Join = the worse of the two tiers (Safe is the unit, Breaking absorbs).
_⊔_ : Echo → Echo → Echo
Safe    ⊔ e        = e
Neutral ⊔ Safe     = Neutral
Neutral ⊔ Neutral  = Neutral
Neutral ⊔ Breaking = Breaking
Breaking ⊔ _       = Breaking

⊔-idem : ∀ (e : Echo) → e ⊔ e ≡ e
⊔-idem Safe = refl
⊔-idem Neutral = refl
⊔-idem Breaking = refl

⊔-comm : ∀ (a b : Echo) → a ⊔ b ≡ b ⊔ a
⊔-comm Safe Safe = refl
⊔-comm Safe Neutral = refl
⊔-comm Safe Breaking = refl
⊔-comm Neutral Safe = refl
⊔-comm Neutral Neutral = refl
⊔-comm Neutral Breaking = refl
⊔-comm Breaking Safe = refl
⊔-comm Breaking Neutral = refl
⊔-comm Breaking Breaking = refl

-- Associativity needs the third argument split too: `Neutral ⊔ c` does
-- not reduce while `c` is a variable.
⊔-assoc : ∀ (a b c : Echo) → (a ⊔ b) ⊔ c ≡ a ⊔ (b ⊔ c)
⊔-assoc Safe b c = refl
⊔-assoc Neutral Safe c = refl
⊔-assoc Neutral Neutral Safe = refl
⊔-assoc Neutral Neutral Neutral = refl
⊔-assoc Neutral Neutral Breaking = refl
⊔-assoc Neutral Breaking Safe = refl
⊔-assoc Neutral Breaking Neutral = refl
⊔-assoc Neutral Breaking Breaking = refl
⊔-assoc Breaking b c = refl

-- Safe is the unit; Breaking is absorbing.
⊔-unitˡ : ∀ (e : Echo) → Safe ⊔ e ≡ e
⊔-unitˡ e = refl

⊔-absorbingʳ : ∀ (e : Echo) → e ⊔ Breaking ≡ Breaking
⊔-absorbingʳ Safe = refl
⊔-absorbingʳ Neutral = refl
⊔-absorbingʳ Breaking = refl

------------------------------------------------------------------------
-- SECTION 2: the two admissibility policies (ADR-0007 D5).
--
--   * `admissible`             — the Safe-only policy for tokenless
--                                `reverse { }` blocks.
--   * `admissibleWithResidue`  — the Bennett policy for
--                                `reversible { } -> tok`: Neutral is
--                                recoverable *because* the residue is
--                                retained; only Breaking is refused.

admissible : Echo → Bool
admissible Safe = true
admissible Neutral = false
admissible Breaking = false

admissibleWithResidue : Echo → Bool
admissibleWithResidue Breaking = false
admissibleWithResidue _ = true

admissibleWithResidue-iff : ∀ (e : Echo) → admissibleWithResidue e ≡ true → e ≡ Safe ⊎ e ≡ Neutral
admissibleWithResidue-iff Safe _ = inj₁ refl
admissibleWithResidue-iff Neutral _ = inj₂ refl
admissibleWithResidue-iff Breaking ()

-- The Safe-only policy is the stricter one: it implies the residue one.
admissible⇒admissibleWithResidue :
  ∀ (e : Echo) → admissible e ≡ true → admissibleWithResidue e ≡ true
admissible⇒admissibleWithResidue Safe _ = refl
admissible⇒admissibleWithResidue Neutral ()
admissible⇒admissibleWithResidue Breaking ()

-- Neutral is exactly what the token buys (ADR-0007 D5’s “(c)” bridge):
-- refused without a residue, admitted with one.
token-buys-neutral :
  admissible Neutral ≡ false × admissibleWithResidue Neutral ≡ true
token-buys-neutral = refl , refl

------------------------------------------------------------------------
-- SECTION 3: blocks, and the admissibility theorems
-- (`blockEcho_admissible` / `blockEcho_admissibleWithResidue` in Lean).

open import Data.List using (List; []; _∷_)

blockEcho : List Echo → Echo
blockEcho = foldr _⊔_ Safe
  where
    foldr : ∀ {A B : Set} → (A → B → B) → B → List A → B
    foldr f z [] = z
    foldr f z (x ∷ xs) = f x (foldr f z xs)

allAdmissible : List Echo → Bool
allAdmissible [] = true
allAdmissible (e ∷ es) = admissible e ∧ allAdmissible es

allAdmissibleWithResidue : List Echo → Bool
allAdmissibleWithResidue [] = true
allAdmissibleWithResidue (e ∷ es) = admissibleWithResidue e ∧ allAdmissibleWithResidue es

-- Join distributes over the residue policy. This is the induction step
-- for the block theorem: without it `admissibleWithResidue (e ⊔ rest)`
-- is stuck on the tail.
join-admissibleWithResidue :
  ∀ (a b : Echo) → admissibleWithResidue (a ⊔ b) ≡ (admissibleWithResidue a ∧ admissibleWithResidue b)
join-admissibleWithResidue Safe Safe = refl
join-admissibleWithResidue Safe Neutral = refl
join-admissibleWithResidue Safe Breaking = refl
join-admissibleWithResidue Neutral Safe = refl
join-admissibleWithResidue Neutral Neutral = refl
join-admissibleWithResidue Neutral Breaking = refl
join-admissibleWithResidue Breaking _ = refl

blockEcho-admissibleWithResidue :
  ∀ (es : List Echo) → admissibleWithResidue (blockEcho es) ≡ allAdmissibleWithResidue es
blockEcho-admissibleWithResidue [] = refl
blockEcho-admissibleWithResidue (e ∷ es) =
  trans (join-admissibleWithResidue e (blockEcho es))
        (cong (admissibleWithResidue e ∧_) (blockEcho-admissibleWithResidue es))

------------------------------------------------------------------------
-- SECTION 4: the retained-loss fibre.
--
-- `Echo f y` is the lineage of `y` under `f`: every input that maps to
-- it. "No loss" is the case where this is a subsingleton — there is
-- nothing to retain because nothing was collapsed.

Echo-of : ∀ {A B : Set} → (A → B) → B → Set
Echo-of {A} f y = Σ A (λ x → f x ≡ y)

-- Safe = injective, read through the fibre: any two inputs that produce
-- the same output are the *same input*, so the lineage carries no
-- branching. (Stated on first components: two fibre inhabitants are
-- equal as soon as their inputs are, and proof irrelevance for the
-- equality components is neither needed nor available under --safe.)
fibre-inputs-agree :
  ∀ {A B : Set} (f : A → B)
  → (∀ {x y} → f x ≡ f y → x ≡ y)
  → ∀ y (u v : Echo-of f y) → proj₁ u ≡ proj₁ v
fibre-inputs-agree f inj y (x , p) (x′ , q) = inj (trans p (sym q))

-- The tier is only meaningful if loss can actually happen, so: a
-- non-injective map has a fibre over one output with two distinct
-- inputs. Exhibited concretely (a constant map) — this is the mechanical
-- form of "structured loss", the thing Neutral must retain a residue for
-- and Breaking cannot recover at all.
const : Bool → Bool
const _ = true

loss-is-real :
  Σ Bool (λ y → Σ Bool (λ x → Σ Bool (λ x′ → (const x ≡ y) × (const x′ ≡ y) × (x ≡ x′ → ⊥))))
loss-is-real = true , true , false , refl , refl , (λ ())
