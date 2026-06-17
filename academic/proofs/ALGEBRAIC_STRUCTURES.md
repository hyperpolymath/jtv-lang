# Algebraic Structures in Julia the Viper

**SPDX-License-Identifier: CC-BY-SA-4.0

This document provides rigorous algebraic foundations for JtV's seven number systems, establishing their group, ring, and field properties, and proving key algebraic theorems.

---

## 1. Abstract Algebra Foundations

### 1.1 Groups

**Definition 1.1 (Group):**
A group (G, ·) is a set G with binary operation · satisfying:
1. **Closure:** ∀a,b ∈ G. a · b ∈ G
2. **Associativity:** ∀a,b,c ∈ G. (a · b) · c = a · (b · c)
3. **Identity:** ∃e ∈ G. ∀a ∈ G. e · a = a · e = a
4. **Inverse:** ∀a ∈ G. ∃a⁻¹ ∈ G. a · a⁻¹ = a⁻¹ · a = e

**Definition 1.2 (Abelian Group):**
A group (G, ·) is Abelian (commutative) if:
∀a,b ∈ G. a · b = b · a

### 1.2 Rings

**Definition 1.3 (Ring):**
A ring (R, +, ×) is a set R with two binary operations satisfying:
1. (R, +) is an Abelian group
2. (R, ×) is a monoid (associative with identity)
3. **Distributivity:** a × (b + c) = (a × b) + (a × c) and (a + b) × c = (a × c) + (b × c)

**Definition 1.4 (Commutative Ring):**
A ring where × is commutative.

**Definition 1.5 (Ring with Unity):**
A ring with multiplicative identity 1 ≠ 0.

### 1.3 Fields

**Definition 1.6 (Field):**
A field (F, +, ×) is a commutative ring with unity where:
∀a ∈ F, a ≠ 0. ∃a⁻¹ ∈ F. a × a⁻¹ = 1

### 1.4 Modules and Vector Spaces

**Definition 1.7 (Module):**
An R-module M over ring R is an Abelian group (M, +) with scalar multiplication:
- R × M → M
- Satisfying distributivity and associativity axioms

**Definition 1.8 (Vector Space):**
An F-module over a field F.

---

## 2. The Integer System (ℤ)

### 2.1 Structure

**Theorem 2.1 (ℤ is a Commutative Ring with Unity):**
(ℤ, +, ×) satisfies all ring axioms with:
- Additive identity: 0
- Multiplicative identity: 1

*Proof:* Standard. The integers under addition form an Abelian group. Multiplication is associative, commutative, and distributes over addition. ∎

### 2.2 Properties in JtV

**Definition 2.1 (JtV Integer Axioms):**
```
∀a,b ∈ Int. a + b = b + a           (commutativity)
∀a,b,c ∈ Int. (a + b) + c = a + (b + c)  (associativity)
∀a ∈ Int. a + 0 = a                  (identity)
∀a ∈ Int. a + (-a) = 0               (inverse)
```

**Theorem 2.2 (JtV Int is Closed Under Addition):**
For all JtV Int values a, b: a + b is a valid Int.

*Proof:* JtV uses arbitrary-precision integers. No overflow occurs. ∎

**Note (Implementation):** When using fixed-precision (i64), checked arithmetic reports overflow errors rather than wrapping.

### 2.3 Divisibility

**Definition 2.2 (Divides):**
a | b ⟺ ∃k ∈ ℤ. b = k × a

**Theorem 2.3 (ℤ is a Unique Factorization Domain):**
Every non-zero non-unit integer factors uniquely into primes (up to order and sign).

**JtV Implementation:**
```jtv
// Check if a divides b (requires Control Language for multiplication)
fn divides(a: Int, b: Int): Bool {
    if a == 0 {
        return b == 0
    }
    remainder = b
    while remainder >= a || remainder <= (-(a)) {
        if remainder >= a {
            remainder = remainder + (-(a))
        } else {
            remainder = remainder + a
        }
    }
    return remainder == 0
}
```

---

## 3. The Rational System (ℚ)

### 3.1 Structure

**Definition 3.1 (Rational Numbers):**
```
ℚ = { p/q | p ∈ ℤ, q ∈ ℤ, q ≠ 0 } / ~
```
where (p₁, q₁) ~ (p₂, q₂) ⟺ p₁ × q₂ = p₂ × q₁

**Theorem 3.1 (ℚ is a Field):**
(ℚ, +, ×) is a field.

*Proof:*
1. Addition: p₁/q₁ + p₂/q₂ = (p₁q₂ + p₂q₁)/(q₁q₂)
2. Additive identity: 0/1
3. Additive inverse: -(p/q) = (-p)/q
4. Multiplication: (p₁/q₁) × (p₂/q₂) = (p₁p₂)/(q₁q₂)
5. Multiplicative identity: 1/1
6. Multiplicative inverse (q ≠ 0): (p/q)⁻¹ = q/p

All field axioms are satisfied. ∎

### 3.2 Properties in JtV

**Definition 3.2 (JtV Rational Representation):**
```
Rational = (numerator: Int, denominator: Int)
```
Invariant: denominator > 0, gcd(|numerator|, denominator) = 1

**Theorem 3.2 (Rational Addition in JtV):**
```
(p₁/q₁) + (p₂/q₂) = (p₁×q₂ + p₂×q₁) / (q₁×q₂)
```
After normalization.

**JtV Implementation:**
```jtv
// Rational addition (requires Control Language for gcd)
fn rat_add(a: Rational, b: Rational): Rational @pure {
    num = a.num * b.den + b.num * a.den
    den = a.den * b.den
    g = gcd(abs(num), den)
    return Rational(num / g, den / g)
}
```

### 3.3 Density

**Theorem 3.3 (ℚ is Dense in ℝ):**
Between any two real numbers, there exists a rational.

**Theorem 3.4 (ℚ is Countable):**
|ℚ| = ℵ₀ (countably infinite)

*Proof:* Cantor's diagonal argument establishes bijection with ℕ. ∎

---

## 4. The Real System (ℝ) - Float Approximation

### 4.1 IEEE 754 Representation

**Definition 4.1 (IEEE 754 Double):**
A 64-bit floating-point number:
- 1 sign bit
- 11 exponent bits
- 52 mantissa bits

Value: (-1)ˢ × 2^(e-1023) × (1 + m/2⁵²)

### 4.2 Algebraic Properties

**Theorem 4.1 (Floats are NOT a Field):**
IEEE 754 floats violate field axioms:

1. **Not associative:**
   ```
   (a + b) + c ≠ a + (b + c)  (in some cases)
   ```
   Example: (1e20 + (-1e20)) + 1 = 1, but 1e20 + ((-1e20) + 1) = 0

2. **Not closed:**
   ```
   1e308 + 1e308 = Infinity
   ```

**Theorem 4.2 (Float Addition is Commutative):**
For all finite floats a, b: a + b = b + a.

*Proof:* IEEE 754 specifies commutative addition. ∎

### 4.3 Error Analysis

**Definition 4.2 (Machine Epsilon):**
ε_mach = 2⁻⁵² ≈ 2.22 × 10⁻¹⁶ for double precision

**Theorem 4.3 (Relative Error Bound):**
For floating-point addition:
```
|(a ⊕ b) - (a + b)| / |a + b| ≤ ε_mach
```
where ⊕ is floating-point addition.

### 4.4 JtV Float Handling

**Definition 4.3 (JtV Float Policy):**
- Uses Rust's f64 (IEEE 754 double)
- NaN propagates
- Infinity on overflow
- No implicit casts to/from Int

---

## 5. The Complex System (ℂ)

### 5.1 Structure

**Definition 5.1 (Complex Numbers):**
```
ℂ = { a + bi | a, b ∈ ℝ, i² = -1 }
```

**Theorem 5.1 (ℂ is a Field):**
(ℂ, +, ×) is a field.

*Proof:*
1. Addition: (a + bi) + (c + di) = (a + c) + (b + d)i
2. Multiplication: (a + bi)(c + di) = (ac - bd) + (ad + bc)i
3. Additive identity: 0 + 0i
4. Multiplicative identity: 1 + 0i
5. Multiplicative inverse: (a + bi)⁻¹ = (a - bi)/(a² + b²)

All axioms verified. ∎

### 5.2 Algebraic Closure

**Theorem 5.2 (Fundamental Theorem of Algebra):**
ℂ is algebraically closed: every non-constant polynomial over ℂ has a root in ℂ.

### 5.3 JtV Complex Operations

**Definition 5.2 (JtV Complex Type):**
```
Complex = (real: Float, imag: Float)
```

**Theorem 5.3 (Complex Addition in JtV Data Language):**
```jtv
// In Data Language (addition only):
(a + bi) + (c + di) = (a + c) + (b + d)i
```

This is expressible purely with addition.

**Note:** Complex multiplication requires Control Language:
```jtv
fn complex_mul(z1: Complex, z2: Complex): Complex {
    real = z1.real * z2.real + (-(z1.imag * z2.imag))
    imag = z1.real * z2.imag + z1.imag * z2.real
    return Complex(real, imag)
}
```

### 5.4 Geometric Interpretation

**Definition 5.3 (Polar Form):**
```
z = r(cos θ + i sin θ) = re^(iθ)
```
where r = |z| = √(a² + b²), θ = arg(z) = atan2(b, a)

**Theorem 5.4 (De Moivre's Formula):**
```
(cos θ + i sin θ)ⁿ = cos(nθ) + i sin(nθ)
```

---

## 6. The Hexadecimal and Binary Systems

### 6.1 Representation

**Definition 6.1 (Positional Notation):**
```
Hex: Σᵢ dᵢ × 16ⁱ  where dᵢ ∈ {0,...,9,A,...,F}
Binary: Σᵢ dᵢ × 2ⁱ  where dᵢ ∈ {0,1}
```

### 6.2 Isomorphism

**Theorem 6.1 (Hex/Binary/Int Isomorphism):**
For non-negative integers:
```
(ℤ≥₀, +) ≅ (Hex, +) ≅ (Binary, +)
```

*Proof:* These are different representations of the same underlying integers. The isomorphism is the identity on values. ∎

### 6.3 JtV Treatment

**Definition 6.2 (JtV Hex/Binary):**
- Hex and Binary are display formats, not distinct types internally
- Arithmetic produces integer results
- Display can be converted

```jtv
x = 0xFF       // Hex literal
y = 0b1010     // Binary literal
z = x + y      // Result: 265 (displayed as Int by default)
```

---

## 7. The Symbolic System

### 7.1 Free Algebra

**Definition 7.1 (Free Abelian Group on Generators):**
The symbolic system is the free Abelian group generated by symbolic terms:
```
Symbolic = FreeAb(Terms)
```

where Terms are symbolic expressions like "x", "sin(θ)", etc.

### 7.2 Operations

**Definition 7.2 (Symbolic Addition):**
```
sym("x") + sym("y") = sym("x + y")
```
(Unevaluated expression tree)

**Definition 7.3 (Symbolic Negation):**
```
-sym("x") = sym("-x")
```

### 7.3 Properties

**Theorem 7.1 (Symbolic Forms an Abelian Group Under Addition):**
Symbolic expressions form an Abelian group.

*Proof:*
1. Closure: Adding expressions produces an expression
2. Associativity: (a + b) + c = a + (b + c) structurally
3. Identity: sym("0") + a = a
4. Inverse: a + (-a) = sym("0")
5. Commutativity: a + b = b + a ∎

### 7.4 Computer Algebra Connection

**Definition 7.4 (Canonical Forms):**
Symbolic expressions should be normalized:
- Collect like terms: x + x → 2x
- Order terms: y + x → x + y
- Simplify: x + 0 → x

**TODO:** Implement full symbolic simplification in JtV.

---

## 8. Number System Hierarchy

### 8.1 Inclusions

```
ℕ ⊂ ℤ ⊂ ℚ ⊂ ℝ ⊂ ℂ
Binary ≅ Hex ≅ ℤ (representation)
Symbolic ⊃ all (abstract representation)
```

### 8.2 JtV Type Coercion Lattice

```
                    Any
                   / | \
                  /  |  \
           Symbolic Complex String
                |   /|
                |  / |
              Float  |
               /    |
              /     |
           Int ----Rational
          / | \
         /  |  \
      Hex Binary Bool
              \
             Never
```

### 8.3 Coercion Rules

**Definition 8.1 (Type Promotion):**
```
Int + Float → Float
Int + Rational → Rational
Int + Complex → Complex
Float + Complex → Complex
any + Symbolic → Symbolic
```

---

## 9. Algebraic Identities

### 9.1 Additive Identities (Data Language)

**Theorem 9.1 (Additive Group Laws in JtV):**
For all JtV numeric types:
```
a + b = b + a                    (commutativity)
(a + b) + c = a + (b + c)        (associativity)
a + 0 = a                        (identity)
a + (-a) = 0                     (inverse)
-(-a) = a                        (involution)
-(a + b) = (-a) + (-b)           (negation distribution)
```

*Proof:* By the algebraic properties of each number system. ∎

### 9.2 Extended Identities (Control Language)

**Theorem 9.2 (Ring Laws):**
With multiplication (Control Language):
```
a × (b + c) = (a × b) + (a × c)  (left distributivity)
(a + b) × c = (a × c) + (b × c)  (right distributivity)
a × 0 = 0                        (zero multiplication)
a × 1 = a                        (multiplicative identity)
```

### 9.3 Field Laws (Where Applicable)

**Theorem 9.3 (Field Division):**
For ℚ, ℝ (non-zero), ℂ (non-zero):
```
a × a⁻¹ = 1   (multiplicative inverse)
a / b = a × b⁻¹   (division as multiplication)
```

---

## 10. Module Theory

### 10.1 Integer Lattice

**Definition 10.1 (ℤⁿ as ℤ-Module):**
The n-dimensional integer lattice ℤⁿ is a free ℤ-module of rank n.

**Application to JtV:** Lists of integers form a ℤ-module:
```jtv
// Vector addition (element-wise)
fn vec_add(a: List<Int>, b: List<Int>): List<Int> {
    result = []
    for i in 0..length(a) {
        result = result ++ [a[i] + b[i]]
    }
    return result
}
```

### 10.2 Vector Spaces Over ℚ and ℝ

**Theorem 10.1 (ℚⁿ and ℝⁿ are Vector Spaces):**
Lists of rationals/floats form vector spaces over their respective fields.

---

## 11. Order Theory

### 11.1 Total Orders

**Definition 11.1 (Total Order):**
A relation ≤ on S is a total order if:
1. Reflexive: a ≤ a
2. Antisymmetric: a ≤ b ∧ b ≤ a → a = b
3. Transitive: a ≤ b ∧ b ≤ c → a ≤ c
4. Total: a ≤ b ∨ b ≤ a

**Theorem 11.1 (Int, Rational, Float are Totally Ordered):**
ℤ, ℚ, ℝ (and their JtV representations) are totally ordered.

### 11.2 Partial Orders

**Theorem 11.2 (Complex Numbers are NOT Ordered):**
There is no total order on ℂ compatible with field operations.

*Proof:* Suppose i > 0. Then i² = -1 > 0. But also -1 < 0. Contradiction.
Suppose i < 0. Then -i > 0, so (-i)² = -1 > 0. Same contradiction.
Suppose i = 0. Then i² = 0 ≠ -1. Contradiction. ∎

### 11.3 Lattice Structure

**Definition 11.2 (Lattice):**
A partially ordered set where every pair has a least upper bound (join) and greatest lower bound (meet).

**Theorem 11.3 (ℤ is a Lattice):**
(ℤ, ≤) with max as join and min as meet forms a lattice.

---

## 12. Abstract Algebra in JtV Programs

### 12.1 Generic Programming

**Definition 12.1 (Algebraic Interface):**
```jtv
// Type class for additive group
interface AdditiveGroup<T> {
    zero: T
    add(a: T, b: T): T @pure
    neg(a: T): T @pure
}

// Instance for Int
instance AdditiveGroup<Int> {
    zero = 0
    add(a, b) = a + b
    neg(a) = -a
}
```

### 12.2 Homomorphism Verification

**Definition 12.2 (Group Homomorphism):**
f : G → H is a homomorphism if:
```
f(a ·_G b) = f(a) ·_H f(b)
```

**JtV Verification:**
```jtv
// Verify f is a homomorphism (for specific inputs)
fn verify_homomorphism<G, H>(
    f: Fn(G) -> H,
    a: G,
    b: G,
    op_g: Fn(G, G) -> G,
    op_h: Fn(H, H) -> H
): Bool {
    left = f(op_g(a, b))
    right = op_h(f(a), f(b))
    return left == right
}
```

---

## 13. Proofs in Lean 4

### 13.1 Algebraic Theorems Formalized

```lean
-- Commutativity of addition
theorem dataExpr_add_comm (e₁ e₂ : DataExpr) (σ : State) :
    evalDataExpr (DataExpr.add e₁ e₂) σ = evalDataExpr (DataExpr.add e₂ e₁) σ := by
  simp [evalDataExpr, Int.add_comm]

-- Associativity of addition
theorem dataExpr_add_assoc (e₁ e₂ e₃ : DataExpr) (σ : State) :
    evalDataExpr (DataExpr.add (DataExpr.add e₁ e₂) e₃) σ =
    evalDataExpr (DataExpr.add e₁ (DataExpr.add e₂ e₃)) σ := by
  simp [evalDataExpr, Int.add_assoc]

-- Identity
theorem dataExpr_add_zero (e : DataExpr) (σ : State) :
    evalDataExpr (DataExpr.add e DataExpr.zero) σ = evalDataExpr e σ := by
  simp [evalDataExpr, DataExpr.zero, Int.add_zero]

-- Inverse
theorem dataExpr_add_neg (e : DataExpr) (σ : State) :
    evalDataExpr (DataExpr.add e (DataExpr.neg e)) σ = 0 := by
  simp [evalDataExpr, Int.add_neg_cancel]
```

### 13.2 Verification Status

| Property | Lean Proof | Status |
|----------|------------|--------|
| Commutativity | ✓ | Verified |
| Associativity | ✓ | Verified |
| Identity | ✓ | Verified |
| Inverse | ✓ | Verified |
| Involution | ✓ | Verified |

---

## 14. Open Problems

### 14.1 Research Questions

1. **Galois Theory:** Can JtV express field extensions meaningfully?
2. **Algebraic Geometry:** Polynomial ideals in symbolic system?
3. **Abstract Algebra Library:** Implement group/ring/field operations generically?

### 14.2 TODO Items

1. Implement symbolic simplification
2. Add arbitrary-precision rationals
3. Create abstract algebra standard library
4. Formalize more algebraic structures in Lean

---

## 15. Summary

JtV's seven number systems provide:

| Type | Algebraic Structure | Key Property |
|------|---------------------|--------------|
| Int | Commutative ring with unity | UFD |
| Float | Approximate field | IEEE 754 |
| Rational | Field | Exact |
| Complex | Field (algebraically closed) | ℂ |
| Hex | ℤ representation | Display format |
| Binary | ℤ representation | Display format |
| Symbolic | Free Abelian group | Unevaluated |

The Data Language's restriction to addition means it operates within the additive group structure of each number system. This is sufficient for many computations and guarantees totality.

---

## References

1. Dummit, D.S., Foote, R.M. (2003). *Abstract Algebra*
2. Lang, S. (2002). *Algebra*
3. Mac Lane, S., Birkhoff, G. (1999). *Algebra*
4. Artin, M. (2011). *Algebra*
5. IEEE 754-2019. Standard for Floating-Point Arithmetic
