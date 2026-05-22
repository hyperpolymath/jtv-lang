# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

# Julia-the-Viper Type System Specification

**Version:** 1.0.0
**Date:** 2026-03-14

---

## 1. Type Language

```
τ ::= Int | Float | Rational | Complex        numeric types
    | Hex | Binary                             representation types
    | Symbolic                                 symbolic math
    | Bool                                     boolean (control language)
    | String                                   text
    | Unit                                     empty tuple
    | List[τ]                                  homogeneous list
    | (τ₁, …, τₙ)                              heterogeneous tuple
    | (τ₁, …, τₙ) → τᵣ                        function type
    | Any                                      inference placeholder
```

### 1.1 Seven Number Systems

JtV uniquely supports seven distinct number systems, all first-class:

| Type | Literal Syntax | Domain |
|------|---------------|--------|
| Int | `42`, `-7` | ℤ (arbitrary precision) |
| Float | `3.14`, `-2.718` | ℝ (IEEE 754) |
| Rational | `3/4`, `22/7` | ℚ (exact) |
| Complex | `3+4i`, `2.5-1.2i` | ℂ |
| Hex | `0xFF`, `0x1A` | Hexadecimal representation of ℤ |
| Binary | `0b1010`, `0b11` | Binary representation of ℤ |
| Symbolic | `x`, `π`, `e` | Symbolic algebra |

---

## 2. Harvard Architecture Type Separation

### 2.1 Data Language Types

The data language is **total** (provably terminating) and uses only
addition-only operators. Type checking is via `infer_data_expr()`:

```
    ──────────────────  [D-Int]     ──────────────────  [D-Float]
    Γ ⊢_D n : Int                  Γ ⊢_D f : Float

    ──────────────────  [D-Rational]   ──────────────────  [D-Complex]
    Γ ⊢_D p/q : Rational             Γ ⊢_D a+bi : Complex

    Γ ⊢_D e₁ : τ₁     Γ ⊢_D e₂ : τ₂
    ────────────────────────────────────────  [D-Add]
    Γ ⊢_D e₁ + e₂ : widen(τ₁, τ₂)

    Γ ⊢_D e : τ     τ numeric
    ──────────────────────────────  [D-Neg]
    Γ ⊢_D -e : τ                  (unary negation OK)
```

**No subtraction operator** in data language — unary negation + addition
achieves the same result while preserving the addition-only invariant.

### 2.2 Control Language Types

The control language is Turing-complete and handles all I/O, loops, and
conditionals. Type checking is via `check_control_stmt()`:

```
    Γ ⊢_C cond : Bool     Γ ⊢_C body : ()
    ──────────────────────────────────────────  [C-If]
    Γ ⊢_C if cond { body } : ()

    Γ ⊢_C cond : Bool     Γ ⊢_C body : ()
    ──────────────────────────────────────────  [C-While]
    Γ ⊢_C while cond { body } : ()

    Γ ⊢_C iter : List[τ]     Γ, x: τ ⊢_C body : ()
    ──────────────────────────────────────────────────  [C-For]
    Γ ⊢_C for x in iter { body } : ()

    Γ ⊢_D e : τ
    ──────────────────────────────  [C-Print]
    Γ ⊢_C print(e) : ()           (I/O is control-only)
```

### 2.3 Cross-Language Typing

Data expressions can appear inside control statements:

```
    Γ ⊢_D e : τ
    ──────────────────────────────  [C-DataEmbed]
    Γ ⊢_C let x = e : ()           (data value assigned in control)

    Γ ⊢_D e₁ : τ₁     Γ ⊢_D e₂ : τ₂
    ──────────────────────────────────────  [C-CondExpr]
    Γ ⊢_C e₁ < e₂ : Bool               (comparison in control context)
```

---

## 3. Numeric Tower (Widening)

JtV has an implicit numeric tower for mixed-type arithmetic:

```
widen : Type × Type → Type

widen(Int, Float) = Float
widen(Int, Rational) = Rational
widen(Int, Complex) = Complex
widen(Float, Complex) = Complex
widen(Hex, Int) = Int
widen(Binary, Int) = Int
widen(τ, τ) = τ
```

Widening is always to a strictly more general type. No implicit narrowing.

---

## 4. Purity System

### 4.1 Purity Levels

```
Purity ::= Total | Pure | Impure

Total ⊂ Pure ⊂ Impure     (strict ordering)
```

### 4.2 Purity Typing Rules

```
    f declared @total     body has no loops, no I/O
    ────────────────────────────────────────────────  [P-Total]
    Γ ⊢ f : Total

    f declared @pure     body has no I/O (loops OK)
    ────────────────────────────────────────────────  [P-Pure]
    Γ ⊢ f : Pure

    otherwise
    ──────────────────  [P-Impure]
    Γ ⊢ f : Impure
```

### 4.3 Purity Propagation

```
    Γ ⊢ f : Total     Γ ⊢ g : Total
    ────────────────────────────────────  [P-Call-Total]
    Γ ⊢ f(g(x)) : Total

    Γ ⊢ f : Total     Γ ⊢ g : Pure
    ────────────────────────────────────  [P-Call-Demote]
    Γ ⊢ f(g(x)) : Pure                 (purity is meet of callee purities)

    Γ ⊢ f : Total     Γ ⊢ g : Unknown
    ──────────────────────────────────────  [P-Call-Unknown]
    Γ ⊢ f(g(x)) : Impure               (unknown callee → assume impure)
```

### 4.4 Statement Purity

```
    ──────────────────────────  [P-Print]
    Γ ⊢ print(e) : Impure     (I/O contaminates)

    ──────────────────────────────  [P-While]
    Γ ⊢ while … { … } : Pure     (loops are at most Pure, not Total)

    ──────────────────────────────  [P-For-Bounded]
    Γ ⊢ for x in [1..n] { … } : Pure  (conservative; could be Total for bounded)
```

---

## 5. Reversible Computing Types

### 5.1 Reverse Block

```
    ∀stmt ∈ body: has_inverse(stmt)
    ────────────────────────────────────────  [T-Reverse]
    Γ ⊢ reverse { body } : ()
```

### 5.2 Invertible Statements

| Statement | Inverse |
|-----------|---------|
| `x += val` | `x -= val` |
| `x -= val` | `x += val` |
| `let x = e` | unbind x |
| `swap(x, y)` | `swap(y, x)` |

Non-invertible statements (e.g., `x = e` without history) are rejected
inside reverse blocks.

---

## 6. Type Inference Algorithm

Two-pass approach:

**Pass 1 (Collection):**
```
∀ function f:
  extract parameter types (from annotations or Any)
  extract return type (from annotation or Any)
  extract purity level (@total, @pure, or default)
  register in Γ
```

**Pass 2 (Checking):**
```
∀ function f:
  Γ' = Γ ∪ {params}
  for each statement in body:
    check_control_stmt(Γ', stmt)
  verify purity matches declaration
```

---

## 7. Properties

1. **Harvard safety:** Data expressions cannot perform I/O or loop — guaranteed by type separation.
2. **Data totality:** All data expressions terminate (no recursion, no loops, addition-only).
3. **Numeric coercion soundness:** Widening preserves mathematical meaning (Int ⊂ Float ⊂ Complex).
4. **Purity monotonicity:** Calling a less-pure function demotes the caller's purity level.
5. **Reverse block safety:** Only invertible statements allowed inside reverse blocks.
6. **Code injection impossibility:** Harvard separation makes it grammatically impossible for data to execute as control — this is a type-level guarantee, not just a convention.
