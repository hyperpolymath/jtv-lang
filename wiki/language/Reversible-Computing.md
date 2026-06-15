<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# Reversible Computing in JtV

JtV v2 introduces **reversible computing** through `reverse` blocks, enabling quantum algorithm simulation and thermodynamically efficient computation.

## Overview

Reversible computing is computation where every step can be undone. This has profound implications:

1. **Quantum Computing**: Quantum operations are inherently reversible (unitary)
2. **Thermodynamics**: Landauer's principle links irreversible computation to heat dissipation
3. **Debugging**: Run computations backward to find bugs

## The reverse Block

```jtv
reverse {
    x += 5       // Forward: x = x + 5
    y += x       // Forward: y = y + x
    z += y       // Forward: z = z + y
}
// Automatically generates inverse:
// z -= y       // Backward: z = z - y
// y -= x       // Backward: y = y - x
// x -= 5       // Backward: x = x - 5
```

## Reversible Operations

### Addition Assignment (+=)

```jtv
reverse {
    x += expr    // Forward: x = x + expr
}
// Inverse:
// x -= expr    // Backward: x = x - expr
```

**Constraint**: The expression must not contain `x` (the target variable).

```jtv
// Valid
reverse {
    x += y + z   // OK: x not in (y + z)
}

// Invalid
reverse {
    x += x + 1   // ERROR: x appears in expression
}
```

### Subtraction Assignment (-=)

```jtv
reverse {
    x -= expr    // Forward: x = x - expr
}
// Inverse:
// x += expr    // Backward: x = x + expr
```

### Swap

```jtv
reverse {
    swap(x, y)   // Forward: exchange x and y
}
// Inverse:
// swap(x, y)   // Backward: exchange again (self-inverse)
```

## Reversibility Rules

### Rule 1: No Information Loss

Every operation must be invertible:

```jtv
// Reversible (information preserved)
reverse {
    x += 5       // Can undo: x -= 5
}

// NOT reversible (information lost)
reverse {
    x = 5        // ERROR: Original value of x lost
}
```

### Rule 2: Target Not in Expression

```jtv
// Valid
reverse {
    x += y       // x not in y
    y += z       // y not in z
}

// Invalid
reverse {
    x += x       // ERROR: x in x (doubles x, not invertible uniquely)
}
```

### Rule 3: Sequential Dependency

Operations are inverted in reverse order:

```jtv
reverse {
    a += 1       // Step 1
    b += a       // Step 2 (uses modified a)
    c += b       // Step 3 (uses modified b)
}
// Inverse order:
// c -= b       // Undo step 3 first
// b -= a       // Undo step 2
// a -= 1       // Undo step 1 last
```

## Implementation

### RecordedOp

The interpreter records operations for reversal:

```rust
pub enum RecordedOp {
    AddAssign { var: String, value: Value },
    SubAssign { var: String, value: Value },
    Swap { var1: String, var2: String },
}

impl RecordedOp {
    pub fn inverse(&self) -> RecordedOp {
        match self {
            RecordedOp::AddAssign { var, value } =>
                RecordedOp::SubAssign { var: var.clone(), value: value.clone() },
            RecordedOp::SubAssign { var, value } =>
                RecordedOp::AddAssign { var: var.clone(), value: value.clone() },
            RecordedOp::Swap { var1, var2 } =>
                RecordedOp::Swap { var1: var1.clone(), var2: var2.clone() },
        }
    }
}
```

### Execution

```rust
pub struct ReverseTrace {
    operations: Vec<RecordedOp>,
}

impl ReverseTrace {
    pub fn execute_backward(&self, state: &mut State) {
        for op in self.operations.iter().rev() {
            op.inverse().execute(state);
        }
    }
}
```

## Quantum Computing Connection

### Unitary Operations

Quantum gates are unitary matrices—inherently reversible:

```jtv
// Simulating quantum NOT gate (Pauli X)
// |0⟩ → |1⟩, |1⟩ → |0⟩
reverse {
    // Flip qubit state
    qubit_state = 1 + -qubit_state  // 0→1, 1→0
}
```

### Bennett's Trick

Compute, copy result, uncompute to save space:

```jtv
// Compute f(x) reversibly
fn reversible_compute(x: Int): Int {
    result = 0

    // Forward computation
    reverse {
        // Complex computation that modifies ancilla
        temp += x
        temp += temp  // Double
        result += temp
    }

    // Copy result (irreversible, but only 1 bit)
    output = result

    // Uncompute (reverse block runs backward automatically)
    // This restores ancilla to original state

    return output
}
```

### Grover's Algorithm (Conceptual)

```jtv
// Grover iteration (simplified)
fn grover_iteration(amplitudes: [Complex], oracle: (Int) -> Bool): [Complex] {
    n = length(amplitudes)

    // Oracle: flip sign of marked states
    for i in 0..n {
        if oracle(i) {
            reverse {
                amplitudes[i] = amplitudes[i] + -2 * amplitudes[i]
            }
        }
    }

    // Diffusion operator
    mean = sum(amplitudes) / n
    for i in 0..n {
        reverse {
            amplitudes[i] += 2 * (mean + -amplitudes[i])
        }
    }

    return amplitudes
}
```

## Thermodynamic Efficiency

### Landauer's Principle

Erasing one bit of information dissipates at least `kT ln(2)` energy.

**Irreversible computation** erases intermediate values → heat dissipation.
**Reversible computation** preserves information → theoretically zero heat.

```jtv
// Irreversible: erases old value of x
x = y + z  // Old x value lost → heat

// Reversible: preserves information
reverse {
    x += y + z  // Old x recoverable via x -= y + z
}
```

### Practical Implications

1. **Low-power computing**: Reversible circuits for IoT, embedded
2. **Cryogenic computing**: Near-absolute-zero systems benefit most
3. **Theoretical limit**: Approaching Landauer limit requires reversibility

## Examples

### Reversible Counter

```jtv
fn reversible_count(n: Int): Int {
    count = 0
    reverse {
        for i in 0..n {
            count += 1
        }
    }
    return count
}

// Can be run backward to decrement
```

### Reversible Fibonacci

```jtv
fn reversible_fib(n: Int): Int {
    a = 0
    b = 1

    reverse {
        for i in 0..n {
            temp = a + b
            // Reversible update
            a += b + -a  // a becomes old b
            b += temp + -b  // b becomes old a + old b
        }
    }

    return a
}
```

### State Machine with Undo

```jtv
struct StateMachine {
    state: Int,
    history: ReverseTrace,
}

fn transition(sm: StateMachine, input: Int): StateMachine {
    reverse {
        sm.state += input  // Recorded for undo
    }
    return sm
}

fn undo(sm: StateMachine): StateMachine {
    sm.history.execute_backward(&sm)
    return sm
}
```

## Verification

The compiler verifies reversibility constraints:

```jtv
// Compiler checks:
reverse {
    x += y       // ✓ x not in y
    y += x       // ✓ y not in x (x is now x+y, still not containing y)
}

// Compiler rejects:
reverse {
    x = y        // ✗ Non-reversible assignment
    x += x       // ✗ Target in expression
}
```

## Formal Semantics

### Forward Semantics

```
⟨x += e, σ, τ⟩ → ⟨σ[x ↦ σ(x) + ⟦e⟧(σ)], τ ++ [AddAssign(x, ⟦e⟧(σ))]⟩
```

### Backward Semantics

```
⟨backward, σ, [op₁, ..., opₙ]⟩ → ⟨σ', []⟩
where σ' = inverse(opₙ)(inverse(opₙ₋₁)(...inverse(op₁)(σ)...))
```

### Correctness Property

```
∀ σ, S: reversible(S) ⟹
    backward(forward(S, σ)) = σ
```

This is formalized in Lean 4:

```lean
theorem reverse_correct (ops : List RevOp) (σ : State) :
    let σ' := execForward ops σ
    let σ'' := execBackward ops σ'
    σ'' = σ := by
  -- Proof by induction on operations
  sorry  -- Implementation in JtvSecurity.lean
```

## See Also

- [Control Language](./Control-Language.md)
- [Harvard Architecture](./Harvard-Architecture.md)
- [Quantum Vision](./Quantum-Vision.md)
- [Compiler Architecture](../compiler/Architecture.md)
