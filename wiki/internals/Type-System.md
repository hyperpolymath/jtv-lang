<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# JtV Type System

The JtV type system provides static type checking with automatic inference and coercion for the 7 number systems.

## Type Hierarchy

```
                    Any
                     │
         ┌──────────┴──────────┐
         │                     │
      Number                String
         │
    ┌────┴────┐
    │         │
  Real    Complex
    │
  ┌─┴──────────┐
  │            │
Integer    Rational
  │
  ├── Int
  ├── Hex
  └── Binary

Float ──── approximates ───► Rational
```

## Primitive Types

### Integer Types

```jtv
x: Int = 42           // Arbitrary-precision integer
h: Hex = 0xFF         // Hexadecimal (Int representation)
b: Binary = 0b1010    // Binary (Int representation)
```

### Floating-Point

```jtv
f: Float = 3.14       // IEEE 754 double precision
```

### Rational

```jtv
r: Rational = 1/3     // Exact fraction
```

### Complex

```jtv
c: Complex = 3 + 4i   // Complex number
```

### String

```jtv
s: String = "hello"   // UTF-8 string
```

### Unit

```jtv
u: Unit = ()          // No value (void equivalent)
```

### Boolean

```jtv
t: Bool = true
f: Bool = false
```

## Type Inference

JtV infers types from context:

```jtv
x = 42              // Inferred: Int
y = 3.14            // Inferred: Float
z = 1/2             // Inferred: Rational
w = 3 + 4i          // Inferred: Complex

sum = x + y         // Inferred: Float (Int + Float → Float)
```

### Inference Rules

```
Γ ⊢ n : Int           (integer literal)
Γ ⊢ r/s : Rational    (rational literal)
Γ ⊢ f.d : Float       (float literal)
Γ ⊢ a + bi : Complex  (complex literal)

Γ ⊢ x : τ             (if Γ(x) = τ)

Γ ⊢ e₁ : τ₁   Γ ⊢ e₂ : τ₂   τ = coerce(τ₁, τ₂)
──────────────────────────────────────────────
            Γ ⊢ e₁ + e₂ : τ
```

## Type Coercion

Automatic promotion follows the coercion hierarchy:

### Coercion Rules

```
Int → Float
Int → Rational
Int → Complex
Float → Complex
Rational → Float
Rational → Complex
Hex → Int
Binary → Int
```

### Examples

```jtv
// Int + Float → Float
result = 5 + 3.14        // 8.14 : Float

// Int + Rational → Rational
result = 5 + 1/2         // 11/2 : Rational

// Anything + Complex → Complex
result = 5 + (3 + 4i)    // 8+4i : Complex
```

### Coercion Function

```
coerce : Type × Type → Type

coerce(Int, Int) = Int
coerce(Int, Float) = Float
coerce(Float, Int) = Float
coerce(Int, Rational) = Rational
coerce(Rational, Int) = Rational
coerce(Float, Rational) = Float
coerce(Rational, Float) = Float
coerce(_, Complex) = Complex
coerce(Complex, _) = Complex
```

## Type Annotations

Explicit type annotations:

```jtv
// Variable annotation
x: Int = 42
y: Float = 3.14

// Function signature
fn add(a: Int, b: Int): Int {
    return a + b
}

// Generic functions (future)
fn identity<T>(x: T): T {
    return x
}
```

## Compound Types

### Tuples

```jtv
point: (Int, Int) = (3, 4)
rgb: (Int, Int, Int) = (255, 128, 0)
```

### Arrays

```jtv
numbers: [Int] = [1, 2, 3, 4, 5]
matrix: [[Float]] = [[1.0, 0.0], [0.0, 1.0]]
```

### Records (Structs)

```jtv
struct Point {
    x: Int,
    y: Int
}

p: Point = Point { x: 3, y: 4 }
```

## Purity Types

Functions have purity levels that affect where they can be called:

```
Purity Hierarchy:
Total ⊂ Pure ⊂ Impure

@total → No loops, no I/O, guaranteed termination
@pure  → May loop, no I/O
(none) → May loop, may do I/O
```

### Purity Type Rules

```
@total fn f(): T    ⊢ f callable in Data context
@pure fn g(): T     ⊢ g callable in Data context
fn h(): T           ⊢ h callable only in Control context
```

### Examples

```jtv
@total fn increment(x: Int): Int {
    return x + 1
}

@pure fn multiply(a: Int, b: Int): Int {
    result = 0
    for i in 0..b {
        result = result + a
    }
    return result
}

fn readAndProcess(): Int {
    input = read()  // I/O
    return parseInt(input)
}

// In Data context:
data_expr = increment(5) + multiply(3, 4)  // OK

// ERROR: Cannot call impure function in Data context
// data_expr = readAndProcess()  // Type error!
```

## Type Checking Algorithm

### Bidirectional Type Checking

JtV uses bidirectional type checking:

1. **Inference mode**: Compute type from expression
2. **Checking mode**: Verify expression has expected type

```
infer : Context × Expr → Type
check : Context × Expr × Type → Bool

infer(Γ, n) = Int                    // Integer literal
infer(Γ, x) = Γ(x)                   // Variable lookup
infer(Γ, e₁ + e₂) = coerce(infer(Γ, e₁), infer(Γ, e₂))

check(Γ, e, τ) = (infer(Γ, e) = τ) ∨ coercible(infer(Γ, e), τ)
```

### Type Checking Phases

1. **Parse**: Build untyped AST
2. **Resolve**: Resolve names and imports
3. **Infer**: Compute types bottom-up
4. **Check**: Verify type constraints
5. **Purity**: Check purity annotations

## Error Messages

### Type Mismatch

```
Error[E101]: type mismatch
  --> main.jtv:5:10
   |
 5 | x: Int = 3.14
   |          ^^^^ expected Int, found Float
   |
   = help: use explicit conversion: floor(3.14)
```

### Purity Violation

```
Error[E201]: purity violation in @pure function
  --> main.jtv:8:5
   |
 8 |     print(x)
   |     ^^^^^^^^ I/O operation not allowed in @pure function
   |
   = note: @pure functions cannot perform I/O
   = help: remove @pure annotation or remove print call
```

### Coercion Failure

```
Error[E103]: cannot coerce types
  --> main.jtv:12:10
   |
12 | s: String = 42
   |             ^^ cannot coerce Int to String
   |
   = help: use toString(42) for explicit conversion
```

## Formal Type Rules

### Data Expressions

```
────────────────── (T-Int)
Γ ⊢ n : Int

────────────────── (T-Float)
Γ ⊢ f : Float

──────────────────── (T-Rational)
Γ ⊢ a/b : Rational

────────────────────── (T-Complex)
Γ ⊢ a + bi : Complex

Γ(x) = τ
────────────── (T-Var)
Γ ⊢ x : τ

Γ ⊢ e₁ : τ₁    Γ ⊢ e₂ : τ₂    τ = coerce(τ₁, τ₂)
─────────────────────────────────────────────────── (T-Add)
                  Γ ⊢ e₁ + e₂ : τ

Γ ⊢ f : (τ₁,...,τₙ) → τ    Γ ⊢ eᵢ : τᵢ    purity(f) ∈ {Total, Pure}
──────────────────────────────────────────────────────────────────── (T-Call-Data)
                       Γ ⊢ f(e₁,...,eₙ) : τ
```

### Control Statements

```
Γ ⊢ e : τ    τ' = coerce(Γ(x), τ)
──────────────────────────────────── (T-Assign)
        Γ ⊢ x = e : Unit

Γ ⊢ b : Bool    Γ ⊢ S₁ : Unit    Γ ⊢ S₂ : Unit
──────────────────────────────────────────────── (T-If)
          Γ ⊢ if b { S₁ } else { S₂ } : Unit

Γ ⊢ b : Bool    Γ ⊢ S : Unit
─────────────────────────────── (T-While)
    Γ ⊢ while b { S } : Unit
```

## See Also

- [Number Systems](../language/Number-Systems.md)
- [Purity System](./Purity-System.md)
- [Type Checker Implementation](./Type-Checker.md)
