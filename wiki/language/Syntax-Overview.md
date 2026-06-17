<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# JtV Syntax Overview

## Basic Structure

A JtV program consists of **Control Language** statements that may contain **Data Language** expressions.

```jtv
// Comments use double-slash
/* Or block comments */

// Variable declarations and assignments
x = 42
y = 3.14
name = "Julia"

// Control flow
if condition {
    // statements
}

while condition {
    // statements
}

for i in 0..10 {
    // statements
}
```

## Lexical Elements

### Identifiers

Identifiers start with a letter or underscore, followed by letters, digits, or underscores:

```jtv
myVariable
_private
camelCase
snake_case
PascalCase
```

### Literals

#### Integer Literals
```jtv
42          // Decimal
0x2A        // Hexadecimal
0b101010    // Binary
0o52        // Octal
```

#### Floating-Point Literals
```jtv
3.14
2.5e10
1.0e-5
```

#### Rational Literals
```jtv
1/2         // One half
3/4         // Three quarters
22/7        // Approximation of π
```

#### Complex Literals
```jtv
3 + 4i      // 3 + 4i
2i          // Pure imaginary
1 + 0i      // Real as complex
```

#### String Literals
```jtv
"Hello, World!"
"Line 1\nLine 2"
"Tab\there"
```

### Keywords

```
if else while for in
fn return
let mut
true false
print
reverse          // v2
@pure @total     // Function annotations
```

## Operators

### Data Language Operators (Addition Only)

```jtv
// The ONLY arithmetic operator in Data Language
a + b           // Addition
a + -b          // Subtraction via additive inverse
```

### Control Language Operators

```jtv
// Assignment
x = expr

// Comparison (in conditions)
a == b
a != b
a < b
a > b
a <= b
a >= b

// Logical
!a
a && b
a || b
```

## Statements

### Assignment

```jtv
variable = expression
```

The assignment is the **sole join point** between Data and Control languages.

### Conditionals

```jtv
if condition {
    // then branch
}

if condition {
    // then branch
} else {
    // else branch
}
```

### Loops

```jtv
// While loop (may not terminate)
while condition {
    // body
}

// For loop (bounded iteration)
for i in start..end {
    // body using i
}

for item in collection {
    // body using item
}
```

### Functions

```jtv
fn name(param1: Type1, param2: Type2): ReturnType {
    // body
    return value
}

// Pure function (callable from Data Language)
@pure fn add(a: Int, b: Int): Int {
    return a + b
}

// Total function (guaranteed to halt)
@total fn square(x: Int): Int {
    return x + x  // Conceptually x * x via repeated addition
}
```

### Print Statement

```jtv
print(expression)
print("Hello")
print(x + y)
```

## Expressions

### Data Expressions

Data expressions are **guaranteed to terminate**:

```jtv
// Literals
42
3.14
1/2
3 + 4i

// Variables
x
counter

// Addition (the only operator)
a + b
x + y + z

// Function calls (pure functions only)
sqrt(x)
abs(y)
```

### Examples

```jtv
// Calculate sum using repeated addition
@pure fn multiply(a: Int, b: Int): Int {
    result = 0
    for i in 0..b {
        result = result + a
    }
    return result
}

// Safe user input handling
user_value = parseInput(input)
total = base + user_value

// Complex number arithmetic
z1 = 3 + 4i
z2 = 1 + 2i
sum = z1 + z2  // 4 + 6i

// Rational arithmetic (exact)
half = 1/2
third = 1/3
sum = half + third  // 5/6 (exact, no floating-point error)
```

## See Also

- [Data Language](./Data-Language.md)
- [Control Language](./Control-Language.md)
- [Number Systems](./Number-Systems.md)
- [Harvard Architecture](./Harvard-Architecture.md)
