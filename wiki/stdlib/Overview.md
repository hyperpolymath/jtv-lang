<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# JtV Standard Library Overview

The JtV standard library provides essential functions and types organized into modules.

## Module Organization

```
std/
├── core        # Core types and functions (auto-imported)
├── math        # Mathematical operations
├── string      # String manipulation
├── io          # Input/output operations
├── collections # Data structures
├── rational    # Rational number utilities
├── complex     # Complex number utilities
├── convert     # Type conversions
└── testing     # Testing utilities
```

## Core Module (std/core)

Auto-imported, provides fundamental operations.

### Types

```jtv
Int         // Arbitrary-precision integer
Float       // IEEE 754 double
Rational    // Exact fraction
Complex     // Complex number
String      // UTF-8 string
Bool        // Boolean
Unit        // No value
```

### Functions

```jtv
// Identity
@total fn identity<T>(x: T): T

// Comparison (for conditions)
@total fn equal<T>(a: T, b: T): Bool
@total fn notEqual<T>(a: T, b: T): Bool
@total fn lessThan(a: Int, b: Int): Bool
@total fn greaterThan(a: Int, b: Int): Bool

// Boolean operations
@total fn not(b: Bool): Bool
@total fn and(a: Bool, b: Bool): Bool
@total fn or(a: Bool, b: Bool): Bool
```

## Math Module (std/math)

Mathematical operations and constants.

### Constants

```jtv
import std/math

pi: Float      // 3.141592653589793
e: Float       // 2.718281828459045
phi: Float     // 1.618033988749895 (golden ratio)
tau: Float     // 6.283185307179586 (2π)
```

### Arithmetic Functions

```jtv
// Implemented via repeated addition
@pure fn multiply(a: Int, b: Int): Int
@pure fn divide(a: Int, b: Int): (Int, Int)  // (quotient, remainder)
@pure fn modulo(a: Int, b: Int): Int
@pure fn power(base: Int, exp: Int): Int

// Sign operations
@total fn abs(x: Int): Int
@total fn negate(x: Int): Int
@total fn sign(x: Int): Int  // -1, 0, or 1
```

### Number Theory

```jtv
@pure fn gcd(a: Int, b: Int): Int
@pure fn lcm(a: Int, b: Int): Int
@pure fn isPrime(n: Int): Bool
@pure fn factorial(n: Int): Int
@pure fn fibonacci(n: Int): Int
```

### Floating Point

```jtv
@total fn floor(x: Float): Int
@total fn ceil(x: Float): Int
@total fn round(x: Float): Int
@total fn truncate(x: Float): Int

@pure fn sqrt(x: Float): Float
@pure fn exp(x: Float): Float
@pure fn log(x: Float): Float
@pure fn sin(x: Float): Float
@pure fn cos(x: Float): Float
```

## String Module (std/string)

String manipulation functions.

### Properties

```jtv
import std/string

@total fn length(s: String): Int
@total fn isEmpty(s: String): Bool
@total fn charAt(s: String, index: Int): String
```

### Operations

```jtv
// Concatenation (uses + for strings)
@total fn concat(a: String, b: String): String

// Substring
@pure fn substring(s: String, start: Int, end: Int): String
@pure fn take(s: String, n: Int): String
@pure fn drop(s: String, n: Int): String

// Search
@pure fn indexOf(s: String, pattern: String): Int
@pure fn contains(s: String, pattern: String): Bool
@pure fn startsWith(s: String, prefix: String): Bool
@pure fn endsWith(s: String, suffix: String): Bool

// Transformation
@pure fn toUpper(s: String): String
@pure fn toLower(s: String): String
@pure fn trim(s: String): String
@pure fn replace(s: String, old: String, new: String): String

// Split and join
@pure fn split(s: String, delimiter: String): [String]
@pure fn join(parts: [String], delimiter: String): String
```

## IO Module (std/io)

Input/output operations (impure).

```jtv
import std/io

// Console I/O
fn print(value: Any): Unit
fn println(value: Any): Unit
fn read(): String
fn readLine(): String

// File I/O
fn readFile(path: String): String
fn writeFile(path: String, content: String): Unit
fn appendFile(path: String, content: String): Unit
fn fileExists(path: String): Bool
```

## Collections Module (std/collections)

Data structures and algorithms.

### Arrays

```jtv
import std/collections

@total fn arrayLength<T>(arr: [T]): Int
@total fn arrayGet<T>(arr: [T], index: Int): T
@total fn arraySet<T>(arr: [T], index: Int, value: T): [T]

@pure fn arrayMap<T, U>(arr: [T], f: (T) -> U): [U]
@pure fn arrayFilter<T>(arr: [T], predicate: (T) -> Bool): [T]
@pure fn arrayFold<T, U>(arr: [T], init: U, f: (U, T) -> U): U

@pure fn arrayConcat<T>(a: [T], b: [T]): [T]
@pure fn arrayReverse<T>(arr: [T]): [T]
@pure fn arraySort<T>(arr: [T]): [T]
```

### Maps (Associative Arrays)

```jtv
struct Map<K, V>

@total fn mapNew<K, V>(): Map<K, V>
@total fn mapGet<K, V>(m: Map<K, V>, key: K): Option<V>
@total fn mapPut<K, V>(m: Map<K, V>, key: K, value: V): Map<K, V>
@total fn mapRemove<K, V>(m: Map<K, V>, key: K): Map<K, V>
@total fn mapContains<K, V>(m: Map<K, V>, key: K): Bool
@pure fn mapKeys<K, V>(m: Map<K, V>): [K]
@pure fn mapValues<K, V>(m: Map<K, V>): [V]
```

### Sets

```jtv
struct Set<T>

@total fn setNew<T>(): Set<T>
@total fn setAdd<T>(s: Set<T>, value: T): Set<T>
@total fn setRemove<T>(s: Set<T>, value: T): Set<T>
@total fn setContains<T>(s: Set<T>, value: T): Bool
@pure fn setUnion<T>(a: Set<T>, b: Set<T>): Set<T>
@pure fn setIntersect<T>(a: Set<T>, b: Set<T>): Set<T>
@pure fn setDifference<T>(a: Set<T>, b: Set<T>): Set<T>
```

## Rational Module (std/rational)

Rational number utilities.

```jtv
import std/rational

// Construction
@total fn rational(num: Int, den: Int): Rational
@total fn fromInt(n: Int): Rational

// Properties
@total fn numerator(r: Rational): Int
@total fn denominator(r: Rational): Int
@total fn isInteger(r: Rational): Bool

// Operations (addition is built-in)
@pure fn multiply(a: Rational, b: Rational): Rational
@pure fn divide(a: Rational, b: Rational): Rational
@pure fn reciprocal(r: Rational): Rational

// Conversion
@total fn toFloat(r: Rational): Float
@pure fn simplify(r: Rational): Rational
```

## Complex Module (std/complex)

Complex number utilities.

```jtv
import std/complex

// Construction
@total fn complex(real: Float, imag: Float): Complex
@total fn fromReal(r: Float): Complex
@total fn fromPolar(magnitude: Float, angle: Float): Complex

// Properties
@total fn real(c: Complex): Float
@total fn imag(c: Complex): Float
@total fn magnitude(c: Complex): Float
@total fn angle(c: Complex): Float

// Operations (addition is built-in)
@pure fn multiply(a: Complex, b: Complex): Complex
@pure fn divide(a: Complex, b: Complex): Complex
@total fn conjugate(c: Complex): Complex

// Conversion
@total fn toPolar(c: Complex): (Float, Float)
```

## Convert Module (std/convert)

Type conversion utilities.

```jtv
import std/convert

// To String
@total fn toString(value: Any): String
@total fn intToString(n: Int): String
@total fn floatToString(f: Float): String
@total fn rationalToString(r: Rational): String

// From String
@pure fn parseInt(s: String): Option<Int>
@pure fn parseFloat(s: String): Option<Float>
@pure fn parseRational(s: String): Option<Rational>

// Between numeric types
@total fn intToFloat(n: Int): Float
@total fn intToRational(n: Int): Rational
@total fn floatToInt(f: Float): Int  // truncates
@total fn rationalToFloat(r: Rational): Float

// Radix conversion
@pure fn toHex(n: Int): String
@pure fn toBinary(n: Int): String
@pure fn fromHex(s: String): Option<Int>
@pure fn fromBinary(s: String): Option<Int>
```

## Testing Module (std/testing)

Testing utilities.

```jtv
import std/testing

// Assertions
fn assert(condition: Bool): Unit
fn assertEqual<T>(actual: T, expected: T): Unit
fn assertNotEqual<T>(actual: T, expected: T): Unit
fn assertApprox(actual: Float, expected: Float, tolerance: Float): Unit

// Test definition
fn test(name: String, body: () -> Unit): Unit
fn describe(name: String, body: () -> Unit): Unit

// Property-based testing
fn property(name: String, generator: () -> Any, predicate: (Any) -> Bool): Unit
fn forAll<T>(gen: Generator<T>, prop: (T) -> Bool): Unit
```

## Using the Standard Library

### Importing Modules

```jtv
// Import entire module
import std/math

// Use qualified names
result = math.multiply(3, 4)

// Import specific items
import std/math { multiply, divide }

// Now unqualified
result = multiply(3, 4)
```

### Auto-imported Core

The `std/core` module is automatically imported:

```jtv
// These work without explicit import
x = identity(42)
b = not(true)
```

## See Also

- [Math Module](./Math.md)
- [String Module](./String.md)
- [Collections Module](./Collections.md)
- [IO Module](./IO.md)
