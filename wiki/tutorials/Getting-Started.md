<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# Getting Started with Julia the Viper

Welcome to Julia the Viper (JtV), a security-focused programming language that makes code injection **grammatically impossible**.

## What is JtV?

JtV is a **Harvard Architecture** programming language that separates:

- **Data Language**: Pure expressions, guaranteed to terminate
- **Control Language**: Full imperative programming

This separation means user input can NEVER become executable code - it's not a runtime check, it's a fundamental design guarantee.

## Installation

### Using Cargo (Rust)

```bash
# Install the JtV CLI
cargo install jtzig

# Verify installation
jtv --version
```

### From Source

```bash
# Clone the repository
git clone https://github.com/hyperpolymath/julia-the-viper.git
cd julia-the-viper

# Build with Cargo
cargo build --release

# Run
./target/release/jtv --help
```

### Using the REPL

```bash
# Start the interactive REPL
jtv repl
```

## Your First Program

### Hello, World!

Create a file `hello.jtv`:

```jtv
// hello.jtv - Your first JtV program
message = "Hello, World!"
print(message)
```

Run it:

```bash
jtv run hello.jtv
```

Output:
```
Hello, World!
```

### Simple Arithmetic

```jtv
// arithmetic.jtv - Basic calculations
x = 10
y = 5

// Addition is the primary operation
sum = x + y
print(sum)  // 15

// Subtraction via additive inverse
diff = x + -y
print(diff)  // 5

// Working with different number types
ratio = 1/2 + 1/3
print(ratio)  // 5/6 (exact rational!)

complex_num = (3 + 4i) + (1 + 2i)
print(complex_num)  // 4+6i
```

### Control Flow

```jtv
// control.jtv - Loops and conditionals

// For loop (bounded, safe)
sum = 0
for i in 1..11 {
    sum = sum + i
}
print(sum)  // 55 (sum of 1 to 10)

// Conditional
x = 42
if x > 0 {
    print("Positive")
} else {
    print("Non-positive")
}
```

## Key Concepts

### 1. Addition-Only Arithmetic

JtV uses addition as the sole arithmetic operator in the Data Language:

```jtv
// This is valid
result = a + b + c

// Subtraction uses additive inverse
result = a + -b  // Same as a - b

// Multiplication uses repeated addition (in Control)
@pure fn multiply(a: Int, b: Int): Int {
    result = 0
    for i in 0..b {
        result = result + a
    }
    return result
}
```

### 2. Harvard Architecture

Data and Control are separate:

```jtv
// DATA LANGUAGE (expressions) - always terminates
data_expr = 5 + 3 + x

// CONTROL LANGUAGE (statements) - may not terminate
while condition {
    // Control flow
}

// JOIN POINT (assignment) - where Data flows into Control
x = data_expr  // Data value assigned to Control variable
```

### 3. Security by Design

```jtv
// User input is ALWAYS data, NEVER code
user_input = getUserInput()  // Returns a value

// This is SAFE - user_input cannot become executable code
result = base + user_input

// There is NO eval(), NO exec(), NO way to execute strings as code
// This is NOT a best practice - it's GRAMMATICALLY IMPOSSIBLE
```

### 4. Number Systems

JtV supports 7 number types:

```jtv
integer = 42
floating = 3.14
rational = 22/7        // Exact fraction
complex_num = 3 + 4i
hex = 0xFF
binary = 0b1010
symbolic = pi          // Mathematical constant
```

## Project Structure

A typical JtV project:

```
my-project/
├── src/
│   ├── main.jtv       # Entry point
│   ├── math.jtv       # Math utilities
│   └── utils.jtv      # Helper functions
├── tests/
│   └── test_math.jtv  # Tests
└── jtv.toml           # Project configuration
```

## Running Programs

### Execute a file

```bash
jtv run src/main.jtv
```

### Interactive REPL

```bash
jtv repl
```

```
JtV REPL v1.0.0
Type :help for commands, :quit to exit

>>> x = 5
5
>>> y = 3
3
>>> x + y
8
>>> 1/2 + 1/3
5/6
>>> :quit
```

### Check syntax

```bash
jtv check src/main.jtv
```

### Format code

```bash
jtv fmt src/
```

## Common Patterns

### Safe User Input

```jtv
// Always safe - input is data, not code
fn processForm(input: String): Int {
    value = parseInt(input)
    // Even if input is "'; DROP TABLE users; --"
    // it's just a string that parseInt rejects
    return value + 100
}
```

### Pure Functions

```jtv
// Mark functions as @pure to use in Data expressions
@pure fn square(x: Int): Int {
    // No loops, no I/O - guaranteed to halt
    return x + x  // Simplified: actual squaring uses multiply
}

// Can use pure functions in Data context
result = square(5) + square(3)
```

### Working with Rationals

```jtv
// Exact arithmetic - no floating point errors
price1 = 19/100  // $0.19
price2 = 99/100  // $0.99
total = price1 + price2  // 118/100 = $1.18 exactly
```

## Next Steps

1. **[Language Reference](../reference/Grammar.md)** - Complete syntax reference
2. **[Number Systems](../language/Number-Systems.md)** - Deep dive into types
3. **[Harvard Architecture](../language/Harvard-Architecture.md)** - Understanding the security model
4. **[Standard Library](../stdlib/Overview.md)** - Built-in functions and modules
5. **[Tooling](../tooling/CLI.md)** - CLI, LSP, and development tools

## Getting Help

- **Documentation**: This wiki
- **Issues**: [GitHub Issues](https://github.com/hyperpolymath/julia-the-viper/issues)
- **Discussions**: [GitHub Discussions](https://github.com/hyperpolymath/julia-the-viper/discussions)

Welcome to secure programming with JtV!
