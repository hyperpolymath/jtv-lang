/-
  Julia the Viper - Denotational Semantics Core Definitions

  This file defines the semantic domains and evaluation functions for JtV's
  Data Language (the Total, addition-only expression sublanguage).

  The key insight: By grammatically separating Control (Turing-complete) from
  Data (Total), we achieve code injection impossibility as an architectural
  guarantee, not a runtime check.
-/

-- Semantic Domains

/-- State: maps variable names (strings) to integers -/
abbrev State := String → Int

/-- The empty state (all variables map to 0) -/
def State.empty : State := fun _ => 0

/-- State update: σ[x ↦ v] -/
def State.update (σ : State) (x : String) (v : Int) : State :=
  fun y => if y == x then v else σ y

notation:max σ "[" x " ↦ " v "]" => State.update σ x v

-- Abstract Syntax for the Data Language (Total sublanguage)

/-- Terms: the base syntactic elements -/
inductive Term where
  | lit : Int → Term           -- Integer literal n
  | var : String → Term        -- Variable reference x
  deriving Repr, DecidableEq

/-- Expressions: terms composed with addition -/
inductive Expr where
  | term : Term → Expr                    -- Single term t
  | add : Term → Term → Expr              -- Addition t₁ + t₂
  deriving Repr, DecidableEq

-- Evaluation Functions (Denotational Semantics)

/-- Term evaluation: ⟦t⟧ₜ(σ) -/
def evalTerm (t : Term) (σ : State) : Int :=
  match t with
  | Term.lit n => n        -- ⟦n⟧ₜ(σ) = n
  | Term.var x => σ x      -- ⟦x⟧ₜ(σ) = σ(x)

/-- Expression evaluation: ⟦e⟧ₑ(σ) -/
def evalExpr (e : Expr) (σ : State) : Int :=
  match e with
  | Expr.term t => evalTerm t σ                          -- ⟦t⟧ₑ(σ) = ⟦t⟧ₜ(σ)
  | Expr.add t₁ t₂ => evalTerm t₁ σ + evalTerm t₂ σ      -- ⟦t₁ + t₂⟧ₑ(σ) = ⟦t₁⟧ₜ(σ) + ⟦t₂⟧ₜ(σ)

-- Notation for semantic brackets
notation:max "⟦" t "⟧ₜ(" σ ")" => evalTerm t σ
notation:max "⟦" e "⟧ₑ(" σ ")" => evalExpr e σ

-- Extended AST for full Data Language (includes nested additions)

/-- Extended expression syntax supporting arbitrary addition chains -/
inductive DataExpr where
  | lit : Int → DataExpr              -- Integer literal
  | var : String → DataExpr           -- Variable reference
  | add : DataExpr → DataExpr → DataExpr  -- Addition
  | neg : DataExpr → DataExpr         -- Negation (for subtraction via addition)
  deriving Repr, DecidableEq

/-- Size of a DataExpr (for termination proofs) -/
def DataExpr.size : DataExpr → Nat
  | lit _ => 1
  | var _ => 1
  | add e₁ e₂ => 1 + e₁.size + e₂.size
  | neg e => 1 + e.size

/-- Evaluation of extended data expressions -/
def evalDataExpr (e : DataExpr) (σ : State) : Int :=
  match e with
  | DataExpr.lit n => n
  | DataExpr.var x => σ x
  | DataExpr.add e₁ e₂ => evalDataExpr e₁ σ + evalDataExpr e₂ σ
  | DataExpr.neg e => -(evalDataExpr e σ)
-- structural recursion on direct subterms; no `termination_by` needed (an
-- explicit well-founded measure here would require a `decreasing_by` proof and
-- leaves `evalDataExpr` stuck, breaking definitional `rfl` reductions below).

notation:max "⟦" e "⟧ᴰ(" σ ")" => evalDataExpr e σ

-- Control Language AST (for demonstrating separation)

/-- Control statements: the Turing-complete imperative fragment -/
inductive ControlStmt where
  | assign : String → DataExpr → ControlStmt          -- x = e
  | seq : ControlStmt → ControlStmt → ControlStmt     -- s₁ ; s₂
  | ifThenElse : DataExpr → ControlStmt → ControlStmt → ControlStmt  -- if e then s₁ else s₂
  | whileLoop : DataExpr → ControlStmt → ControlStmt  -- while e do s
  | skip : ControlStmt                                 -- no-op
  deriving Repr

-- Note: We intentionally do NOT define a general evaluator for ControlStmt
-- because while loops may not terminate. This structural limitation is the
-- key to our security guarantee.

/-- Single-step execution of Control statements (for finite traces) -/
def execStmt (s : ControlStmt) (σ : State) (fuel : Nat) : Option State :=
  match fuel with
  | 0 => none  -- Out of fuel (potential non-termination)
  | fuel' + 1 =>
    match s with
    | ControlStmt.skip => some σ
    | ControlStmt.assign x e => some (σ[x ↦ evalDataExpr e σ])
    | ControlStmt.seq s₁ s₂ =>
        match execStmt s₁ σ fuel' with
        | some σ' => execStmt s₂ σ' fuel'
        | none => none
    | ControlStmt.ifThenElse e s₁ s₂ =>
        if evalDataExpr e σ ≠ 0
        then execStmt s₁ σ fuel'
        else execStmt s₂ σ fuel'
    | ControlStmt.whileLoop e s =>
        if evalDataExpr e σ ≠ 0
        then match execStmt s σ fuel' with
             | some σ' => execStmt (ControlStmt.whileLoop e s) σ' fuel'
             | none => none
        else some σ

-- Free variables (for state independence analysis)

/-- Free variables in a term -/
def Term.freeVars : Term → List String
  | lit _ => []
  | var x => [x]

/-- Free variables in an expression -/
def Expr.freeVars : Expr → List String
  | term t => t.freeVars
  | add t₁ t₂ => t₁.freeVars ++ t₂.freeVars

/-- Free variables in an extended data expression -/
def DataExpr.freeVars : DataExpr → List String
  | lit _ => []
  | var x => [x]
  | add e₁ e₂ => e₁.freeVars ++ e₂.freeVars
  | neg e => e.freeVars

-- Substitution (for compositional reasoning)

/-- Substitute a variable with a value in a data expression -/
def DataExpr.subst (e : DataExpr) (x : String) (v : Int) : DataExpr :=
  match e with
  | lit n => lit n
  | var y => if y == x then lit v else var y
  | add e₁ e₂ => add (e₁.subst x v) (e₂.subst x v)
  | neg e => neg (e.subst x v)

-- Constants for common expressions

def DataExpr.zero : DataExpr := DataExpr.lit 0
def DataExpr.one : DataExpr := DataExpr.lit 1

-- Example expressions for testing

example : evalTerm (Term.lit 42) State.empty = 42 := rfl
example : evalTerm (Term.var "x") (State.empty["x" ↦ 10]) = 10 := rfl
example : evalExpr (Expr.add (Term.lit 2) (Term.lit 3)) State.empty = 5 := rfl

example : evalDataExpr (DataExpr.add (DataExpr.lit 2) (DataExpr.lit 3)) State.empty = 5 := rfl
example : evalDataExpr (DataExpr.neg (DataExpr.lit 5)) State.empty = -5 := rfl
