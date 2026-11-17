-- SPDX-License-Identifier: MPL-2.0
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath)
/-
  JtV — grading a reversible step by the carrier it produces (Lean 4).

  The Lean counterpart of `theory/agda/src/Jtv/Promotion.agda`, written so the
  Agda statements have a Lean statement to be compared against (PROOF-POLICY
  §2.2). The Agda module is the source of truth; this file is target-side
  evidence until the owner reconciles the statements.

  A reversible step `x += v` is graded on the Echo lattice. Grading by the
  target's carrier alone is unsound when the operand promotes the target:
  `Int += Float` leaves a Float and `Int += Rational` leaves a Rational, and
  an arithmetic inverse stays in the promoted carrier. `stepEcho` therefore
  joins the statement shape, the target's carrier tier, the result's carrier
  tier, and `neutral` for any change of carrier.

  * `result_bounds_grade`, `target_bounds_grade` — the grade bounds both tiers.
  * `safe_grade_sound` — a step graded `safe` has a safe shape, safe carriers
    and no change of carrier.
  * `promoted_add_not_restored`, `promoted_sub_not_restored` — for any
    implementation whose `add` follows the promotion table, undoing a
    promoting step by arithmetic inversion never returns the original value.

  Scope, as in the Agda module: the tier map `ce` is a parameter; the table
  mirrors `Value::add` in `crates/jtv-core/src/number.rs` by inspection (and
  by the Rust test `promote_agrees_with_value_add_on_every_carrier_pair`),
  not by a transported witness.

  Correspondence notes (recorded per theorem in `theory/agda/README.adoc`):
  Agda's `change` branches on a decision procedure built from a ℕ encoding;
  here it branches on the derived `DecidableEq`. Agda's `RoundTrip` is a
  parameterised module; here its parameters are explicit arguments of each
  theorem in the `RoundTrip` section.
-/
import JtvEcho

namespace Jtv.Promotion

open Jtv.Echo

-- ============================================================================
-- SECTION 1: CARRIERS
-- ============================================================================

/-- The seven number carriers of `Value`. -/
inductive Carrier where
  | int
  | rational
  | float
  | complex
  | hex
  | binary
  | symbolic
  deriving Repr, DecidableEq

-- ============================================================================
-- SECTION 2: THE PROMOTION TABLE
-- ============================================================================

/-- `promote a b` is the carrier of `a + b`, or `none` where the sum is a type
    error: the seven same-carrier arms of `Value::add`, then its six coercion
    arms. -/
def promote : Carrier → Carrier → Option Carrier
  | .int,      .int      => some .int
  | .rational, .rational => some .rational
  | .float,    .float    => some .float
  | .complex,  .complex  => some .complex
  | .hex,      .hex      => some .hex
  | .binary,   .binary   => some .binary
  | .symbolic, .symbolic => some .symbolic
  | .int,      .float    => some .float
  | .float,    .int      => some .float
  | .int,      .rational => some .rational
  | .rational, .int      => some .rational
  | .float,    .complex  => some .complex
  | .complex,  .float    => some .complex
  | _,         _         => none

/-- Adding the same operand again does not promote further. This is what makes
    an arithmetic inverse stay in the promoted carrier. -/
theorem promote_absorb (a b c : Carrier) :
    promote a b = some c → promote c b = some c := by
  cases a <;> cases b <;> cases c <;> intro h <;>
    first
      | rfl
      | cases h

-- ============================================================================
-- SECTION 3: ORDER FACTS
-- ============================================================================

/-- Transitivity of the Echo order, by cases. (`Jtv.Echo.le_trans` proves the
    same statement with `simp_all`, which brings in `propext`; this proof uses
    no axiom, so nothing in this file does.) -/
theorem le_trans' (a b c : Echo) : a ≤ b → b ≤ c → a ≤ c := by
  cases a <;> cases b <;> cases c <;> intro hab hbc <;>
    first
      | rfl
      | (cases hab; done)
      | (cases hbc; done)

theorem join_upper_left (a b : Echo) : a ≤ a ⊔ b := by
  cases a <;> cases b <;> rfl

theorem join_upper_right (a b : Echo) : b ≤ a ⊔ b := by
  cases a <;> cases b <;> rfl

theorem join_safe_inv (a b : Echo) :
    a ⊔ b = Echo.safe → a = Echo.safe ∧ b = Echo.safe := by
  cases a <;> cases b <;> intro h <;>
    first
      | exact ⟨rfl, rfl⟩
      | cases h

-- ============================================================================
-- SECTION 4: THE GRADE OF A REVERSIBLE STEP
-- ============================================================================

/-- The loss from the carrier itself changing. `neutral`, not `breaking`,
    because a retained snapshot of the old value restores it. -/
def change (a c : Carrier) : Echo :=
  if a = c then Echo.safe else Echo.neutral

theorem change_safe (a c : Carrier) : change a c = Echo.safe → a = c := by
  unfold change
  intro h
  by_cases q : a = c
  · exact q
  · rw [if_neg q] at h
    cases h

theorem change_neutral (a c : Carrier) : a ≠ c → change a c = Echo.neutral := by
  intro ne
  unfold change
  exact if_neg ne

/-- The grade. `ce` is the carrier tier map, `shape` the statement-shape tier,
    `cx` the target's carrier and `c` the carrier of the result. -/
def stepEcho (ce : Carrier → Echo) (shape : Echo) (cx c : Carrier) : Echo :=
  shape ⊔ (ce cx ⊔ (ce c ⊔ change cx c))

/-- The grade is at least the tier of the target's carrier. -/
theorem target_bounds_grade (ce : Carrier → Echo) (shape : Echo) (cx c : Carrier) :
    ce cx ≤ stepEcho ce shape cx c :=
  le_trans' _ _ _
    (join_upper_left (ce cx) (ce c ⊔ change cx c))
    (join_upper_right shape (ce cx ⊔ (ce c ⊔ change cx c)))

/-- The grade is at least the tier of the promoted result's carrier. -/
theorem result_bounds_grade (ce : Carrier → Echo) (shape : Echo) (cx c : Carrier) :
    ce c ≤ stepEcho ce shape cx c :=
  le_trans' _ _ _
    (le_trans' _ _ _
      (join_upper_left (ce c) (change cx c))
      (join_upper_right (ce cx) (ce c ⊔ change cx c)))
    (join_upper_right shape (ce cx ⊔ (ce c ⊔ change cx c)))

/-- A step graded `safe` has a safe shape, safe carriers and no change of
    carrier. The checker admits a tokenless `reverse { }` only at this grade. -/
theorem safe_grade_sound (ce : Carrier → Echo) (shape : Echo) (cx c : Carrier) :
    stepEcho ce shape cx c = Echo.safe →
    shape = Echo.safe ∧ ce cx = Echo.safe ∧ ce c = Echo.safe ∧ cx = c := by
  intro p
  have s₁ := join_safe_inv shape (ce cx ⊔ (ce c ⊔ change cx c)) p
  have s₂ := join_safe_inv (ce cx) (ce c ⊔ change cx c) s₁.2
  have s₃ := join_safe_inv (ce c) (change cx c) s₂.2
  exact ⟨s₁.1, s₂.1, s₃.1, change_safe cx c s₃.2⟩

/-- A step that changes carrier is never graded `safe`. -/
theorem promoting_step_not_safe (ce : Carrier → Echo) (shape : Echo) (cx c : Carrier) :
    cx ≠ c → stepEcho ce shape cx c ≠ Echo.safe :=
  fun ne p => ne (safe_grade_sound ce shape cx c p).2.2.2

/-- A step that changes carrier is at least `neutral`. -/
theorem promoting_step_at_least_neutral
    (ce : Carrier → Echo) (shape : Echo) (cx c : Carrier) :
    cx ≠ c → Echo.neutral ≤ stepEcho ce shape cx c := by
  intro ne
  unfold stepEcho
  rw [change_neutral cx c ne]
  exact le_trans' _ _ _
    (le_trans' _ _ _
      (join_upper_right (ce c) Echo.neutral)
      (join_upper_right (ce cx) (ce c ⊔ Echo.neutral)))
    (join_upper_right shape (ce cx ⊔ (ce c ⊔ Echo.neutral)))

-- ============================================================================
-- SECTION 5: ARITHMETIC INVERSION CANNOT UNDO A PROMOTION
-- ============================================================================

section RoundTrip

/-- Undoing `x += v` by `r += neg v` lands in the carrier of `r`. -/
theorem add_inverse_stays_promoted
    {V : Type} (tag : V → Carrier) (add : V → V → Option V) (neg : V → V)
    (add_tag : ∀ x v r, add x v = some r → promote (tag x) (tag v) = some (tag r))
    (neg_tag : ∀ v, tag (neg v) = tag v)
    (x v r x' : V) :
    add x v = some r → add r (neg v) = some x' → tag x' = tag r := by
  intro fwd bwd
  have absorbed : promote (tag r) (tag v) = some (tag r) :=
    promote_absorb (tag x) (tag v) (tag r) (add_tag x v r fwd)
  have undone : promote (tag r) (tag v) = some (tag x') := by
    have h := add_tag r (neg v) x' bwd
    rw [neg_tag v] at h
    exact h
  exact Option.some.inj (undone.symm.trans absorbed)

/-- When the forward step changed the carrier, the value the arithmetic
    inverse restores is not the original. -/
theorem promoted_add_not_restored
    {V : Type} (tag : V → Carrier) (add : V → V → Option V) (neg : V → V)
    (add_tag : ∀ x v r, add x v = some r → promote (tag x) (tag v) = some (tag r))
    (neg_tag : ∀ v, tag (neg v) = tag v)
    (x v r x' : V) :
    add x v = some r → add r (neg v) = some x' → tag x ≠ tag r → x' ≠ x := by
  intro fwd bwd ne eq
  have h := add_inverse_stays_promoted tag add neg add_tag neg_tag x v r x' fwd bwd
  rw [eq] at h
  exact ne h

/-- The same for `x -= v`, undone by `r += v`. -/
theorem sub_inverse_stays_promoted
    {V : Type} (tag : V → Carrier) (add : V → V → Option V) (neg : V → V)
    (add_tag : ∀ x v r, add x v = some r → promote (tag x) (tag v) = some (tag r))
    (neg_tag : ∀ v, tag (neg v) = tag v)
    (x v r x' : V) :
    add x (neg v) = some r → add r v = some x' → tag x' = tag r := by
  intro fwd bwd
  have forward : promote (tag x) (tag v) = some (tag r) := by
    have h := add_tag x (neg v) r fwd
    rw [neg_tag v] at h
    exact h
  have absorbed : promote (tag r) (tag v) = some (tag r) :=
    promote_absorb (tag x) (tag v) (tag r) forward
  exact Option.some.inj ((add_tag r v x' bwd).symm.trans absorbed)

theorem promoted_sub_not_restored
    {V : Type} (tag : V → Carrier) (add : V → V → Option V) (neg : V → V)
    (add_tag : ∀ x v r, add x v = some r → promote (tag x) (tag v) = some (tag r))
    (neg_tag : ∀ v, tag (neg v) = tag v)
    (x v r x' : V) :
    add x (neg v) = some r → add r v = some x' → tag x ≠ tag r → x' ≠ x := by
  intro fwd bwd ne eq
  have h := sub_inverse_stays_promoted tag add neg add_tag neg_tag x v r x' fwd bwd
  rw [eq] at h
  exact ne h

/-- The grade and the semantics agree: a step graded `safe` did not promote,
    so neither theorem above applies to it. -/
theorem safe_step_keeps_carrier {V : Type} (tag : V → Carrier)
    (add : V → V → Option V)
    (ce : Carrier → Echo) (shape : Echo) (x v r : V) :
    add x v = some r →
    stepEcho ce shape (tag x) (tag r) = Echo.safe → tag x = tag r :=
  fun _ p => (safe_grade_sound ce shape (tag x) (tag r) p).2.2.2

end RoundTrip

-- ============================================================================
-- SECTION 6: THE TWO REPORTED CASES
-- ============================================================================
-- Taking the carriers themselves as values (tag = identity, add = promote)
-- satisfies both assumptions above, so those theorems are not vacuous.

/-- `Int += Float`: whatever the inverse step returns, it is not the Int. -/
theorem int_plus_float_not_restored (x' : Carrier) :
    promote .float .float = some x' → x' ≠ .int :=
  fun bwd =>
    promoted_add_not_restored (V := Carrier) id promote id
      (fun _ _ _ p => p) (fun _ => rfl)
      .int .float .float x' rfl bwd (fun h => nomatch h)

/-- `Int += Rational`: likewise, although ℚ itself is an exact group. -/
theorem int_plus_rational_not_restored (x' : Carrier) :
    promote .rational .rational = some x' → x' ≠ .int :=
  fun bwd =>
    promoted_add_not_restored (V := Carrier) id promote id
      (fun _ _ _ p => p) (fun _ => rfl)
      .int .rational .rational x' rfl bwd (fun h => nomatch h)

end Jtv.Promotion
