<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# JtV Purity System

The purity system enforces the Harvard Architecture by tracking function purity at compile time.

## Purity Levels

JtV has three purity levels:

```
┌─────────────────────────────────────────────────────────┐
│                        Impure                           │
│  (Turing-complete, I/O, may not terminate)             │
│  ┌─────────────────────────────────────────────────┐   │
│  │                    Pure                          │   │
│  │  (May loop, no I/O, deterministic)              │   │
│  │  ┌─────────────────────────────────────────┐    │   │
│  │  │               Total                      │    │   │
│  │  │  (No loops, no I/O, guaranteed halt)    │    │   │
│  │  └─────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## @total Functions

**@total** functions are guaranteed to terminate:

```jtv
@total fn increment(x: Int): Int {
    return x + 1
}

@total fn add(a: Int, b: Int): Int {
    return a + b
}
```

### @total Restrictions

| Allowed | Forbidden |
|---------|-----------|
| Assignment | While loops |
| If/else | For loops |
| Function calls (@total only) | Recursion |
| Arithmetic | I/O operations |
| Return | Function calls (@pure or impure) |

### Valid @total Examples

```jtv
@total fn abs(x: Int): Int {
    if x < 0 {
        return x + -x + -x  // Negate
    } else {
        return x
    }
}

@total fn max(a: Int, b: Int): Int {
    if a > b {
        return a
    } else {
        return b
    }
}
```

### Invalid @total Examples

```jtv
// ERROR: Loop in @total function
@total fn bad_multiply(a: Int, b: Int): Int {
    result = 0
    for i in 0..b {  // ERROR: loop forbidden
        result = result + a
    }
    return result
}

// ERROR: I/O in @total function
@total fn bad_greet(name: String): Unit {
    print("Hello, " + name)  // ERROR: I/O forbidden
}
```

## @pure Functions

**@pure** functions have no side effects but may loop:

```jtv
@pure fn multiply(a: Int, b: Int): Int {
    result = 0
    for i in 0..b {
        result = result + a
    }
    return result
}

@pure fn factorial(n: Int): Int {
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

### @pure Restrictions

| Allowed | Forbidden |
|---------|-----------|
| Everything @total allows | I/O operations |
| For loops | Print statements |
| While loops | Input operations |
| Recursion | File/network access |
| Function calls (@total or @pure) | Global state mutation |

### Valid @pure Examples

```jtv
@pure fn gcd(a: Int, b: Int): Int {
    while b != 0 {
        temp = b
        b = a % b  // Modulo via repeated subtraction
        a = temp
    }
    return a
}

@pure fn fibonacci(n: Int): Int {
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

### Invalid @pure Examples

```jtv
// ERROR: I/O in @pure function
@pure fn bad_log(msg: String): Unit {
    print(msg)  // ERROR: I/O forbidden
}

// ERROR: Calling impure function
@pure fn bad_process(): Int {
    input = read()  // ERROR: read() is impure
    return parseInt(input)
}
```

## Impure Functions

Functions without purity annotations are **impure** by default:

```jtv
fn readNumber(): Int {
    print("Enter a number: ")
    input = read()
    return parseInt(input)
}

fn processFile(path: String): String {
    content = readFile(path)
    return transform(content)
}
```

### Impure Capabilities

Impure functions can:
- Perform I/O (print, read, file operations)
- Access network
- Modify global state
- Call any function (total, pure, or impure)

## Data Context Rules

The **Data Language** can only call @total and @pure functions:

```jtv
@total fn square(x: Int): Int {
    return x + x  // Simplified
}

@pure fn cube(x: Int): Int {
    return multiply(x, multiply(x, x))
}

fn impureCalc(): Int {
    print("Calculating...")
    return 42
}

// In assignment (Data context on right side):
result = square(5) + cube(3)  // OK: both callable from Data

// ERROR: Cannot call impure function in Data context
// result = square(5) + impureCalc()  // Compile error!
```

## Purity Inference

The compiler infers purity when not annotated:

```jtv
// Inferred as @total (no loops, no I/O)
fn helper(x: Int): Int {
    return x + 1
}

// Inferred as @pure (has loop, no I/O)
fn repeat(x: Int, n: Int): Int {
    result = 0
    for i in 0..n {
        result = result + x
    }
    return result
}

// Inferred as impure (has I/O)
fn greet(): Unit {
    print("Hello")
}
```

### Annotation Verification

When you annotate a function, the compiler verifies:

```jtv
// Compiler checks that this IS total
@total fn add(a: Int, b: Int): Int {
    return a + b  // OK: no loops, no I/O
}

// ERROR: Annotation doesn't match body
@total fn bad(x: Int): Int {
    for i in 0..x {  // ERROR: loop violates @total
        // ...
    }
}
```

## Purity and the Harvard Architecture

The purity system enforces the Harvard Architecture separation:

```
┌─────────────────────────────────────────────────────────┐
│                    CONTROL LANGUAGE                     │
│                                                         │
│  • Impure functions allowed                            │
│  • Can call any function                               │
│  • Can do I/O                                          │
│  • May not terminate                                   │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │                 DATA LANGUAGE                    │   │
│  │                                                  │   │
│  │  • Only @total and @pure functions             │   │
│  │  • No I/O                                       │   │
│  │  • Deterministic                                │   │
│  │  • (Usually) guaranteed to terminate           │   │
│  │                                                  │   │
│  │  This is where user input is processed safely  │   │
│  └─────────────────────────────────────────────────┘   │
│                         │                               │
│               ┌─────────┴─────────┐                    │
│               │   JOIN POINT      │                    │
│               │   (assignment)    │                    │
│               │   x = data_expr   │                    │
│               └───────────────────┘                    │
└─────────────────────────────────────────────────────────┘
```

## Security Implications

Purity enforcement provides security guarantees:

1. **User input isolation**: User data can only flow through pure computations
2. **No hidden I/O**: Data expressions cannot secretly perform I/O
3. **Predictable execution**: Pure functions always return same output for same input
4. **Termination (with @total)**: No denial-of-service via infinite loops

### Example: Safe Input Processing

```jtv
// User input processing - all pure
@pure fn sanitize(input: String): String {
    // Pure string processing
    return removeInvalidChars(input)
}

@pure fn validate(input: String): Bool {
    // Pure validation
    return isValidFormat(input)
}

@pure fn transform(input: String): Int {
    // Pure transformation
    return parseInt(input)
}

// Main program (impure) coordinates pure components
fn main(): Unit {
    raw = read()  // Impure: I/O

    // All processing is pure - user input cannot become code
    clean = sanitize(raw)
    if validate(clean) {
        result = transform(clean) + 100  // Data context: pure only
        print(result)
    } else {
        print("Invalid input")
    }
}
```

## Purity Checker Implementation

The purity checker performs static analysis:

```rust
pub enum PurityLevel {
    Total,   // No loops, no I/O
    Pure,    // May loop, no I/O
    Impure,  // Everything allowed
}

impl PurityChecker {
    fn analyze_function(&self, func: &Function) -> PurityLevel {
        let body_purity = self.analyze_stmt(&func.body);

        if let Some(annotation) = &func.purity_annotation {
            self.verify_annotation(annotation, body_purity)?;
        }

        body_purity
    }

    fn analyze_stmt(&self, stmt: &Stmt) -> PurityLevel {
        match stmt {
            Stmt::While(_) | Stmt::For(_) => {
                // Loops make it at least non-total
                max_purity(Pure, self.analyze_body(body))
            }
            Stmt::Print(_) | Stmt::Read(_) => Impure,
            Stmt::Call(f, _) => self.get_callee_purity(f),
            // ...
        }
    }
}
```

## See Also

- [Harvard Architecture](../language/Harvard-Architecture.md)
- [Type System](./Type-System.md)
- [Data Language](../language/Data-Language.md)
- [Security Properties](./Security.md)
