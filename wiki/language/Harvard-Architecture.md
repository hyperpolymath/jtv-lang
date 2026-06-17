<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# Harvard Architecture in JtV

## Overview

Julia the Viper implements a **Harvard Architecture** at the language level, separating code into two distinct sublanguages:

```
┌─────────────────────────────────────────────────────────────┐
│                    JtV PROGRAM                               │
├─────────────────────────┬───────────────────────────────────┤
│    DATA LANGUAGE        │      CONTROL LANGUAGE             │
│    (Decidable)          │      (Turing-Complete)            │
├─────────────────────────┼───────────────────────────────────┤
│ • Addition only         │ • Assignments                     │
│ • Guaranteed halting    │ • If/else                         │
│ • No side effects       │ • While/for loops                 │
│ • Pure evaluation       │ • Function calls                  │
│                         │ • I/O operations                  │
└─────────────────────────┴───────────────────────────────────┘
              │                         ▲
              │    JOIN POINT           │
              │    (assignment)         │
              └─────────────────────────┘
```

## Why Harvard Architecture?

### Traditional Languages are Vulnerable

In Python, JavaScript, PHP, and most languages, strings can become code:

```python
# Python - VULNERABLE
user_input = "'; import os; os.system('rm -rf /'); '"
eval(f"x = {user_input}")  # Executes malicious code!
```

```javascript
// JavaScript - VULNERABLE
const userInput = "'); alert('hacked'); //";
eval("process('" + userInput + "')");  // XSS attack!
```

### JtV is Architecturally Secure

In JtV, data can NEVER become code:

```jtv
// JtV - SECURE
user_input = "'; malicious; '"  // This is just a string
x = user_input + 42             // Concatenation, not execution
// There is NO way to execute user_input as code
```

## The Two Languages

### Data Language (Total)

The Data Language is **decidable** - every expression is guaranteed to terminate:

```jtv
// Data expressions - always halt
x = 5 + 3                    // Integer addition
y = 1/2 + 1/3                // Rational addition
z = (3 + 4i) + (1 + 2i)      // Complex addition
result = a + b + c           // Chain of additions
```

**Properties:**
- Addition is the ONLY arithmetic operation
- No loops, no recursion
- Evaluation always terminates
- No side effects
- Referentially transparent

### Control Language (Turing-Complete)

The Control Language provides full computational power:

```jtv
// Control statements - may not terminate
x = 5                        // Assignment
if x > 0 {                   // Conditional
    print(x)
}
while x > 0 {                // Loop (may not halt)
    x = x + -1
}
for i in 0..10 {             // Bounded loop
    sum = sum + i
}
```

**Properties:**
- Turing-complete (can compute anything)
- May not terminate (while loops)
- Can have side effects (print, I/O)
- Controls program flow

## The Join Point

The **only** place where Data and Control interact is the **assignment statement**:

```
variable = expression
   ▲           ▲
   │           │
Control     Data
 side       side
```

This unidirectional flow (Data → Control) is the key security property:
- Data can flow INTO control variables
- Control CANNOT create new Data expressions
- There is no `eval()`, no code generation from strings

## Formal Guarantees

The Harvard Architecture provides these mathematically proven guarantees:

### 1. Code Injection Impossibility
```
Theorem: For any DataExpr e, there exists no function
         f : DataExpr → ControlStmt in the grammar.
```
Data cannot become code because there's no grammatical production for it.

### 2. Totality of Data
```
Theorem: For all expressions e and states σ,
         evaluation ⟦e⟧(σ) terminates.
```
Every Data expression is guaranteed to halt.

### 3. Unidirectional Flow
```
Theorem: All join points flow Data → Control,
         never Control → Data.
```
The information flow is strictly one-way.

## Aspect-Oriented Language Development

JtV implements **AOLD** (Aspect-Oriented Language Development):

| Aspect | Traditional AOP | JtV AOLD |
|--------|-----------------|----------|
| Weaving | Runtime/Compile-time | Grammar-level |
| Join Points | Method calls, field access | Assignment only |
| Enforcement | Convention | Type system + Grammar |
| Security | Best practice | Mathematical guarantee |

## Practical Implications

### What You CAN Do

```jtv
// Arithmetic (via repeated addition)
@pure fn multiply(a: Int, b: Int): Int {
    result = 0
    for i in 0..b {
        result = result + a
    }
    return result
}

// Complex calculations
ratio = 1/2 + 1/3 + 1/6           // Exact: 1/1
point = (1 + 2i) + (3 + 4i)       // Complex: 4+6i

// Safe user input handling
user_value = parseInput(input)    // Returns Int, not code
total = base + user_value         // Safe addition
```

### What You CANNOT Do

```jtv
// These are GRAMMATICALLY IMPOSSIBLE:
eval(string)           // No eval function
exec(string)           // No exec function
new Function(string)   // No dynamic code creation
system(command)        // No shell access in Data
```

## Summary

The Harvard Architecture in JtV provides:

1. **Security by Design**: Code injection is impossible at the grammar level
2. **Decidability**: Data expressions always terminate
3. **Predictability**: Clear separation of pure and effectful code
4. **Verifiability**: Properties can be formally proven

This isn't security through careful programming - it's security through **architectural impossibility**.
