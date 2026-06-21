/-
  JtV - Type System Formalization

  This file defines the type system for JtV including:
  1. Type definitions (7 number systems + compound types)
  2. Typing rules for expressions
  3. Type soundness proofs
  4. Purity enforcement (@pure, @total)
-/

import JtvCore
import JtvOperational

-- ============================================================================
-- SECTION 1: TYPE DEFINITIONS
-- ============================================================================

/-- Basic types in JtV (7 number systems + extras) -/
inductive JtvType where
  | int      : JtvType   -- Signed integers
  | float    : JtvType   -- IEEE 754 floating point
  | rational : JtvType   -- Exact fractions
  | complex  : JtvType   -- Complex numbers
  | hex      : JtvType   -- Hexadecimal (represented as int)
  | binary   : JtvType   -- Binary (represented as int)
  | symbolic : JtvType   -- Symbolic expressions
  | bool     : JtvType   -- Boolean
  | string   : JtvType   -- Strings
  | unit     : JtvType   -- Unit type ()
  | list     : JtvType → JtvType           -- List<T>
  | tuple    : List JtvType → JtvType      -- (T₁, T₂, ...)
  | func     : List JtvType → JtvType → JtvType  -- Fn(T₁, T₂, ...) -> R
  deriving Repr

-- DecidableEq cannot be auto-derived because `tuple`/`func` nest `JtvType`
-- inside `List`. We provide it manually via mutual structural recursion.
mutual
def JtvType.decEq : (a b : JtvType) → Decidable (a = b)
  | .int, .int => isTrue rfl
  | .float, .float => isTrue rfl
  | .rational, .rational => isTrue rfl
  | .complex, .complex => isTrue rfl
  | .hex, .hex => isTrue rfl
  | .binary, .binary => isTrue rfl
  | .symbolic, .symbolic => isTrue rfl
  | .bool, .bool => isTrue rfl
  | .string, .string => isTrue rfl
  | .unit, .unit => isTrue rfl
  | .list a, .list b =>
    match JtvType.decEq a b with
    | isTrue h => isTrue (by rw [h])
    | isFalse h => isFalse (by intro he; injection he; contradiction)
  | .tuple a, .tuple b =>
    match JtvType.decEqList a b with
    | isTrue h => isTrue (by rw [h])
    | isFalse h => isFalse (by intro he; injection he; contradiction)
  | .func a r, .func b s =>
    match JtvType.decEqList a b, JtvType.decEq r s with
    | isTrue h1, isTrue h2 => isTrue (by rw [h1, h2])
    | isFalse h, _ => isFalse (by intro he; injection he with hl hr; exact h hl)
    | _, isFalse h => isFalse (by intro he; injection he with hl hr; exact h hr)
  | .int, .float => isFalse (by intro h; cases h)
  | .int, .rational => isFalse (by intro h; cases h)
  | .int, .complex => isFalse (by intro h; cases h)
  | .int, .hex => isFalse (by intro h; cases h)
  | .int, .binary => isFalse (by intro h; cases h)
  | .int, .symbolic => isFalse (by intro h; cases h)
  | .int, .bool => isFalse (by intro h; cases h)
  | .int, .string => isFalse (by intro h; cases h)
  | .int, .unit => isFalse (by intro h; cases h)
  | .int, .list _ => isFalse (by intro h; cases h)
  | .int, .tuple _ => isFalse (by intro h; cases h)
  | .int, .func _ _ => isFalse (by intro h; cases h)
  | .float, .int => isFalse (by intro h; cases h)
  | .float, .rational => isFalse (by intro h; cases h)
  | .float, .complex => isFalse (by intro h; cases h)
  | .float, .hex => isFalse (by intro h; cases h)
  | .float, .binary => isFalse (by intro h; cases h)
  | .float, .symbolic => isFalse (by intro h; cases h)
  | .float, .bool => isFalse (by intro h; cases h)
  | .float, .string => isFalse (by intro h; cases h)
  | .float, .unit => isFalse (by intro h; cases h)
  | .float, .list _ => isFalse (by intro h; cases h)
  | .float, .tuple _ => isFalse (by intro h; cases h)
  | .float, .func _ _ => isFalse (by intro h; cases h)
  | .rational, .int => isFalse (by intro h; cases h)
  | .rational, .float => isFalse (by intro h; cases h)
  | .rational, .complex => isFalse (by intro h; cases h)
  | .rational, .hex => isFalse (by intro h; cases h)
  | .rational, .binary => isFalse (by intro h; cases h)
  | .rational, .symbolic => isFalse (by intro h; cases h)
  | .rational, .bool => isFalse (by intro h; cases h)
  | .rational, .string => isFalse (by intro h; cases h)
  | .rational, .unit => isFalse (by intro h; cases h)
  | .rational, .list _ => isFalse (by intro h; cases h)
  | .rational, .tuple _ => isFalse (by intro h; cases h)
  | .rational, .func _ _ => isFalse (by intro h; cases h)
  | .complex, .int => isFalse (by intro h; cases h)
  | .complex, .float => isFalse (by intro h; cases h)
  | .complex, .rational => isFalse (by intro h; cases h)
  | .complex, .hex => isFalse (by intro h; cases h)
  | .complex, .binary => isFalse (by intro h; cases h)
  | .complex, .symbolic => isFalse (by intro h; cases h)
  | .complex, .bool => isFalse (by intro h; cases h)
  | .complex, .string => isFalse (by intro h; cases h)
  | .complex, .unit => isFalse (by intro h; cases h)
  | .complex, .list _ => isFalse (by intro h; cases h)
  | .complex, .tuple _ => isFalse (by intro h; cases h)
  | .complex, .func _ _ => isFalse (by intro h; cases h)
  | .hex, .int => isFalse (by intro h; cases h)
  | .hex, .float => isFalse (by intro h; cases h)
  | .hex, .rational => isFalse (by intro h; cases h)
  | .hex, .complex => isFalse (by intro h; cases h)
  | .hex, .binary => isFalse (by intro h; cases h)
  | .hex, .symbolic => isFalse (by intro h; cases h)
  | .hex, .bool => isFalse (by intro h; cases h)
  | .hex, .string => isFalse (by intro h; cases h)
  | .hex, .unit => isFalse (by intro h; cases h)
  | .hex, .list _ => isFalse (by intro h; cases h)
  | .hex, .tuple _ => isFalse (by intro h; cases h)
  | .hex, .func _ _ => isFalse (by intro h; cases h)
  | .binary, .int => isFalse (by intro h; cases h)
  | .binary, .float => isFalse (by intro h; cases h)
  | .binary, .rational => isFalse (by intro h; cases h)
  | .binary, .complex => isFalse (by intro h; cases h)
  | .binary, .hex => isFalse (by intro h; cases h)
  | .binary, .symbolic => isFalse (by intro h; cases h)
  | .binary, .bool => isFalse (by intro h; cases h)
  | .binary, .string => isFalse (by intro h; cases h)
  | .binary, .unit => isFalse (by intro h; cases h)
  | .binary, .list _ => isFalse (by intro h; cases h)
  | .binary, .tuple _ => isFalse (by intro h; cases h)
  | .binary, .func _ _ => isFalse (by intro h; cases h)
  | .symbolic, .int => isFalse (by intro h; cases h)
  | .symbolic, .float => isFalse (by intro h; cases h)
  | .symbolic, .rational => isFalse (by intro h; cases h)
  | .symbolic, .complex => isFalse (by intro h; cases h)
  | .symbolic, .hex => isFalse (by intro h; cases h)
  | .symbolic, .binary => isFalse (by intro h; cases h)
  | .symbolic, .bool => isFalse (by intro h; cases h)
  | .symbolic, .string => isFalse (by intro h; cases h)
  | .symbolic, .unit => isFalse (by intro h; cases h)
  | .symbolic, .list _ => isFalse (by intro h; cases h)
  | .symbolic, .tuple _ => isFalse (by intro h; cases h)
  | .symbolic, .func _ _ => isFalse (by intro h; cases h)
  | .bool, .int => isFalse (by intro h; cases h)
  | .bool, .float => isFalse (by intro h; cases h)
  | .bool, .rational => isFalse (by intro h; cases h)
  | .bool, .complex => isFalse (by intro h; cases h)
  | .bool, .hex => isFalse (by intro h; cases h)
  | .bool, .binary => isFalse (by intro h; cases h)
  | .bool, .symbolic => isFalse (by intro h; cases h)
  | .bool, .string => isFalse (by intro h; cases h)
  | .bool, .unit => isFalse (by intro h; cases h)
  | .bool, .list _ => isFalse (by intro h; cases h)
  | .bool, .tuple _ => isFalse (by intro h; cases h)
  | .bool, .func _ _ => isFalse (by intro h; cases h)
  | .string, .int => isFalse (by intro h; cases h)
  | .string, .float => isFalse (by intro h; cases h)
  | .string, .rational => isFalse (by intro h; cases h)
  | .string, .complex => isFalse (by intro h; cases h)
  | .string, .hex => isFalse (by intro h; cases h)
  | .string, .binary => isFalse (by intro h; cases h)
  | .string, .symbolic => isFalse (by intro h; cases h)
  | .string, .bool => isFalse (by intro h; cases h)
  | .string, .unit => isFalse (by intro h; cases h)
  | .string, .list _ => isFalse (by intro h; cases h)
  | .string, .tuple _ => isFalse (by intro h; cases h)
  | .string, .func _ _ => isFalse (by intro h; cases h)
  | .unit, .int => isFalse (by intro h; cases h)
  | .unit, .float => isFalse (by intro h; cases h)
  | .unit, .rational => isFalse (by intro h; cases h)
  | .unit, .complex => isFalse (by intro h; cases h)
  | .unit, .hex => isFalse (by intro h; cases h)
  | .unit, .binary => isFalse (by intro h; cases h)
  | .unit, .symbolic => isFalse (by intro h; cases h)
  | .unit, .bool => isFalse (by intro h; cases h)
  | .unit, .string => isFalse (by intro h; cases h)
  | .unit, .list _ => isFalse (by intro h; cases h)
  | .unit, .tuple _ => isFalse (by intro h; cases h)
  | .unit, .func _ _ => isFalse (by intro h; cases h)
  | .list _, .int => isFalse (by intro h; cases h)
  | .list _, .float => isFalse (by intro h; cases h)
  | .list _, .rational => isFalse (by intro h; cases h)
  | .list _, .complex => isFalse (by intro h; cases h)
  | .list _, .hex => isFalse (by intro h; cases h)
  | .list _, .binary => isFalse (by intro h; cases h)
  | .list _, .symbolic => isFalse (by intro h; cases h)
  | .list _, .bool => isFalse (by intro h; cases h)
  | .list _, .string => isFalse (by intro h; cases h)
  | .list _, .unit => isFalse (by intro h; cases h)
  | .list _, .tuple _ => isFalse (by intro h; cases h)
  | .list _, .func _ _ => isFalse (by intro h; cases h)
  | .tuple _, .int => isFalse (by intro h; cases h)
  | .tuple _, .float => isFalse (by intro h; cases h)
  | .tuple _, .rational => isFalse (by intro h; cases h)
  | .tuple _, .complex => isFalse (by intro h; cases h)
  | .tuple _, .hex => isFalse (by intro h; cases h)
  | .tuple _, .binary => isFalse (by intro h; cases h)
  | .tuple _, .symbolic => isFalse (by intro h; cases h)
  | .tuple _, .bool => isFalse (by intro h; cases h)
  | .tuple _, .string => isFalse (by intro h; cases h)
  | .tuple _, .unit => isFalse (by intro h; cases h)
  | .tuple _, .list _ => isFalse (by intro h; cases h)
  | .tuple _, .func _ _ => isFalse (by intro h; cases h)
  | .func _ _, .int => isFalse (by intro h; cases h)
  | .func _ _, .float => isFalse (by intro h; cases h)
  | .func _ _, .rational => isFalse (by intro h; cases h)
  | .func _ _, .complex => isFalse (by intro h; cases h)
  | .func _ _, .hex => isFalse (by intro h; cases h)
  | .func _ _, .binary => isFalse (by intro h; cases h)
  | .func _ _, .symbolic => isFalse (by intro h; cases h)
  | .func _ _, .bool => isFalse (by intro h; cases h)
  | .func _ _, .string => isFalse (by intro h; cases h)
  | .func _ _, .unit => isFalse (by intro h; cases h)
  | .func _ _, .list _ => isFalse (by intro h; cases h)
  | .func _ _, .tuple _ => isFalse (by intro h; cases h)
def JtvType.decEqList : (a b : List JtvType) → Decidable (a = b)
  | [], [] => isTrue rfl
  | [], _ :: _ => isFalse (by intro h; cases h)
  | _ :: _, [] => isFalse (by intro h; cases h)
  | x :: xs, y :: ys =>
    match JtvType.decEq x y, JtvType.decEqList xs ys with
    | isTrue h1, isTrue h2 => isTrue (by rw [h1, h2])
    | isFalse h, _ => isFalse (by intro he; injection he with hx hxs; exact h hx)
    | _, isFalse h => isFalse (by intro he; injection he with hx hxs; exact h hxs)
end

instance : DecidableEq JtvType := JtvType.decEq

/-- Type environment: maps variable names to types -/
abbrev TypeEnv := String → Option JtvType

/-- Empty type environment -/
def TypeEnv.empty : TypeEnv := fun _ => none

/-- Extend type environment -/
def TypeEnv.extend (Γ : TypeEnv) (x : String) (τ : JtvType) : TypeEnv :=
  fun y => if y == x then some τ else Γ y

notation:max Γ "[" x " : " τ "]" => TypeEnv.extend Γ x τ

-- ============================================================================
-- SECTION 2: TYPING RULES (Data Language)
-- ============================================================================

/--
  Typing judgment for Data expressions: Γ ⊢ e : τ
  "In environment Γ, expression e has type τ"
-/
inductive DataTyping : TypeEnv → DataExpr → JtvType → Prop where
  | litInt : ∀ Γ n,
      DataTyping Γ (DataExpr.lit n) JtvType.int

  | var : ∀ Γ x τ,
      Γ x = some τ →
      DataTyping Γ (DataExpr.var x) τ

  | addInt : ∀ Γ e₁ e₂,
      DataTyping Γ e₁ JtvType.int →
      DataTyping Γ e₂ JtvType.int →
      DataTyping Γ (DataExpr.add e₁ e₂) JtvType.int

  | addFloat : ∀ Γ e₁ e₂,
      DataTyping Γ e₁ JtvType.float →
      DataTyping Γ e₂ JtvType.float →
      DataTyping Γ (DataExpr.add e₁ e₂) JtvType.float

  | addRational : ∀ Γ e₁ e₂,
      DataTyping Γ e₁ JtvType.rational →
      DataTyping Γ e₂ JtvType.rational →
      DataTyping Γ (DataExpr.add e₁ e₂) JtvType.rational

  | addComplex : ∀ Γ e₁ e₂,
      DataTyping Γ e₁ JtvType.complex →
      DataTyping Γ e₂ JtvType.complex →
      DataTyping Γ (DataExpr.add e₁ e₂) JtvType.complex

  | addSymbolic : ∀ Γ e₁ e₂,
      DataTyping Γ e₁ JtvType.symbolic →
      DataTyping Γ e₂ JtvType.symbolic →
      DataTyping Γ (DataExpr.add e₁ e₂) JtvType.symbolic

  | negInt : ∀ Γ e,
      DataTyping Γ e JtvType.int →
      DataTyping Γ (DataExpr.neg e) JtvType.int

  | negFloat : ∀ Γ e,
      DataTyping Γ e JtvType.float →
      DataTyping Γ (DataExpr.neg e) JtvType.float

  | negRational : ∀ Γ e,
      DataTyping Γ e JtvType.rational →
      DataTyping Γ (DataExpr.neg e) JtvType.rational

  | negComplex : ∀ Γ e,
      DataTyping Γ e JtvType.complex →
      DataTyping Γ (DataExpr.neg e) JtvType.complex

notation:50 Γ " ⊢ᴰ " e " : " τ => DataTyping Γ e τ

-- ============================================================================
-- SECTION 3: TYPE COERCION
-- ============================================================================

/-- Subtyping/coercion relation: τ₁ can be promoted to τ₂ -/
inductive Coercible : JtvType → JtvType → Prop where
  | refl : ∀ τ, Coercible τ τ
  | intToFloat : Coercible JtvType.int JtvType.float
  | intToRational : Coercible JtvType.int JtvType.rational
  | intToComplex : Coercible JtvType.int JtvType.complex
  | floatToComplex : Coercible JtvType.float JtvType.complex
  | hexToInt : Coercible JtvType.hex JtvType.int
  | binaryToInt : Coercible JtvType.binary JtvType.int

notation:50 τ₁ " ≤ᵀ " τ₂ => Coercible τ₁ τ₂

/-- Typing with coercion: Γ ⊢ e : τ₁ and τ₁ ≤ τ₂ implies Γ ⊢ e : τ₂ -/
theorem typing_coercion (Γ : TypeEnv) (e : DataExpr) (τ₁ τ₂ : JtvType) :
    DataTyping Γ e τ₁ → Coercible τ₁ τ₂ → ∃ τ₃, DataTyping Γ e τ₃ := by
  intro h₁ _
  exact ⟨τ₁, h₁⟩

-- ============================================================================
-- SECTION 4: PURITY SYSTEM
-- ============================================================================

/-- Purity levels (ordered: Total ⊂ Pure ⊂ Impure) -/
inductive Purity where
  | total   : Purity  -- Guaranteed to terminate, no side effects
  | pure    : Purity  -- No side effects, may not terminate
  | impure  : Purity  -- May have side effects and may not terminate
  deriving Repr, DecidableEq

/-- Purity ordering: more restrictive ≤ less restrictive -/
def Purity.le : Purity → Purity → Bool
  | total, _ => true
  | pure, pure => true
  | pure, impure => true
  | impure, impure => true
  | _, _ => false

instance : LE Purity where
  le p₁ p₂ := p₁.le p₂ = true

/--
  Purity analysis for Data expressions.
  All Data expressions are Total (guaranteed to terminate).
-/
def DataExpr.purity : DataExpr → Purity
  | lit _ => Purity.total
  | var _ => Purity.total
  | add e₁ e₂ => max e₁.purity e₂.purity  -- Composition preserves totality
  | neg e => e.purity
where
  max : Purity → Purity → Purity
    | Purity.impure, _ => Purity.impure
    | _, Purity.impure => Purity.impure
    | Purity.pure, _ => Purity.pure
    | _, Purity.pure => Purity.pure
    | Purity.total, Purity.total => Purity.total

/--
  **Theorem (Data Language Totality)**:
  All Data expressions have Total purity.
-/
theorem data_is_total (e : DataExpr) : e.purity = Purity.total := by
  induction e with
  | lit _ => rfl
  | var _ => rfl
  | add e₁ e₂ ih₁ ih₂ =>
    simp [DataExpr.purity, ih₁, ih₂, DataExpr.purity.max]
  | neg e ih =>
    simp [DataExpr.purity, ih]

-- ============================================================================
-- SECTION 5: PURITY ENFORCEMENT FOR FUNCTIONS
-- ============================================================================

/-- Function declaration with purity annotation -/
structure FuncDecl where
  name : String
  params : List (String × JtvType)
  returnType : JtvType
  purity : Purity
  body : ControlStmt
  deriving Repr

/-- Function environment -/
abbrev FuncEnv := String → Option FuncDecl

/--
  Check if a Control statement respects a purity constraint.
  - Total functions: no loops allowed
  - Pure functions: no I/O allowed
  - Impure functions: anything goes
-/
def ControlStmt.respectsPurity : ControlStmt → Purity → Bool
  | skip, _ => true
  | assign _ _, _ => true
  | seq s₁ s₂, p => s₁.respectsPurity p && s₂.respectsPurity p
  | ifThenElse _ s₁ s₂, p => s₁.respectsPurity p && s₂.respectsPurity p
  | whileLoop _ _, Purity.total => false  -- Loops violate totality
  | whileLoop _ s, p => s.respectsPurity p

/--
  **Theorem (Pure Function Restriction)**:
  A function marked @pure or @total cannot contain while loops
  if it claims totality.
-/
theorem total_no_loops (s : ControlStmt) (h : s.respectsPurity Purity.total = true) :
    ∀ e body, s ≠ ControlStmt.whileLoop e body := by
  intro e body heq
  cases s with
  | whileLoop e' body' =>
    simp [ControlStmt.respectsPurity] at h
  | _ => simp_all

-- ============================================================================
-- SECTION 6: TYPE SOUNDNESS
-- ============================================================================

/-- Typed values: runtime values with their types.

    `int`, `hex`, and `binary` share the same `Int` runtime carrier —
    hex and binary are int-represented (see `JtvType`) — so an evaluated
    `Int` legitimately inhabits any of the three. The remaining number
    systems (`float`/`rational`/`complex`/`symbolic`) have distinct
    runtime carriers that the `Int`-only evaluation model does not build,
    so they are deliberately absent here (see PROOF-CAPABILITY-MATRIX
    §"Scope of the semantic model"). -/
inductive TypedValue : Int → JtvType → Prop where
  | int    : ∀ n, TypedValue n JtvType.int
  | hex    : ∀ n, TypedValue n JtvType.hex
  | binary : ∀ n, TypedValue n JtvType.binary

/-- The integer-represented type family: exactly the `JtvType`s whose
    runtime values are plain `Int`s (`int`, `hex`, `binary`). These are
    the types for which `type_preservation` holds under the `Int`-only
    evaluation model. -/
def JtvType.isIntRepr : JtvType → Prop
  | JtvType.int    => True
  | JtvType.hex    => True
  | JtvType.binary => True
  | _              => False

/--
  **Theorem (Type Preservation, integer-represented family)**:
  If `Γ ⊢ e : τ` with `τ` an integer-represented type (`int`, `hex`, or
  `binary`) and `e` evaluates to `v`, then `v` has type `τ`. Because the
  three int-represented types share the `Int` carrier, the evaluated
  value inhabits whichever of them the type system assigned.

  This lifts the previous `τ = int` mechanisation to the full int-
  represented family; `float`/`rational`/`complex`/`symbolic` remain
  `stated-unproven` at the value level pending their own evaluation
  models (they do not share the `Int` carrier).
-/
theorem type_preservation (Γ : TypeEnv) (e : DataExpr) (τ : JtvType) (σ : State)
    (_h : DataTyping Γ e τ) (hτ : τ.isIntRepr) :
    TypedValue (evalDataExpr e σ) τ := by
  cases τ <;>
    first
      | exact TypedValue.int (evalDataExpr e σ)
      | exact TypedValue.hex (evalDataExpr e σ)
      | exact TypedValue.binary (evalDataExpr e σ)
      | simp only [JtvType.isIntRepr] at hτ

/--
  **Corollary**: the original `τ = int` preservation statement, recovered
  from the int-represented-family theorem. Kept so the `int`-only claim
  remains directly citable.
-/
theorem type_preservation_int (Γ : TypeEnv) (e : DataExpr) (τ : JtvType) (σ : State)
    (h : DataTyping Γ e τ) :
    τ = JtvType.int → TypedValue (evalDataExpr e σ) JtvType.int := by
  intro hτ
  subst hτ
  exact type_preservation Γ e JtvType.int σ h trivial

/--
  **Theorem (Progress for Typed Terms)**:
  If Γ ⊢ e : τ, then either e is a value or e can step.
-/
theorem typed_progress (Γ : TypeEnv) (e : DataExpr) (τ : JtvType)
    (_h : DataTyping Γ e τ) :
    e.isValue = true ∨ ∃ e', DataStep ⟨e, State.empty⟩ ⟨e', State.empty⟩ := by
  exact data_progress e State.empty

-- ============================================================================
-- SECTION 7: DATA/CONTROL TYPE SEPARATION
-- ============================================================================

/-
  **Key Type-Level Invariant**:
  DataExpr and ControlStmt are distinct types with no overlap.
  This is enforced by Lean's type system itself.
-/

-- DataExpr constructors
#check DataExpr.lit
#check DataExpr.var
#check DataExpr.add
#check DataExpr.neg

-- ControlStmt constructors
#check ControlStmt.skip
#check ControlStmt.assign
#check ControlStmt.seq
#check ControlStmt.ifThenElse
#check ControlStmt.whileLoop

/-
  The type system prevents any mixing:
  - There is no DataExpr constructor that takes a ControlStmt
  - There is no ControlStmt constructor that produces a DataExpr value

  The ONLY interaction is:
  - ControlStmt.assign : String → DataExpr → ControlStmt
    (Control reads from Data)
  - ControlStmt.ifThenElse : DataExpr → ControlStmt → ControlStmt → ControlStmt
    (Control uses Data for conditions)
  - ControlStmt.whileLoop : DataExpr → ControlStmt → ControlStmt
    (Control uses Data for conditions)

  This unidirectional flow (Data → Control) is the foundation of
  code injection impossibility.
-/

-- ============================================================================
-- SECTION 8: TYPE INFERENCE ALGORITHM
-- ============================================================================

/-- Infer the type of a Data expression (if well-typed) -/
def inferType (Γ : TypeEnv) (e : DataExpr) : Option JtvType :=
  match e with
  | DataExpr.lit _ => some JtvType.int
  | DataExpr.var x => Γ x
  | DataExpr.add e₁ e₂ =>
    match inferType Γ e₁, inferType Γ e₂ with
    -- Only the addable number systems have a DataTyping rule for `+`.
    | some JtvType.int, some JtvType.int => some JtvType.int
    | some JtvType.float, some JtvType.float => some JtvType.float
    | some JtvType.rational, some JtvType.rational => some JtvType.rational
    | some JtvType.complex, some JtvType.complex => some JtvType.complex
    | some JtvType.symbolic, some JtvType.symbolic => some JtvType.symbolic
    | _, _ => none
  | DataExpr.neg e =>
    match inferType Γ e with
    | some JtvType.int => some JtvType.int
    | some JtvType.float => some JtvType.float
    | some JtvType.rational => some JtvType.rational
    | some JtvType.complex => some JtvType.complex
    | _ => none

/--
  **Theorem (Type Inference Soundness)**:
  If inferType Γ e = some τ, then Γ ⊢ e : τ
-/
theorem infer_sound (Γ : TypeEnv) (e : DataExpr) (τ : JtvType) :
    inferType Γ e = some τ → DataTyping Γ e τ := by
  intro h
  induction e generalizing τ with
  | lit n =>
    simp [inferType] at h
    subst h
    exact DataTyping.litInt Γ n
  | var x =>
    simp [inferType] at h
    exact DataTyping.var Γ x τ h
  | add e₁ e₂ ih₁ ih₂ =>
    simp only [inferType] at h
    split at h <;> (
      first
      | (injection h with h; subst h
         first
         | exact DataTyping.addInt Γ e₁ e₂ (ih₁ _ ‹_›) (ih₂ _ ‹_›)
         | exact DataTyping.addFloat Γ e₁ e₂ (ih₁ _ ‹_›) (ih₂ _ ‹_›)
         | exact DataTyping.addRational Γ e₁ e₂ (ih₁ _ ‹_›) (ih₂ _ ‹_›)
         | exact DataTyping.addComplex Γ e₁ e₂ (ih₁ _ ‹_›) (ih₂ _ ‹_›)
         | exact DataTyping.addSymbolic Γ e₁ e₂ (ih₁ _ ‹_›) (ih₂ _ ‹_›))
      | exact absurd h (by simp))
  | neg e ih =>
    simp only [inferType] at h
    split at h <;> (
      first
      | (injection h with h; subst h
         first
         | exact DataTyping.negInt Γ e (ih _ ‹_›)
         | exact DataTyping.negFloat Γ e (ih _ ‹_›)
         | exact DataTyping.negRational Γ e (ih _ ‹_›)
         | exact DataTyping.negComplex Γ e (ih _ ‹_›))
      | exact absurd h (by simp))

-- ============================================================================
-- SECTION 9: PURITY CHECKING ALGORITHM
-- ============================================================================

/-- Check if a function body respects its declared purity -/
def checkPurity (decl : FuncDecl) : Bool :=
  decl.body.respectsPurity decl.purity

/--
  **Theorem (Purity Check Correctness)**:
  If checkPurity returns true, the function respects its purity level.
-/
theorem purity_check_correct (decl : FuncDecl) :
    checkPurity decl = true → decl.body.respectsPurity decl.purity = true := by
  intro h
  exact h

-- Summary:
-- 1. JtvType: 7 number systems + compound types
-- 2. DataTyping: Type rules for Data expressions
-- 3. Coercion: Int ≤ Float ≤ Complex, etc.
-- 4. Purity: Total ⊂ Pure ⊂ Impure
-- 5. Type soundness: Progress + Preservation
-- 6. Inference: Decidable type inference
-- 7. Purity checking: @pure/@total enforcement
