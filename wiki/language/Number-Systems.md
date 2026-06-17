<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# Number Systems

JtV supports **7 number systems** with automatic type coercion and exact arithmetic where possible.

## Overview

| Type | Syntax | Example | Precision |
|------|--------|---------|-----------|
| Integer | Decimal | `42` | Arbitrary |
| Float | Decimal with `.` | `3.14` | IEEE 754 |
| Rational | `a/b` | `1/3` | Exact |
| Complex | `a + bi` | `3 + 4i` | Float components |
| Hexadecimal | `0x...` | `0xFF` | Integer |
| Binary | `0b...` | `0b1010` | Integer |
| Symbolic | Identifier | `pi`, `e` | Symbolic |

## Integer (Int)

Arbitrary-precision integers with no overflow.

```jtv
// Literals
x = 42
y = -17
big = 12345678901234567890

// Operations
sum = x + y
diff = x + -y  // Subtraction via additive inverse

// No overflow!
huge = 999999999999999999999999999 + 1  // Works correctly
```

### Properties
- Arbitrary precision (limited only by memory)
- Exact arithmetic
- Closed under addition

## Float

IEEE 754 double-precision floating-point numbers.

```jtv
// Literals
pi = 3.14159265358979
e = 2.71828
small = 1.0e-10
large = 1.5e100

// Scientific notation
avogadro = 6.022e23

// Operations
sum = pi + e  // 5.859872...
```

### Properties
- 64-bit IEEE 754
- Approximate arithmetic
- Subject to rounding errors

### Caveats

```jtv
// Floating-point precision issues
x = 0.1 + 0.2  // 0.30000000000000004, not 0.3

// Use Rational for exact decimal arithmetic
y = 1/10 + 2/10  // Exactly 3/10
```

## Rational

Exact fractions as numerator/denominator pairs.

```jtv
// Literals
half = 1/2
third = 1/3
quarter = 1/4

// Automatic simplification
six_eighths = 6/8  // Stored as 3/4

// Exact arithmetic
sum = 1/2 + 1/3  // Exactly 5/6, not 0.833...

// No precision loss
x = 1/3 + 1/3 + 1/3  // Exactly 1/1
```

### Properties
- Numerator and denominator are arbitrary-precision integers
- Always stored in lowest terms
- Exact arithmetic (no rounding)
- Closed under addition

### When to Use Rational

```jtv
// Financial calculations (exact)
price = 19/100 + 99/100  // $0.19 + $0.99 = $1.18 exactly

// Scientific fractions
probability = 1/6 + 1/6 + 1/6  // 1/2 exactly

// Avoiding float errors
result = 1/10 + 1/10 + 1/10  // 3/10 exactly
```

## Complex

Complex numbers with real and imaginary parts.

```jtv
// Literals
z1 = 3 + 4i
z2 = -2 + 1i
pure_imaginary = 5i
real_as_complex = 7 + 0i

// Addition
sum = z1 + z2  // (3 + -2) + (4 + 1)i = 1 + 5i

// Conjugate via additive inverse of imaginary part
// z* = a - bi = a + (-b)i
conj = 3 + -4i  // Conjugate of 3 + 4i
```

### Properties
- Real and imaginary parts are floats
- Closed under addition
- Useful for signal processing, physics, engineering

### Complex Arithmetic Examples

```jtv
// Signal processing
signal1 = 1 + 2i
signal2 = 3 + 4i
combined = signal1 + signal2  // 4 + 6i

// Quantum state representation (simplified)
state_0 = 1 + 0i
state_1 = 0 + 0i
superposition = state_0 + state_1  // Simplified example
```

## Hexadecimal (Hex)

Base-16 integers, commonly used for colors, addresses, bit patterns.

```jtv
// Literals (case-insensitive)
red = 0xFF0000
green = 0x00ff00
blue = 0x0000FF

// Mixed case
mixed = 0xDeAdBeEf

// Arithmetic (result is integer)
sum = 0x10 + 0x20  // 48 (0x30)
```

### Properties
- Syntactic sugar for integers
- Useful for low-level programming
- Display preference can be set

### Use Cases

```jtv
// Colors (RGB)
white = 0xFFFFFF
black = 0x000000
gray = 0x808080

// Memory addresses
base_addr = 0x1000
offset = 0x100
target = base_addr + offset  // 0x1100

// Bit patterns
mask = 0xFF00
value = 0x1234
// Note: JtV uses addition only, bitwise ops via libraries
```

## Binary (Bin)

Base-2 integers for bit-level visualization.

```jtv
// Literals
byte = 0b11111111      // 255
nibble = 0b1010        // 10
flag = 0b1             // 1

// Arithmetic
sum = 0b1010 + 0b0101  // 15 (0b1111)
```

### Properties
- Syntactic sugar for integers
- Useful for understanding bit patterns
- Educational tool for binary arithmetic

### Use Cases

```jtv
// Permissions (Unix-style concept)
read = 0b100     // 4
write = 0b010    // 2
execute = 0b001  // 1
rwx = read + write + execute  // 7 (0b111)

// Flags
flag_a = 0b0001
flag_b = 0b0010
combined = flag_a + flag_b  // 0b0011
```

## Symbolic

Named mathematical constants and symbolic expressions.

```jtv
// Built-in constants
circle_ratio = pi       // π ≈ 3.14159...
euler = e               // e ≈ 2.71828...
golden = phi            // φ ≈ 1.61803...

// Symbolic arithmetic (preserved symbolically when possible)
two_pi = pi + pi        // 2π (may be evaluated or kept symbolic)
```

### Properties
- Named constants with known values
- May be evaluated or kept symbolic
- Useful for mathematical expressions

### Built-in Symbols

| Symbol | Value | Description |
|--------|-------|-------------|
| `pi` | 3.14159... | Circle constant |
| `e` | 2.71828... | Euler's number |
| `phi` | 1.61803... | Golden ratio |
| `inf` | ∞ | Positive infinity |
| `nan` | NaN | Not a number |

## Type Coercion

JtV automatically promotes types in mixed expressions:

```
Coercion Hierarchy:

        Complex
        ↗     ↖
    Float     (future: Complex Rational)
    ↗   ↖
  Int   Rational
  ↑
Hex/Binary
```

### Coercion Rules

```jtv
// Int + Float → Float
x = 5 + 3.14        // 8.14 (Float)

// Int + Rational → Rational
y = 5 + 1/2         // 11/2 (Rational)

// Float + Rational → Float
z = 0.5 + 1/3       // 0.833... (Float)

// Any + Complex → Complex
w = 5 + (3 + 4i)    // 8 + 4i (Complex)

// Hex/Binary → Int (always)
h = 0xFF + 1        // 256 (Int)
b = 0b1010 + 5      // 15 (Int)
```

### Explicit Type Annotation

```jtv
// Force specific type
x: Float = 5        // 5.0
y: Rational = 3     // 3/1
z: Complex = 7      // 7 + 0i
```

## Comparison Table

| Feature | Int | Float | Rational | Complex | Hex | Binary |
|---------|-----|-------|----------|---------|-----|--------|
| Exact | ✓ | ✗ | ✓ | ✗ | ✓ | ✓ |
| Arbitrary precision | ✓ | ✗ | ✓ | ✗ | ✓ | ✓ |
| Closed under + | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Negative values | ✓ | ✓ | ✓ | ✓ | ✓* | ✓* |
| Fractional values | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ |

*Hex and Binary are stored as signed integers

## See Also

- [Data Language](./Data-Language.md)
- [Type System](../internals/Type-System.md)
- [Arithmetic Operations](./Arithmetic.md)
