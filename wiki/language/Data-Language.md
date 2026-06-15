<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# Data Language

The Data Language is JtV's **decidable**, **total** sublanguage for pure computation.

## Core Properties

| Property | Description |
|----------|-------------|
| **Total** | Every expression terminates |
| **Pure** | No side effects |
| **Decidable** | Halting is guaranteed |
| **Addition-only** | Single arithmetic operation |

## Why Addition Only?

Addition is the **universal operation**:

1. **Works across all number systems**: integers, floats, rationals, complex, hex, binary, symbolic
2. **Enables reversibility**: `a + b` reverses to `a - b` (v2 reverse blocks)
3. **With loops**: Turing-complete (can compute anything)
4. **Without loops**: Total (always halts)

### Implementing Other Operations

```jtv
// Subtraction: add the additive inverse
x + -5        // x - 5

// Multiplication: repeated addition
@pure fn multiply(a: Int, b: Int): Int {
    result = 0
    for i in 0..b {
        result = result + a
    }
    return result
}

// Division: repeated subtraction (with remainder)
@pure fn divide(a: Int, b: Int): (Int, Int) {
    quotient = 0
    remainder = a
    while remainder >= b {
        remainder = remainder + -b
        quotient = quotient + 1
    }
    return (quotient, remainder)
}
```

## Grammar

```ebnf
DataExpr ::= Term (('+') Term)*
Term     ::= Factor
Factor   ::= Number | Variable | '(' DataExpr ')' | FunctionCall
Number   ::= Integer | Float | Rational | Complex | Hex | Binary | Symbolic
```

## Semantic Domain

```
State = String → ℤ
⟦ · ⟧ : DataExpr → State → ℤ
```

The evaluation function maps expressions to integers given a state.

## Evaluation Rules

### Literals

```
⟦ n ⟧(σ) = n                     (integer literal)
⟦ r/s ⟧(σ) = Rational(r, s)      (rational literal)
⟦ a + bi ⟧(σ) = Complex(a, b)    (complex literal)
```

### Variables

```
⟦ x ⟧(σ) = σ(x)                  (variable lookup)
```

### Addition

```
⟦ e₁ + e₂ ⟧(σ) = ⟦ e₁ ⟧(σ) + ⟦ e₂ ⟧(σ)
```

## Totality Guarantee

**Theorem**: For all Data expressions `e` and states `σ`, evaluation `⟦e⟧(σ)` terminates.

**Proof**: By structural induction on `e`:
- **Base case (literal)**: Immediate evaluation
- **Base case (variable)**: Single lookup
- **Inductive case (addition)**: Both subexpressions terminate by IH, addition terminates

This is formalized in Lean 4:

```lean
theorem dataExpr_totality (e : DataExpr) (σ : State) :
    ∃ (n : Int), evalDataExpr e σ = n := by
  induction e with
  | lit n => exact ⟨n, rfl⟩
  | var x => exact ⟨σ x, rfl⟩
  | add e₁ e₂ ih₁ ih₂ =>
    obtain ⟨n₁, h₁⟩ := ih₁
    obtain ⟨n₂, h₂⟩ := ih₂
    exact ⟨n₁ + n₂, by simp [evalDataExpr, h₁, h₂]⟩
```

## Pure Functions in Data Context

Only **@pure** or **@total** functions can be called from Data expressions:

```jtv
// ✓ Allowed - pure function
@pure fn square(x: Int): Int {
    return x + x  // No loops, no I/O
}

data_expr = square(5) + 3  // OK

// ✗ Not allowed - impure function
fn readInput(): String {
    return input()  // I/O operation
}

data_expr = readInput()  // ERROR: Cannot call impure function in Data context
```

## Number System Coercion

The Data Language supports automatic coercion between compatible types:

```
Int → Float → Complex
Int → Rational → Complex
```

```jtv
// Automatic promotion
x = 5 + 3.14        // Int + Float → Float
y = 1/2 + 0.25      // Rational + Float → Float
z = 3 + 2i          // Int + Complex → Complex
```

## Referential Transparency

Data expressions are **referentially transparent**: the same expression with the same state always produces the same result.

```jtv
// These are equivalent
a = x + y
b = x + y
// a == b is always true (assuming no intervening state change)

// Can substitute equals for equals
result = (x + y) + (x + y)
// Equivalent to:
temp = x + y
result = temp + temp
```

## Security Implications

The Data Language's restrictions provide security guarantees:

1. **No code injection**: Data cannot become code
2. **Predictable execution**: Always terminates
3. **No side effects**: Cannot modify external state
4. **Sandboxed**: Cannot access files, network, etc.

## See Also

- [Control Language](./Control-Language.md)
- [Number Systems](./Number-Systems.md)
- [Harvard Architecture](./Harvard-Architecture.md)
- [Purity Enforcement](../internals/Purity-System.md)
