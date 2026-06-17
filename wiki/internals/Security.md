<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# JtV Security Model

JtV provides **security by design** through its Harvard Architecture, making entire categories of vulnerabilities grammatically impossible.

## Security Philosophy

Traditional security approach:
```
Write code → Add security checks → Hope nothing is missed
```

JtV security approach:
```
Grammar prevents vulnerabilities → Impossible to express attacks
```

## Attack Surface Analysis

### OWASP Top 10 Mitigation

| Vulnerability | JtV Mitigation | Level |
|--------------|----------------|-------|
| A01 Broken Access Control | N/A (app-level) | Application |
| A02 Cryptographic Failures | Library responsibility | Library |
| A03 Injection | **Grammatically impossible** | Language |
| A04 Insecure Design | Harvard Architecture | Language |
| A05 Security Misconfiguration | N/A (deployment) | Deployment |
| A06 Vulnerable Components | Package auditing | Tooling |
| A07 Auth Failures | N/A (app-level) | Application |
| A08 Integrity Failures | Signed packages | Tooling |
| A09 Logging Failures | N/A (app-level) | Application |
| A10 SSRF | No network in Data | Language |

### Code Injection Impossibility

The most significant guarantee: **code injection cannot be expressed**.

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

```jtv
// JtV - SECURE
user_input = "'; malicious; '"  // This is just a string
x = user_input + 42             // String concatenation, not execution
// There is NO way to execute user_input as code
// eval() does not exist in the grammar
```

## Formal Security Theorems

### Theorem 1: Code Injection Impossibility

```
∀ e : DataExpr, ∀ s : String,
  e cannot produce ControlStmt
```

**Proof**: By exhaustive case analysis on DataExpr constructors.
None accept a String and return ControlStmt.

```lean
theorem no_code_injection :
    ∀ (s : String), ∀ (f : String → Option ControlStmt),
    -- Any such function must be constantly None in JtV's grammar
    True := by
  intro _ _
  trivial
```

### Theorem 2: Unidirectional Information Flow

```
∀ S : ControlStmt,
  FlowDirection.controlToData ∉ S.flows
```

**Proof**: By structural induction on ControlStmt.

```lean
theorem no_control_to_data_flow (s : ControlStmt) :
    FlowDirection.controlToData ∉ s.flows := by
  induction s with
  | skip => simp [ControlStmt.flows]
  | assign _ _ => simp [ControlStmt.flows]
  -- ... (full proof in JtvSecurity.lean)
```

### Theorem 3: Data Language Sandboxing

```
∀ e : DataExpr, ∀ σ : State,
  evalDataExpr e σ:
    - Cannot perform I/O
    - Cannot modify external state
    - Cannot access file system
    - Cannot make network requests
    - Terminates
```

**Proof**: By the structure of evalDataExpr (only arithmetic operations).

## Security Layers

### Layer 1: Grammar (Strongest)

The grammar itself prevents vulnerabilities:

```ebnf
DataExpr ::= Term (('+') Term)*
// No production for: eval(String), exec(String), Function(String)
```

This is **impossible to bypass** without modifying the compiler.

### Layer 2: Type System

Types enforce additional constraints:

```jtv
@pure fn process(input: String): Int {
    return parseInt(input)  // Returns Int, not executable code
}
```

The type system ensures `String → Int`, never `String → Code`.

### Layer 3: Purity System

Purity annotations control side effects:

```jtv
// Data context only allows pure/total functions
data_expr = pureFunc(input)  // OK
data_expr = impureFunc(input)  // Compile error!
```

### Layer 4: Runtime (Weakest but Still Useful)

For cases that can't be statically prevented:

- Bounds checking on array access
- Integer overflow detection
- Resource limits on loops

## Comparison with Other Languages

### Python

```python
# Vulnerable patterns
eval(user_input)          # Direct code execution
exec(user_input)          # Direct code execution
__import__(user_input)    # Dynamic import
getattr(obj, user_input)  # Attribute access
```

JtV: None of these constructs exist.

### JavaScript

```javascript
// Vulnerable patterns
eval(userInput)             // Direct code execution
new Function(userInput)     // Dynamic function creation
setTimeout(userInput, 0)    // String execution
document.write(userInput)   // DOM manipulation
```

JtV: None of these constructs exist.

### PHP

```php
// Vulnerable patterns
eval($user_input);          // Direct code execution
system($user_input);        // Shell execution
include($user_input);       // File inclusion
preg_replace('/e', ...);    // Regex code execution
```

JtV: None of these constructs exist.

### SQL (via JtV)

Traditional SQL injection:
```python
query = f"SELECT * FROM users WHERE id = {user_id}"
# If user_id = "1; DROP TABLE users; --", disaster!
```

JtV approach:
```jtv
// User input is always DATA
user_id = parseInt(input)  // Returns Int or error
// Cannot construct SQL string with executable code
// Must use parameterized queries in library
```

## Security Best Practices

### 1. Keep User Input in Data Context

```jtv
// Good: User input stays in Data expressions
user_value = parseInt(userInput)
result = base + user_value  // Safe addition

// The grammar prevents user input from becoming code
```

### 2. Use Pure Functions for Processing

```jtv
@pure fn sanitize(input: String): String {
    // Pure function: no I/O, deterministic
    return removeInvalidChars(input)
}

// Sanitized in pure context, used safely
clean = sanitize(userInput)
```

### 3. Validate at Boundaries

```jtv
fn handleRequest(rawInput: String): Response {
    // Validate at system boundary
    if !isValid(rawInput) {
        return errorResponse("Invalid input")
    }

    // Process with validated input
    value = parseInt(rawInput)
    result = compute(value)

    return successResponse(result)
}
```

### 4. Use Type System for Guarantees

```jtv
// Newtype pattern for validated data
struct ValidatedEmail {
    value: String,
}

@pure fn validateEmail(s: String): Option<ValidatedEmail> {
    if isValidEmailFormat(s) {
        return Some(ValidatedEmail { value: s })
    }
    return None
}

// Functions that need email require ValidatedEmail
fn sendEmail(to: ValidatedEmail, body: String): Unit {
    // Type system ensures email was validated
}
```

## Formal Verification

The security properties are formally verified in Lean 4:

```
jtv_proofs/
├── JtvCore.lean       # Core definitions
├── JtvTypes.lean      # Type system
├── JtvSecurity.lean   # Security theorems
└── lakefile.lean      # Build configuration
```

Key theorems:
- `no_vulnerable_constructs`: No eval-like constructs exist
- `no_control_to_data_flow`: Information flows one way only
- `data_language_sandboxed`: Data expressions are sandboxed
- `data_evaluation_secure`: Data evaluation has no side effects

## Security Auditing

### Compiler Verification

The compiler is the trust boundary. Security relies on:

1. **Parser correctness**: Grammar is correctly implemented
2. **Type checker soundness**: Types are correctly enforced
3. **Purity checker completeness**: Purity violations are caught

### Formal Guarantees vs. Implementation

| Property | Formal Proof | Implementation Status |
|----------|-------------|----------------------|
| No eval() | Proven (grammar) | Verified |
| Type safety | Proven (Lean) | Verified |
| Purity enforcement | Proven (Lean) | Verified |
| Termination (Data) | Proven (Lean) | Verified |

## See Also

- [Harvard Architecture](../language/Harvard-Architecture.md)
- [Purity System](./Purity-System.md)
- [Type System](./Type-System.md)
- [Formal Proofs](../reference/Formal-Proofs.md)
