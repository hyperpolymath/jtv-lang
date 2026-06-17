<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# Property-Based Testing in JtV

JtV supports property-based testing inspired by tools like QuickCheck, Hypothesis, and **Echidna** (smart contract fuzzer). Instead of writing individual test cases, you specify properties that should always hold.

## Overview

Traditional testing:
```jtv
test "addition is correct" {
    assertEqual(add(2, 3), 5)
    assertEqual(add(0, 0), 0)
    assertEqual(add(-1, 1), 0)
}
```

Property-based testing:
```jtv
property "addition is commutative" {
    forAll (a: Int, b: Int) {
        assert(a + b == b + a)
    }
}
```

## Property Syntax

### Basic Properties

```jtv
import std/testing

property "name" {
    forAll (params...) {
        // assertions
    }
}
```

### Generators

```jtv
// Built-in generators
forAll (n: Int) { ... }           // Any integer
forAll (f: Float) { ... }         // Any float
forAll (r: Rational) { ... }      // Any rational
forAll (s: String) { ... }        // Any string
forAll (b: Bool) { ... }          // true or false

// Constrained generators
forAll (n: Int where n > 0) { ... }           // Positive integers
forAll (n: Int where n >= 0 && n < 100) { ... }  // Range
forAll (s: String where length(s) < 10) { ... }  // Short strings

// Custom generators
generator SmallInt = Int where -100 <= self && self <= 100
forAll (n: SmallInt) { ... }
```

### Multiple Parameters

```jtv
property "addition associativity" {
    forAll (a: Int, b: Int, c: Int) {
        assert((a + b) + c == a + (b + c))
    }
}
```

## JtV-Specific Properties

### Harvard Architecture Properties

```jtv
// Verify Data expressions are pure
property "data expressions have no side effects" {
    forAll (e: DataExpr, state: State) {
        let result = eval(e, state)
        let result2 = eval(e, state)
        assert(result == result2)  // Same result = no side effects
    }
}

// Verify Data expressions terminate
property "data expressions terminate" {
    forAll (e: DataExpr, state: State) {
        // This property is verified by running - if it hangs, test fails
        let _ = eval(e, state)
        assert(true)
    }
}
```

### Purity Properties

```jtv
// @total functions always terminate
property "@total functions halt" {
    forAll (f: TotalFunction, args: Args) {
        timeout(1000) {  // 1 second timeout
            let _ = f(args)
            assert(true)
        }
    }
}

// @pure functions are deterministic
property "@pure functions are deterministic" {
    forAll (f: PureFunction, args: Args) {
        let r1 = f(args)
        let r2 = f(args)
        assert(r1 == r2)
    }
}
```

### Type System Properties

```jtv
// Type coercion is consistent
property "coercion is transitive" {
    forAll (a: Int, b: Float, c: Complex) {
        let r1 = (a + b) + c
        let r2 = a + (b + c)
        assertApprox(r1, r2, 1e-10)
    }
}

// Rationals are exact
property "rational arithmetic is exact" {
    forAll (a: Rational, b: Rational) {
        let sum = a + b
        let diff = sum + -b
        assert(diff == a)  // No precision loss
    }
}
```

### Reversibility Properties

```jtv
// Reverse blocks are invertible
property "reverse blocks restore state" {
    forAll (initialState: State, ops: [RevOp]) {
        let state1 = clone(initialState)

        reverse {
            for op in ops {
                execute(op, state1)
            }
        }
        // Automatic reversal happens here

        assert(state1 == initialState)
    }
}

// Specific operation reversibility
property "addition assignment is reversible" {
    forAll (x: Int, y: Int where x != y) {
        let original = x
        reverse {
            x += y
        }
        assert(x == original)
    }
}
```

## Invariant Testing (Echidna-style)

Inspired by Echidna, JtV supports **invariant testing** where you define properties that must hold across all possible sequences of operations.

### Invariant Syntax

```jtv
invariant "balance is non-negative" for Account {
    assert(self.balance >= 0)
}

invariant "total supply is constant" for TokenSystem {
    let total = sum(account.balance for account in self.accounts)
    assert(total == self.initialSupply)
}
```

### State Machine Testing

```jtv
stateMachine Counter {
    state: Int = 0

    action increment() {
        state += 1
    }

    action decrement() requires state > 0 {
        state += -1
    }

    invariant "non-negative" {
        assert(state >= 0)
    }
}

test "counter invariant holds" {
    fuzz(Counter, iterations: 10000)
}
```

## Shrinking

When a property fails, JtV automatically shrinks the failing input to find a minimal counterexample.

```jtv
// If this fails with (a=1000, b=-500)
property "always positive" {
    forAll (a: Int, b: Int) {
        assert(a + b > 0)  // Fails!
    }
}
// Shrunk counterexample: (a=0, b=0) or (a=1, b=-1)
```

### Custom Shrinking

```jtv
generator TreeNode shrink {
    // Define how to shrink TreeNode to smaller versions
    match self {
        Leaf(v) => [Leaf(shrink(v))]
        Branch(l, r) => [l, r, Branch(shrink(l), r), Branch(l, shrink(r))]
    }
}
```

## Coverage-Guided Fuzzing

JtV's fuzzer tracks code coverage to guide test generation:

```jtv
fuzz "parser coverage" {
    target: parse
    corpus: ["1 + 2", "x = 5", "if true { }"]
    iterations: 100000
    coverage: branch  // Track branch coverage
}
```

### Coverage Modes

- `line`: Line coverage
- `branch`: Branch coverage
- `path`: Path coverage (expensive)
- `mutation`: Mutation coverage

## Integration with Formal Proofs

Properties can be linked to Lean 4 proofs:

```jtv
// This property is proven in Lean
@proven("dataExpr_totality")
property "data expressions terminate" {
    forAll (e: DataExpr, state: State) {
        let _ = eval(e, state)
        assert(true)
    }
}
```

The `@proven` annotation indicates the property has a formal proof, but the test still runs to verify the implementation matches the specification.

## Configuration

### Test Configuration

```toml
# jtv.toml
[test.property]
iterations = 1000        # Tests per property
timeout = 5000          # ms per test
seed = 42               # Reproducible randomness
shrink_iterations = 100 # Shrinking attempts
coverage = "branch"     # Coverage mode

[test.fuzz]
iterations = 100000
corpus_dir = "fuzz/corpus"
crashes_dir = "fuzz/crashes"
```

### CLI Options

```bash
# Run property tests
jtv test --property

# Run with specific seed
jtv test --property --seed 12345

# Run more iterations
jtv test --property --iterations 10000

# Enable fuzzing
jtv test --fuzz --iterations 1000000
```

## Example Test Suite

```jtv
// tests/properties.jtv
import std/testing

// Arithmetic properties
property "addition commutative" {
    forAll (a: Int, b: Int) {
        assert(a + b == b + a)
    }
}

property "addition associative" {
    forAll (a: Int, b: Int, c: Int) {
        assert((a + b) + c == a + (b + c))
    }
}

property "additive identity" {
    forAll (a: Int) {
        assert(a + 0 == a)
    }
}

property "additive inverse" {
    forAll (a: Int) {
        assert(a + -a == 0)
    }
}

// Rational properties
property "rational addition exact" {
    forAll (a: Rational, b: Rational) {
        let c = a + b
        let d = c + -b
        assert(d == a)  // No precision loss
    }
}

// Pure function properties
@pure fn double(x: Int): Int {
    return x + x
}

property "double is 2x" {
    forAll (x: Int) {
        assert(double(x) == x + x)
    }
}

// Reversibility
property "reverse block identity" {
    forAll (x: Int, delta: Int) {
        let original = x
        reverse {
            x += delta
        }
        assert(x == original)
    }
}
```

## See Also

- [Testing Framework](./Testing-Framework.md)
- [CLI](./CLI.md)
- [Formal Proofs](../reference/Formal-Proofs.md)
- [Security](../internals/Security.md)
