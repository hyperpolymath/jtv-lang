/-
  Julia the Viper — Echo Types: Structured Loss / Non-Total Erasure

  This module formalises JtV's *Echo system* (spec v2, §8–9, §12) and aligns it
  with the `echo-types` Agda library (hyperpolymath/echo-types) and its
  executable companion `EchoTypes.jl`.

  PRINCIPLE: Echo is about *structured, proof-relevant loss* — information may
  be collapsed, weakened, sampled, projected, or degraded, but the
  residue / provenance / lineage of that loss is still representable. The fibre
  `Echo f y := Σ x, f x = y` is used not as a generic Σ-type but as the carrier
  of *retained-loss lineage*: it records which inputs were collapsed into `y`.

  An **Echo** classifies the loss behaviour of an operation:

    * `safe`     — no loss. The operation is injective / reversible; its fibre
                   over any output is a subsingleton, so lineage is trivial.
    * `neutral`  — *structured loss*. Information is collapsed, but the fibre
                   retains the loss lineage (non-trivial yet bounded).
    * `breaking` — *total erasure*. Lineage is destroyed; cannot be inverted.

  Lattice order:  `safe ⊑ neutral ⊑ breaking`  (join moves rightward, losing
  guarantees — mirroring the effect lattice `Total ⊑ Pure ⊑ Impure`).

  The headline result for the type checker is `blockEcho_admissible`: under the
  **Safe-only** reversal policy, a reverse block is admissible iff *every*
  constituent statement is `safe` (fully reversible). `neutral` (lossy but
  residue-retaining) and `breaking` are both rejected. This is the formal
  contract enforced by `crates/jtv-core/src/echo.rs`.

  The companion result `blockEcho_admissibleWithResidue` (Section 2b) formalises
  the v2 `reversible { } -> tok` policy: `neutral` is *also* admissible there,
  because its loss lineage is retained as a token (Bennett-style); only
  `breaking` is rejected. The operational justification is
  `rev_forward_backward_with_token` in `JtvTheorems`.

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
-- SECTION 2b: RESIDUE-RETAINING ADMISSIBILITY (the reversible{}->tok contract)
-- ============================================================================

/-- Boolean admissibility under the **residue-retaining** (Bennett) policy: an
    Echo may appear in a `reversible { } -> tok` block iff it is not `breaking`.
    Unlike `admissible` (Safe-only), this admits `neutral`: a lossy op is
    reversible when its loss lineage (the retained token / residue) is kept. The
    operational justification is `rev_forward_backward_with_token` in
    `JtvTheorems`. `breaking` (total erasure) is still rejected — no token can
    recover destroyed lineage. (ADR-0007 D5/D6; spec v2 §9.) -/
def Echo.admissibleWithResidue : Echo → Bool
  | .breaking => false
  | _         => true

theorem admissibleWithResidue_iff (e : Echo) :
    e.admissibleWithResidue = true ↔ e ≠ Echo.breaking := by
  cases e <;> simp [Echo.admissibleWithResidue]

/-- The Safe-only policy is strictly stronger: anything admissible under
    `reverse { }` is admissible under `reversible { } -> tok`. -/
theorem admissible_implies_admissibleWithResidue (e : Echo) :
    e.admissible = true → e.admissibleWithResidue = true := by
  cases e <;> simp [Echo.admissible, Echo.admissibleWithResidue]

/-- `neutral` is exactly the capability the token form adds: rejected by
    Safe-only, accepted with a retained residue. -/
theorem neutral_residue_only :
    Echo.neutral.admissible = false ∧ Echo.neutral.admissibleWithResidue = true :=
  ⟨rfl, rfl⟩

/-- Compositional law for the residue policy (mirrors `join_admissible`). -/
theorem join_admissibleWithResidue (a b : Echo) :
    (a ⊔ b).admissibleWithResidue
      = (a.admissibleWithResidue && b.admissibleWithResidue) := by
  cases a <;> cases b <;> rfl

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

-- ----------------------------------------------------------------------------
-- Residue-policy block soundness (companion to the Safe-only results above,
-- placed here because it consumes `blockEcho` from this section).
-- ----------------------------------------------------------------------------

/-- Block-level "all admissible with residue" (residue/Bennett policy). -/
def allAdmissibleWithResidue : List Echo → Bool
  | []      => true
  | e :: es => e.admissibleWithResidue && allAdmissibleWithResidue es

/-- **Residue-block soundness.** A `reversible { } -> tok` block is well-typed
    iff every statement is non-`breaking` — the contract `echo.rs` enforces for
    the token-carrying reversal form. -/
theorem blockEcho_admissibleWithResidue (es : List Echo) :
    (blockEcho es).admissibleWithResidue = allAdmissibleWithResidue es := by
  induction es with
  | nil => rfl
  | cons e es ih =>
      show (e ⊔ blockEcho es).admissibleWithResidue
            = (e.admissibleWithResidue && allAdmissibleWithResidue es)
      rw [join_admissibleWithResidue, ih]

/-- Even with residue retention, a single `breaking` op blocks reversal. -/
theorem breaking_blocks_residual_reversal (pre post : List Echo) :
    (blockEcho (pre ++ Echo.breaking :: post)).admissibleWithResidue = false := by
  rw [blockEcho_admissibleWithResidue]
  induction pre with
  | nil => rfl
  | cons e pre ih =>
      show (e.admissibleWithResidue
            && allAdmissibleWithResidue (pre ++ Echo.breaking :: post)) = false
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
  -- `h : (fun n => -n) a = (fun n => -n) b`; the lambda applications are
  -- definitionally `-a` / `-b`, so re-state `h` in beta-reduced form (the
  -- defeq coercion does the beta step `rw` could not see) and finish with
  -- linear integer reasoning.
  have h' : -a = -b := h
  omega

-- ============================================================================
-- SECTION 5: FUNCTION EFFECTS — Echo × Epistemic (ADR-0009 D1+D3)
-- ============================================================================

/-
  ADR-0009 lifts Echo to a first-class *function effect* and adds a parallel
  *Epistemic* effect (what a function reveals about its inputs). The effect a
  function carries is a point in the product lattice `Echo × Epistemic`, and
  composition across calls is the (idempotent, commutative) join. This section
  mechanises the Epistemic lattice and the product, mirroring the Echo lattice
  of SECTION 1, and pins the lattice laws that make composition well-defined.

  Constructor names map to the Rust `epistemic.rs` / ADR-0009 D2 grades:
  `hidden = Opaque`, `bounded = Partial`, `full = Transparent`
  (lowercase `opaque` / `partial` are Lean keywords, hence the rename).
-/

/-- The epistemic grade (ADR-0009 D2): how much a function reveals about inputs.
    `hidden ⊑ bounded ⊑ full`  (= Opaque ⊑ Partial ⊑ Transparent). -/
inductive Epistemic where
  | hidden
  | bounded
  | full
  deriving Repr, DecidableEq

/-- Join (worst-case revelation): `full` absorbing, `hidden` the unit. -/
def Epistemic.join : Epistemic → Epistemic → Epistemic
  | .full,    _        => .full
  | _,        .full    => .full
  | .bounded, _        => .bounded
  | _,        .bounded => .bounded
  | .hidden,  .hidden  => .hidden

theorem epi_join_comm (a b : Epistemic) : a.join b = b.join a := by
  cases a <;> cases b <;> rfl

theorem epi_join_assoc (a b c : Epistemic) :
    (a.join b).join c = a.join (b.join c) := by
  cases a <;> cases b <;> cases c <;> rfl

theorem epi_join_idem (a : Epistemic) : a.join a = a := by
  cases a <;> rfl

/-- The function effect row (ADR-0009 D3): a point in `Echo × Epistemic`,
    composed componentwise. -/
structure FunctionEffect where
  echo : Echo
  epi : Epistemic
  deriving Repr, DecidableEq

/-- Componentwise join of two function effects. -/
def FunctionEffect.join (x y : FunctionEffect) : FunctionEffect :=
  { echo := x.echo ⊔ y.echo, epi := x.epi.join y.epi }

/-- **Composition is a commutative, idempotent join** (ADR-0009): the effect of
    a composite is the join of its parts', and these three laws make that fold
    independent of the order and repetition of calls — so `g ∘ f` carries
    `effect f ⊔ effect g` however the calls are arranged. -/
theorem feffect_join_comm (a b : FunctionEffect) : a.join b = b.join a := by
  cases a; cases b
  simp [FunctionEffect.join, join_comm, epi_join_comm]

theorem feffect_join_assoc (a b c : FunctionEffect) :
    (a.join b).join c = a.join (b.join c) := by
  cases a; cases b; cases c
  simp [FunctionEffect.join, join_assoc, epi_join_assoc]

theorem feffect_join_idem (a : FunctionEffect) : a.join a = a := by
  cases a
  simp [FunctionEffect.join, join_idem, epi_join_idem]

-- ============================================================================
-- SECTION 6: NUMBER-SYSTEM STRATIFICATION BY ADDITIVE ALGEBRA
-- ============================================================================

/-
  The reversibility tier of `+` — and hence of `reverse { x += v }` — over a
  given number system is NOT a per-system stipulation. It is *forced* by the
  additive algebra of the carrier. This section stratifies JtV's numeric
  systems by where they sit in the additive-algebra tower and reads off the
  Echo tier mechanically.

  Additive-algebra tower (only the levels reversal can distinguish):

    abelianGroup   associative + commutative + EXACT inverses
                   → reverse-add is total and exact            → safe
    approxGroup    commutative with identity, but NON-associative
                   and inverses only approximate (rounding)
                   → reverse-add recoverable but lossy          → neutral
    nonGroup       `+` not invertible at all (e.g. tropical
                   min-plus): no reverse exists                 → breaking

  This tower maps 1:1 onto the Echo lattice of SECTION 1 (safe/neutral/breaking)
  — which is why three levels suffice: a finer carrier tower
  (Monoid ⊂ CancellativeMonoid ⊂ Group) would collapse onto the same 3-valued
  Echo codomain.

  Headline collapse: `hex` and `binary` are NOT distinct algebras — they are
  *encodings of ℤ* (cf. the `JtvType` constructor comments "represented as
  int"), so they share `int`'s `abelianGroup` instance. The seven surface
  systems thus reduce to TWO inhabited algebra classes here; `nonGroup` is
  reserved for future non-invertible systems (spec D6).

  Mirrors `crates/jtv-core/src/number.rs` (the runtime carriers) and the
  numeric constructors of `Jtv.JtvType`.
-/

/-- The additive-algebra class of a carrier, at the granularity reversal cares
    about. One constructor per Echo tier. -/
inductive NumAlgebra where
  | abelianGroup   -- exact inverses; reverse-add total & exact
  | approxGroup    -- non-associative / rounding; reverse-add lossy
  | nonGroup       -- `+` not invertible; no reverse
  deriving Repr, DecidableEq

/-- JtV's addable number systems (the numeric constructors of `JtvType`). -/
inductive NumSystem where
  | int | rational | complex | symbolic | hex | binary | float
  deriving Repr, DecidableEq

/-- Where each system sits in the additive-algebra tower. `hex`/`binary` map to
    the SAME class as `int` because they are encodings of ℤ, not new algebras. -/
def NumSystem.algebra : NumSystem → NumAlgebra
  | .int      => .abelianGroup
  | .rational => .abelianGroup
  | .complex  => .abelianGroup
  | .symbolic => .abelianGroup
  | .hex      => .abelianGroup   -- ℤ in hex clothing
  | .binary   => .abelianGroup   -- ℤ in binary clothing
  | .float    => .approxGroup    -- IEEE-754: non-associative, rounding

/-- The Echo tier *forced* by an additive algebra. This is the stratification
    law: the reversal tier is a function of the algebra, never a free choice. -/
def NumAlgebra.echo : NumAlgebra → Echo
  | .abelianGroup => .safe
  | .approxGroup  => .neutral
  | .nonGroup     => .breaking

/-- A number system's reversal tier = the Echo of its additive algebra. -/
def NumSystem.echo (s : NumSystem) : Echo := s.algebra.echo

-- The stratification is total and the map is a genuine function of the algebra,
-- so every result below is closed by definitional reduction / finite decision.

/-- **The hex/binary collapse**: the integer *encodings* carry int's tier
    exactly — encoding is not algebra. -/
theorem hex_binary_collapse :
    NumSystem.echo .hex = NumSystem.echo .int
  ∧ NumSystem.echo .binary = NumSystem.echo .int := ⟨rfl, rfl⟩

/-- The exact abelian groups are all `safe` (`+` is fully reversible). -/
theorem exact_groups_safe :
    NumSystem.echo .int = .safe
  ∧ NumSystem.echo .rational = .safe
  ∧ NumSystem.echo .complex = .safe
  ∧ NumSystem.echo .symbolic = .safe := ⟨rfl, rfl, rfl, rfl⟩

/-- **Float is not `safe`.** IEEE-754 addition is non-associative, so its
    reversal is lossy; the stratification lifts it to `neutral`. This is the one
    place the surface "addition-only ⇒ reversible" slogan is *qualified* by the
    carrier — and exactly why float's Echo grade sits above `safe`. -/
theorem float_not_safe : NumSystem.echo .float ≠ .safe := by decide

theorem float_neutral : NumSystem.echo .float = .neutral := rfl

/-- **No current system is `breaking`.** Every inhabited carrier is at least an
    approximate group, so the `breaking` tier is presently empty — it is held in
    reserve for a future non-invertible (`nonGroup`) system. -/
theorem no_current_system_breaks (s : NumSystem) : NumSystem.echo s ≠ .breaking := by
  cases s <;> decide

/-- **Stratification meets the reversal policies.** Reading the algebra off the
    carrier decides admissibility under both policies of SECTION 1/2b:
    `safe` (Safe-only reversal) holds iff the carrier is an exact abelian group;
    non-`breaking` (the `reversible { } -> tok` token policy, which also admits
    `neutral`) holds iff the carrier is not a `nonGroup`. -/
theorem reversal_tier_by_algebra (s : NumSystem) :
    (NumSystem.echo s = .safe ↔ s.algebra = .abelianGroup)
  ∧ (NumSystem.echo s ≠ .breaking ↔ s.algebra ≠ .nonGroup) := by
  cases s <;> decide

end Jtv.Echo
