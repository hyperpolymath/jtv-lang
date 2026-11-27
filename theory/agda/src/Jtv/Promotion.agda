{-# OPTIONS --cubical-compatible --safe #-}

------------------------------------------------------------------------
-- Jtv.Promotion — grading a reversible step by the carrier it produces.
--
-- A reversible step `x += v` (or `x -= v`) is graded on the Echo lattice.
-- Before this module the grade looked only at the *target's* carrier. That
-- is unsound when the operand promotes the target to another carrier:
--
--   * `Int += Float` produces a Float. Undoing it by subtraction rounds,
--     and the result is a Float where an Int stood.
--   * `Int += Rational` produces a Rational. Undoing it by subtraction is
--     exact as arithmetic, but leaves `1/1` where the integer `1` stood.
--
-- This module states the rule that closes both, and proves it:
--
--   1. THE GRADE (`stepEcho`) joins the statement shape, the target's
--      carrier, the *result's* carrier, and a `Neutral` for any change of
--      carrier.
--   2. BOUND (`result-bounds-grade`, `target-bounds-grade`): the grade is at
--      least the carrier tier of the promoted result, and of the target.
--   3. SOUNDNESS OF `Safe` (`safe-grade-sound`): a step graded `Safe` has a
--      Safe shape, a Safe target carrier, and no change of carrier.
--   4. WHY IT IS NEEDED (`RoundTrip.promoted-add-not-restored`,
--      `promoted-sub-not-restored`): for *any* implementation whose `add`
--      follows the promotion table, undoing a promoting step by arithmetic
--      inversion never returns the original value.
--
-- Scope. The carrier tier map `ce` is a parameter: the theorems hold for
-- every classification, so they do not depend on which carriers the
-- implementation currently calls Safe. The promotion table mirrors
-- `Value::add` in `crates/jtv-core/src/number.rs`; that correspondence is
-- by inspection (`argued`), not a transported witness. Exact restoration
-- for a Safe step over ℤ is `Jtv.Reversal`; the other exact carriers have
-- no value-level proof here.
------------------------------------------------------------------------

module Jtv.Promotion where

open import Jtv.Echo
  using (Echo; Safe; Neutral; Breaking; _⊑_; _⊔_; tt′)

open import Data.Empty using (⊥; ⊥-elim)
open import Data.Maybe using (Maybe; just; nothing)
open import Data.Nat using (ℕ; zero; suc)
open import Data.Nat.Properties using () renaming (_≟_ to _≟ℕ_)
open import Data.Product using (_×_; _,_; proj₁; proj₂)
open import Relation.Binary.PropositionalEquality
  using (_≡_; _≢_; refl; sym; trans; cong)
open import Relation.Nullary using (¬_; Dec; yes; no)

------------------------------------------------------------------------
-- SECTION 1: carriers and their decidable equality.

data Carrier : Set where
  int rational float complex hex binary symbolic : Carrier

private
  code : Carrier → ℕ
  code int      = 0
  code rational = 1
  code float    = 2
  code complex  = 3
  code hex      = 4
  code binary   = 5
  code symbolic = 6

  decode : ℕ → Maybe Carrier
  decode 0 = just int
  decode 1 = just rational
  decode 2 = just float
  decode 3 = just complex
  decode 4 = just hex
  decode 5 = just binary
  decode 6 = just symbolic
  decode _ = nothing

  decode-code : ∀ c → decode (code c) ≡ just c
  decode-code int      = refl
  decode-code rational = refl
  decode-code float    = refl
  decode-code complex  = refl
  decode-code hex      = refl
  decode-code binary   = refl
  decode-code symbolic = refl

just-injective : ∀ {A : Set} {a b : A} → just a ≡ just b → a ≡ b
just-injective refl = refl

private
  code-injective : ∀ a b → code a ≡ code b → a ≡ b
  code-injective a b p =
    just-injective (trans (sym (decode-code a)) (trans (cong decode p) (decode-code b)))

_≟_ : (a b : Carrier) → Dec (a ≡ b)
a ≟ b with code a ≟ℕ code b
... | yes p = yes (code-injective a b p)
... | no ¬p = no (λ q → ¬p (cong code q))

------------------------------------------------------------------------
-- SECTION 2: the promotion table.
--
-- `promote a b` is the carrier of `a + b`, or `nothing` where the sum is a
-- type error. It mirrors the arms of `Value::add`: the seven same-carrier
-- arms, then the six coercion arms.

promote : Carrier → Carrier → Maybe Carrier
promote int      int      = just int
promote rational rational = just rational
promote float    float    = just float
promote complex  complex  = just complex
promote hex      hex      = just hex
promote binary   binary   = just binary
promote symbolic symbolic = just symbolic
promote int      float    = just float
promote float    int      = just float
promote int      rational = just rational
promote rational int      = just rational
promote float    complex  = just complex
promote complex  float    = just complex
promote _        _        = nothing

-- Adding the same operand again does not promote further. This is what
-- makes an arithmetic inverse stay in the promoted carrier.
promote-absorb : ∀ a b c → promote a b ≡ just c → promote c b ≡ just c
promote-absorb int      int      _ refl = refl
promote-absorb int      rational _ refl = refl
promote-absorb int      float    _ refl = refl
promote-absorb int      complex  _ ()
promote-absorb int      hex      _ ()
promote-absorb int      binary   _ ()
promote-absorb int      symbolic _ ()
promote-absorb rational int      _ refl = refl
promote-absorb rational rational _ refl = refl
promote-absorb rational float    _ ()
promote-absorb rational complex  _ ()
promote-absorb rational hex      _ ()
promote-absorb rational binary   _ ()
promote-absorb rational symbolic _ ()
promote-absorb float    int      _ refl = refl
promote-absorb float    rational _ ()
promote-absorb float    float    _ refl = refl
promote-absorb float    complex  _ refl = refl
promote-absorb float    hex      _ ()
promote-absorb float    binary   _ ()
promote-absorb float    symbolic _ ()
promote-absorb complex  int      _ ()
promote-absorb complex  rational _ ()
promote-absorb complex  float    _ refl = refl
promote-absorb complex  complex  _ refl = refl
promote-absorb complex  hex      _ ()
promote-absorb complex  binary   _ ()
promote-absorb complex  symbolic _ ()
promote-absorb hex      int      _ ()
promote-absorb hex      rational _ ()
promote-absorb hex      float    _ ()
promote-absorb hex      complex  _ ()
promote-absorb hex      hex      _ refl = refl
promote-absorb hex      binary   _ ()
promote-absorb hex      symbolic _ ()
promote-absorb binary   int      _ ()
promote-absorb binary   rational _ ()
promote-absorb binary   float    _ ()
promote-absorb binary   complex  _ ()
promote-absorb binary   hex      _ ()
promote-absorb binary   binary   _ refl = refl
promote-absorb binary   symbolic _ ()
promote-absorb symbolic int      _ ()
promote-absorb symbolic rational _ ()
promote-absorb symbolic float    _ ()
promote-absorb symbolic complex  _ ()
promote-absorb symbolic hex      _ ()
promote-absorb symbolic binary   _ ()
promote-absorb symbolic symbolic _ refl = refl

------------------------------------------------------------------------
-- SECTION 3: order facts about the Echo lattice used below.

⊑-refl : ∀ a → a ⊑ a
⊑-refl Safe     = tt′
⊑-refl Neutral  = tt′
⊑-refl Breaking = tt′

⊑-trans : ∀ a b c → a ⊑ b → b ⊑ c → a ⊑ c
⊑-trans Safe     _        _        _  _  = tt′
⊑-trans Neutral  Safe     _        () _
⊑-trans Neutral  Neutral  Safe     _  ()
⊑-trans Neutral  Neutral  Neutral  _  _  = tt′
⊑-trans Neutral  Neutral  Breaking _  _  = tt′
⊑-trans Neutral  Breaking Safe     _  ()
⊑-trans Neutral  Breaking Neutral  _  ()
⊑-trans Neutral  Breaking Breaking _  _  = tt′
⊑-trans Breaking Safe     _        () _
⊑-trans Breaking Neutral  _        () _
⊑-trans Breaking Breaking Safe     _  ()
⊑-trans Breaking Breaking Neutral  _  ()
⊑-trans Breaking Breaking Breaking _  _  = tt′

-- The join is an upper bound of both arguments.
⊔-upperˡ : ∀ a b → a ⊑ (a ⊔ b)
⊔-upperˡ Safe     _        = tt′
⊔-upperˡ Neutral  Safe     = tt′
⊔-upperˡ Neutral  Neutral  = tt′
⊔-upperˡ Neutral  Breaking = tt′
⊔-upperˡ Breaking _        = tt′

⊔-upperʳ : ∀ a b → b ⊑ (a ⊔ b)
⊔-upperʳ Safe     b        = ⊑-refl b
⊔-upperʳ Neutral  Safe     = tt′
⊔-upperʳ Neutral  Neutral  = tt′
⊔-upperʳ Neutral  Breaking = tt′
⊔-upperʳ Breaking Safe     = tt′
⊔-upperʳ Breaking Neutral  = tt′
⊔-upperʳ Breaking Breaking = tt′

-- A join is Safe only when both arguments are.
⊔-safe-inv : ∀ a b → a ⊔ b ≡ Safe → a ≡ Safe × b ≡ Safe
⊔-safe-inv Safe     Safe     _  = refl , refl
⊔-safe-inv Safe     Neutral  ()
⊔-safe-inv Safe     Breaking ()
⊔-safe-inv Neutral  Safe     ()
⊔-safe-inv Neutral  Neutral  ()
⊔-safe-inv Neutral  Breaking ()
⊔-safe-inv Breaking _        ()

------------------------------------------------------------------------
-- SECTION 4: the grade of a reversible step.
--
-- `change a c` is the loss from the carrier itself changing: the original
-- value lived in `a`, and an arithmetic inverse of a value in `c` stays in
-- `c` (SECTION 5). It is `Neutral`, not `Breaking`, because a retained
-- snapshot of the old value restores it.

change : Carrier → Carrier → Echo
change a c with a ≟ c
... | yes _ = Safe
... | no  _ = Neutral

change-safe : ∀ a c → change a c ≡ Safe → a ≡ c
change-safe a c p with a ≟ c
change-safe a c p  | yes q = q
change-safe a c () | no  _

change-neutral : ∀ a c → a ≢ c → change a c ≡ Neutral
change-neutral a c ne with a ≟ c
... | yes q = ⊥-elim (ne q)
... | no  _ = refl

-- The grade. `ce` is the carrier tier map, `shape` the statement-shape tier
-- (self-reference is Neutral), `cx` the target's carrier and `c` the carrier
-- of the result, i.e. `promote cx cv ≡ just c` for the operand's carrier `cv`.
stepEcho : (Carrier → Echo) → Echo → Carrier → Carrier → Echo
stepEcho ce shape cx c = shape ⊔ (ce cx ⊔ (ce c ⊔ change cx c))

-- The grade is at least the tier of the target's carrier ...
target-bounds-grade : ∀ ce shape cx c → ce cx ⊑ stepEcho ce shape cx c
target-bounds-grade ce shape cx c =
  ⊑-trans (ce cx) (ce cx ⊔ (ce c ⊔ change cx c)) (stepEcho ce shape cx c)
    (⊔-upperˡ (ce cx) (ce c ⊔ change cx c))
    (⊔-upperʳ shape (ce cx ⊔ (ce c ⊔ change cx c)))

-- ... and at least the tier of the promoted result's carrier.
result-bounds-grade : ∀ ce shape cx c → ce c ⊑ stepEcho ce shape cx c
result-bounds-grade ce shape cx c =
  ⊑-trans (ce c) (ce cx ⊔ (ce c ⊔ change cx c)) (stepEcho ce shape cx c)
    (⊑-trans (ce c) (ce c ⊔ change cx c) (ce cx ⊔ (ce c ⊔ change cx c))
      (⊔-upperˡ (ce c) (change cx c))
      (⊔-upperʳ (ce cx) (ce c ⊔ change cx c)))
    (⊔-upperʳ shape (ce cx ⊔ (ce c ⊔ change cx c)))

-- A step graded Safe has a Safe shape, Safe carriers and no change of
-- carrier. This is the direction the checker relies on: it admits a
-- tokenless `reverse { }` only at grade Safe.
safe-grade-sound :
  ∀ ce shape cx c → stepEcho ce shape cx c ≡ Safe →
  shape ≡ Safe × ce cx ≡ Safe × ce c ≡ Safe × cx ≡ c
safe-grade-sound ce shape cx c p =
  let s₁ = ⊔-safe-inv shape (ce cx ⊔ (ce c ⊔ change cx c)) p
      s₂ = ⊔-safe-inv (ce cx) (ce c ⊔ change cx c) (proj₂ s₁)
      s₃ = ⊔-safe-inv (ce c) (change cx c) (proj₂ s₂)
  in proj₁ s₁ , proj₁ s₂ , proj₁ s₃ , change-safe cx c (proj₂ s₃)

-- So a step that changes carrier is never graded Safe ...
promoting-step-not-safe :
  ∀ ce shape cx c → cx ≢ c → stepEcho ce shape cx c ≢ Safe
promoting-step-not-safe ce shape cx c ne p =
  ne (proj₂ (proj₂ (proj₂ (safe-grade-sound ce shape cx c p))))

-- ... it is at least Neutral.
promoting-step-at-least-neutral :
  ∀ ce shape cx c → cx ≢ c → Neutral ⊑ stepEcho ce shape cx c
promoting-step-at-least-neutral ce shape cx c ne
  with change cx c | change-neutral cx c ne
... | .Neutral | refl =
  ⊑-trans Neutral (ce cx ⊔ (ce c ⊔ Neutral)) (shape ⊔ (ce cx ⊔ (ce c ⊔ Neutral)))
    (⊑-trans Neutral (ce c ⊔ Neutral) (ce cx ⊔ (ce c ⊔ Neutral))
      (⊔-upperʳ (ce c) Neutral)
      (⊔-upperʳ (ce cx) (ce c ⊔ Neutral)))
    (⊔-upperʳ shape (ce cx ⊔ (ce c ⊔ Neutral)))

------------------------------------------------------------------------
-- SECTION 5: why the rule is needed — arithmetic inversion cannot undo a
-- promotion.
--
-- The theorems hold for any value type `V` with carrier tags, a partial
-- `add` and a `neg`, provided `add` follows the promotion table and `neg`
-- keeps the carrier. Nothing else about the implementation is assumed.

module RoundTrip
  (V       : Set)
  (tag     : V → Carrier)
  (add     : V → V → Maybe V)
  (neg     : V → V)
  (add-tag : ∀ x v r → add x v ≡ just r → promote (tag x) (tag v) ≡ just (tag r))
  (neg-tag : ∀ v → tag (neg v) ≡ tag v)
  where

  -- Undoing `x += v` by `r += neg v` lands in the carrier of `r`.
  add-inverse-stays-promoted :
    ∀ x v r x′ → add x v ≡ just r → add r (neg v) ≡ just x′ → tag x′ ≡ tag r
  add-inverse-stays-promoted x v r x′ fwd bwd =
    let absorbed : promote (tag r) (tag v) ≡ just (tag r)
        absorbed = promote-absorb (tag x) (tag v) (tag r) (add-tag x v r fwd)
        undone : promote (tag r) (tag v) ≡ just (tag x′)
        undone = trans (cong (promote (tag r)) (sym (neg-tag v))) (add-tag r (neg v) x′ bwd)
    in just-injective (trans (sym undone) absorbed)

  -- So when the forward step changed the carrier, the value it restores is
  -- not the original.
  promoted-add-not-restored :
    ∀ x v r x′ → add x v ≡ just r → add r (neg v) ≡ just x′ →
    tag x ≢ tag r → x′ ≢ x
  promoted-add-not-restored x v r x′ fwd bwd ne eq =
    ne (trans (sym (cong tag eq)) (add-inverse-stays-promoted x v r x′ fwd bwd))

  -- The same for `x -= v`, undone by `r += v`.
  sub-inverse-stays-promoted :
    ∀ x v r x′ → add x (neg v) ≡ just r → add r v ≡ just x′ → tag x′ ≡ tag r
  sub-inverse-stays-promoted x v r x′ fwd bwd =
    let forward : promote (tag x) (tag v) ≡ just (tag r)
        forward = trans (cong (promote (tag x)) (sym (neg-tag v))) (add-tag x (neg v) r fwd)
        absorbed : promote (tag r) (tag v) ≡ just (tag r)
        absorbed = promote-absorb (tag x) (tag v) (tag r) forward
    in just-injective (trans (sym (add-tag r v x′ bwd)) absorbed)

  promoted-sub-not-restored :
    ∀ x v r x′ → add x (neg v) ≡ just r → add r v ≡ just x′ →
    tag x ≢ tag r → x′ ≢ x
  promoted-sub-not-restored x v r x′ fwd bwd ne eq =
    ne (trans (sym (cong tag eq)) (sub-inverse-stays-promoted x v r x′ fwd bwd))

  -- The grade and the semantics agree: a step graded Safe did not promote,
  -- so neither theorem above applies to it.
  safe-step-keeps-carrier :
    ∀ ce shape x v r → add x v ≡ just r →
    stepEcho ce shape (tag x) (tag r) ≡ Safe → tag x ≡ tag r
  safe-step-keeps-carrier ce shape x v r _ p =
    proj₂ (proj₂ (proj₂ (safe-grade-sound ce shape (tag x) (tag r) p)))

------------------------------------------------------------------------
-- SECTION 6: the assumptions are satisfiable, and the two reported cases.
--
-- Taking the carriers themselves as values (tag = identity, add = promote)
-- satisfies both assumptions of `RoundTrip`, so its theorems are not
-- vacuous. The two instances are the defects this module was written for.

private
  module Tags =
    RoundTrip Carrier (λ c → c) promote (λ c → c) (λ _ _ _ p → p) (λ _ → refl)

-- `Int += Float`: whatever the inverse step returns, it is not the Int.
int-plus-float-not-restored : ∀ x′ → promote float float ≡ just x′ → x′ ≢ int
int-plus-float-not-restored x′ bwd =
  Tags.promoted-add-not-restored int float float x′ refl bwd (λ ())

-- `Int += Rational`: likewise, although ℚ itself is an exact group.
int-plus-rational-not-restored : ∀ x′ → promote rational rational ≡ just x′ → x′ ≢ int
int-plus-rational-not-restored x′ bwd =
  Tags.promoted-add-not-restored int rational rational x′ refl bwd (λ ())
