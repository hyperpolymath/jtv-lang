<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# Control Language

The Control Language is JtV's **Turing-complete** sublanguage for imperative programming.

## Core Properties

| Property | Description |
|----------|-------------|
| **Turing-complete** | Can compute anything computable |
| **Stateful** | Modifies variable bindings |
| **May not terminate** | While loops can run forever |
| **I/O capable** | Print, input, file operations |

## Statements

### Skip (No-op)

```jtv
// Empty statement - does nothing
skip
```

### Assignment

The **sole join point** between Data and Control:

```jtv
variable = data_expression
```

```
┌─────────────┐         ┌─────────────┐
│   Control   │ ←─────  │    Data     │
│  (variable) │  value  │ (expression)│
└─────────────┘         └─────────────┘
```

### Sequence

```jtv
statement1
statement2
// Or explicitly:
statement1; statement2
```

### Conditional

```jtv
if condition {
    // then branch
}

if condition {
    // then branch
} else {
    // else branch
}

// Nested
if a {
    if b {
        // ...
    }
} else {
    // ...
}
```

### While Loop

```jtv
while condition {
    // body (may execute 0 or more times)
}
```

**Warning**: While loops may not terminate!

```jtv
// This runs forever
while true {
    x = x + 1
}
```

### For Loop

```jtv
// Range-based (guaranteed to terminate for finite ranges)
for i in 0..10 {
    sum = sum + i
}

// Collection-based
for item in collection {
    process(item)
}
```

### Function Definition

```jtv
fn function_name(param1: Type1, param2: Type2): ReturnType {
    // body
    return value
}
```

### Function Call

```jtv
result = function_name(arg1, arg2)
```

### Print

```jtv
print(expression)
print("Hello, " + name)
```

## Semantic Domains

```
State = String → Value
Stmt  : ControlStmt → State → State
```

Control statements transform state.

## Evaluation Rules

### Skip
```
⟨skip, σ⟩ → σ
```

### Assignment
```
⟨x := e, σ⟩ → σ[x ↦ ⟦e⟧(σ)]
```

### Sequence
```
⟨S₁, σ⟩ → σ'    ⟨S₂, σ'⟩ → σ''
─────────────────────────────────
      ⟨S₁; S₂, σ⟩ → σ''
```

### Conditional
```
⟦b⟧(σ) = true     ⟨S₁, σ⟩ → σ'
──────────────────────────────────
  ⟨if b then S₁ else S₂, σ⟩ → σ'

⟦b⟧(σ) = false    ⟨S₂, σ⟩ → σ'
──────────────────────────────────
  ⟨if b then S₁ else S₂, σ⟩ → σ'
```

### While
```
⟦b⟧(σ) = false
─────────────────────────
⟨while b do S, σ⟩ → σ

⟦b⟧(σ) = true    ⟨S, σ⟩ → σ'    ⟨while b do S, σ'⟩ → σ''
──────────────────────────────────────────────────────────
              ⟨while b do S, σ⟩ → σ''
```

## Operational Semantics

### Small-Step (Structural)

```
      ⟨S₁, σ⟩ →₁ ⟨S₁', σ'⟩
─────────────────────────────────────
⟨S₁; S₂, σ⟩ →₁ ⟨S₁'; S₂, σ'⟩

      ⟨S₁, σ⟩ →₁ σ'
─────────────────────────────
⟨S₁; S₂, σ⟩ →₁ ⟨S₂, σ'⟩
```

### Big-Step (Natural)

```
────────────────────────
⟨skip, σ⟩ ⇓ σ

⟨S₁, σ⟩ ⇓ σ'    ⟨S₂, σ'⟩ ⇓ σ''
─────────────────────────────────
      ⟨S₁; S₂, σ⟩ ⇓ σ''
```

## Purity Levels

Control Language functions have purity levels:

| Level | Loops | I/O | Callable from Data |
|-------|-------|-----|-------------------|
| @total | ✗ | ✗ | ✓ |
| @pure | ✓ (bounded) | ✗ | ✓ |
| (none) | ✓ | ✓ | ✗ |

```jtv
// Total - no loops, no I/O, guaranteed termination
@total fn increment(x: Int): Int {
    return x + 1
}

// Pure - may loop but no I/O
@pure fn multiply(a: Int, b: Int): Int {
    result = 0
    for i in 0..b {
        result = result + a
    }
    return result
}

// Impure - can do anything
fn greet(name: String): Unit {
    print("Hello, " + name)
}
```

## Reversible Blocks (v2)

Control Language supports reversible execution:

```jtv
reverse {
    x += 5      // Forward: x = x + 5
    y += x      // Forward: y = y + x
}
// Automatically generates:
// y -= x      // Backward: y = y - x
// x -= 5      // Backward: x = x - 5
```

## Examples

### Factorial

```jtv
fn factorial(n: Int): Int {
    result = 1
    for i in 1..n+1 {
        // Multiply via repeated addition
        temp = 0
        for j in 0..i {
            temp = temp + result
        }
        result = temp
    }
    return result
}
```

### Fibonacci

```jtv
fn fibonacci(n: Int): Int {
    if n <= 1 {
        return n
    }
    a = 0
    b = 1
    for i in 2..n+1 {
        temp = a + b
        a = b
        b = temp
    }
    return b
}
```

### Safe User Input Processing

```jtv
fn processUserInput(input: String): Int {
    // User input is DATA, not CODE
    value = parseInt(input)  // Returns Int, not executable code

    // Even malicious input like "'; DROP TABLE users; --"
    // is just a string that fails parseInt

    if value >= 0 {
        return value + 100
    } else {
        return 0
    }
}
```

## Information Flow

Control Language enforces unidirectional flow:

```
Data Language ──────→ Control Language
              values

Control Language ──✗──→ Data Language
                cannot create
                new expressions
```

## See Also

- [Data Language](./Data-Language.md)
- [Harvard Architecture](./Harvard-Architecture.md)
- [Purity System](../internals/Purity-System.md)
- [Reversible Computing](./Reversible-Computing.md)
