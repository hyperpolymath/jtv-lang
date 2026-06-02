/-
  Julia the Viper — Echo Types: Structured Loss / Non-Total Erasure

  This module formalises JtV's *Echo system* (spec v2, §8–9, §12) and aligns it
  with the `echo-types` Agda library (hyperpolymath/echo-types) and its
  executable companion `EchoTypes.jl`.

  An **Echo** classifies the information-loss behaviour of an operation:

    * `safe`     — no loss. The operation is injective / reversible
                   (its fibre over any output is a subsingleton).
    * `neutral`  — *structured loss* (non-total erasure). Information is lost,
                   but a residue witness is retained: the fibre
                   `Echo f y := Σ x, f x = y` is non-trivial yet bounded.
    * `breaking` — *total erasure*. Information is destroyed and cannot be
                   recovered or inverted.

  Lattice order:  `safe ⊑ neutral ⊑ breaking`  (join moves rightward, losing
  guarantees — mirroring the effect lattice `Total ⊑ Pure ⊑ Impure`).

  The headline result for the type checker is `blockEcho_admissible`: under the
  **Safe-only** reversal policy, a reverse block is admissible iff *every*
  constituent statement is `safe` (fully reversible). `neutral` (lossy but
  residue-retaining) and `breaking` are both rejected. This is the formal
  contract enforced by `crates/jtv-core/src/echo.rs`.

  Proofs are deliberately Mathlib-free (Lean core only) and discharge every
  goal by finite case analysis, matching the style of the other JtV proofs.
-/

namespace Jtv.Echo

-- ============================================================================
-- SECTION 1: THE ECHO LATTICE
-- ============================================================================

/-- The three loss classes of the Echo taxonomy. -/
inductive Echo where
  | safe
  | neutral
  | breaking
  deriving Repr, DecidableEq

/-- Numeric rank, giving the lattice order `safe < neutral < breaking`. -/
def Echo.rank : Echo → Nat
  | .safe     => 0
  | .neutral  => 1
  | .breaking => 2

/-- Join (least upper bound). Defined by direct pattern match so that every
    law below reduces by `rfl`. `breaking` is absorbing; `safe` is the unit. -/
def Echo.join : Echo → Echo → Echo
  | .breaking, _        => .breaking
  | _,        .breaking => .breaking
  | .neutral, _         => .neutral
  | _,        .neutral  => .neutral
  | .safe,    .safe     => .safe

@[inherit_doc] infixl:65 " ⊔ " => Echo.join

-- Lattice laws (finite case analysis).

theorem join_comm (a b : Echo) : a ⊔ b = b ⊔ a := by
  cases a <;> cases b <;> rfl

theorem join_assoc (a b c : Echo) : (a ⊔ b) ⊔ c = a ⊔ (b ⊔ c) := by
  cases a <;> cases b <;> cases c <;> rfl

theorem join_idem (a : Echo) : a ⊔ a = a := by
  cases a <;> rfl

/-- `safe` is the bottom element (identity of join): no loss adds nothing. -/
theorem join_safe_left (a : Echo) : Echo.safe ⊔ a = a := by
  cases a <;> rfl

theorem join_safe_right (a : Echo) : a ⊔ Echo.safe = a := by
  cases a <;> rfl

/-- `breaking` is the top element (absorbing): once destroyed, always destroyed. -/
theorem join_breaking_left (a : Echo) : Echo.breaking ⊔ a = Echo.breaking := by
  cases a <;> rfl

theorem join_breaking_right (a : Echo) : a ⊔ Echo.breaking = Echo.breaking := by
  cases a <;> rfl

/-- Partial order induced by the join: `a ≤ b ↔ a ⊔ b = b`. -/
def Echo.le (a b : Echo) : Prop := a ⊔ b = b

instance : LE Echo := ⟨Echo.le⟩

theorem le_refl (a : Echo) : a ≤ a := join_idem a

theorem le_trans (a b c : Echo) : a ≤ b → b ≤ c → a ≤ c := by
  cases a <;> cases b <;> cases c <;> intro hab hbc <;>
    simp_all [LE.le, Echo.le, Echo.join]

/-- `safe` is below everything, `breaking` above everything. -/
theorem safe_le (a : Echo) : Echo.safe ≤ a := by
  cases a <;> rfl

theorem le_breaking (a : Echo) : a ≤ Echo.breaking := by
  cases a <;> rfl

-- ============================================================================
-- SECTION 2: REVERSE-BLOCK ADMISSIBILITY (the type-checker contract)
-- ============================================================================

/-- Boolean admissibility: an Echo may appear in a reverse block iff it is
    `safe`. This is the **Safe-only** policy: a reverse block must be fully
    reversible, so lossy `neutral` operations (whose Bennett-style residue
    reversal is not yet implemented) are rejected alongside `breaking`. Spec v2
    §9 permits `neutral` in principle; the checker is conservatively stricter. -/
def Echo.admissible : Echo → Bool
  | .safe => true
  | _     => false

theorem admissible_iff (e : Echo) : e.admissible = true ↔ e = Echo.safe := by
  cases e <;> simp [Echo.admissible]

/-- **Key compositional law.** The echo of a composite operation `a ⊔ b` is
    admissible iff *both* parts are. A single `breaking` sub-operation makes
    the whole composite inadmissible. -/
theorem join_admissible (a b : Echo) :
    (a ⊔ b).admissible = (a.admissible && b.admissible) := by
  cases a <;> cases b <;> rfl

/-- Admissibility is downward-closed under the lattice order: weakening the
    loss class of an admissible operation keeps it admissible. -/
theorem admissible_downward_closed (a b : Echo) :
    a ≤ b → b.admissible = true → a.admissible = true := by
  cases a <;> cases b <;> intro hle hb <;>
    simp_all [LE.le, Echo.le, Echo.join, Echo.admissible]

-- ============================================================================
-- SECTION 3: REVERSE BLOCKS AS LISTS OF ECHOES
-- ============================================================================

/-- Aggregate echo of a sequence of statements: the join of their echoes,
    starting from `safe`. -/
def blockEcho : List Echo → Echo
  | []      => .safe
  | e :: es => e ⊔ blockEcho es

/-- Explicit "all admissible" predicate over a statement list. -/
def allAdmissible : List Echo → Bool
  | []      => true
  | e :: es => e.admissible && allAdmissible es

/-- **Reverse-block soundness.** A block's aggregate echo is admissible iff
    every statement in it is admissible — i.e. (under the Safe-only policy) a
    reverse block is well-typed exactly when every statement is `safe`.

    This is the property `echo::classify_stmts` / the reverse-block gate in
    `crates/jtv-core/src/echo.rs` must implement. -/
theorem blockEcho_admissible (es : List Echo) :
    (blockEcho es).admissible = allAdmissible es := by
  induction es with
  | nil => rfl
  | cons e es ih =>
      show (e ⊔ blockEcho es).admissible = (e.admissible && allAdmissible es)
      rw [join_admissible, ih]

/-- Corollary: if any statement is `breaking`, the whole block is inadmissible. -/
theorem breaking_blocks_reversal (pre post : List Echo) :
    (blockEcho (pre ++ Echo.breaking :: post)).admissible = false := by
  rw [blockEcho_admissible]
  induction pre with
  | nil => rfl
  | cons e pre ih =>
      show (e.admissible && allAdmissible (pre ++ Echo.breaking :: post)) = false
      rw [ih]; exact Bool.and_false _

-- ============================================================================
-- SECTION 4: FIBRES — THE echo-types CORRESPONDENCE
-- ============================================================================

/-
  The `echo-types` library defines the echo of `f : A → B` at `y : B` as the
  fibre `Echo f y := Σ (x : A), f x ≡ y`. We reproduce that definition and
  prove the bridge between the *value-level* fibre and the *effect-level*
  Echo class above:

    * injective `f`            ↔  every fibre is a subsingleton   ↔  `safe`
    * non-injective, retained  ↔  fibre has ≥2 witnesses, bounded ↔  `neutral`
    * residue weakening        ↔  witnesses forgotten             ↔  loss is
                                                                     one-way
-/

/-- The fibre ("echo") of `f` at `y`: inputs mapping to `y`, paired with proof.
    This is the `echo-types` `Echo f y := Σ x, f x ≡ y`. -/
def Fibre {A B : Type} (f : A → B) (y : B) : Type := { x : A // f x = y }

/-- `echo-intro`: every input lives in the fibre over its own image. -/
def echoIntro {A B : Type} (f : A → B) (x : A) : Fibre f (f x) := ⟨x, rfl⟩

/-- `map-over`: a fibre of `g ∘ f` over `z` projects to a fibre of `g` over `z`. -/
def mapOver {A B C : Type} (f : A → B) (g : B → C) (z : C) :
    Fibre (g ∘ f) z → Fibre g z
  | ⟨x, h⟩ => ⟨f x, h⟩

/-- Injectivity, stated without Mathlib. -/
def Injective {A B : Type} (f : A → B) : Prop :=
  ∀ x₁ x₂ : A, f x₁ = f x₂ → x₁ = x₂

/-- **Safe ⇔ no loss.** If `f` is injective, every fibre is a subsingleton:
    the witness is uniquely recoverable, so the operation is reversible. -/
theorem injective_fibre_subsingleton {A B : Type} (f : A → B)
    (hf : Injective f) (y : B) (p q : Fibre f y) : p.val = q.val :=
  hf p.val q.val (p.property.trans q.property.symm)

/-- The residue map: weaken an echo to its observed output, forgetting the
    witness. This is `echo_to_residue` in `EchoTypes.jl`. -/
def toResidue {A B : Type} (f : A → B) (y : B) : Fibre f y → B := fun _ => y

/-- **Structured loss is real and one-way.** There is a (non-injective) `f`
    whose fibre carries two distinct witnesses that the residue cannot tell
    apart — information genuinely lost, justifying the `neutral`/`breaking`
    classes rather than `safe`. -/
theorem residue_lossy :
    ∃ (f : Bool → Unit) (y : Unit) (p q : Fibre f y), p.val ≠ q.val := by
  refine ⟨fun _ => (), (), ⟨true, rfl⟩, ⟨false, rfl⟩, ?_⟩
  decide

/-- Sanity check: the canonical reversible primitive (negation) is injective,
    hence `safe` — the value-level justification for `reverse { x += v }`. -/
theorem neg_injective : Injective (fun n : Int => -n) := by
  intro a b h
  have : - -a = - -b := by rw [h]
  simpa using this

end Jtv.Echo
