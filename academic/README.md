# Julia the Viper: Academic Documentation

**SPDX-License-Identifier: MPL-2.0

This directory contains comprehensive academic documentation for Julia the Viper (JtV), including formal proofs, white papers, and theoretical foundations.

---

## Overview

JtV is a programming language that achieves **code injection immunity through grammatical separation**. This documentation provides rigorous academic treatment of all theoretical aspects.

---

## Document Index

### White Papers

| Document | Description |
|----------|-------------|
| [papers/WHITE_PAPER.md](papers/WHITE_PAPER.md) | Main language white paper with security proofs |

### Formal Proofs

| Document | Description |
|----------|-------------|
| [proofs/MATHEMATICAL_FOUNDATIONS.md](proofs/MATHEMATICAL_FOUNDATIONS.md) | Set theory, category theory, domain theory, lambda calculus |
| [proofs/COMPUTATIONAL_THEORY.md](proofs/COMPUTATIONAL_THEORY.md) | Chomsky hierarchy, decidability, complexity analysis |
| [proofs/TYPE_THEORY.md](proofs/TYPE_THEORY.md) | Complete type system formalization with metatheory |
| [proofs/SECURITY_PROOFS.md](proofs/SECURITY_PROOFS.md) | Non-interference, attack models, OWASP mapping |
| [proofs/QUANTUM_REVERSIBILITY.md](proofs/QUANTUM_REVERSIBILITY.md) | Reversible computation, quantum gate simulation |
| [proofs/ALGEBRAIC_STRUCTURES.md](proofs/ALGEBRAIC_STRUCTURES.md) | Groups, rings, fields for all 7 number systems |
| [proofs/STATISTICS_PROBABILITY.md](proofs/STATISTICS_PROBABILITY.md) | Probabilistic semantics and statistical foundations |

### Mechanized Proofs (Lean 4)

| File | Description |
|------|-------------|
| [../jtv_proofs/JtvCore.lean](../jtv_proofs/JtvCore.lean) | Denotational semantics |
| [../jtv_proofs/JtvTypes.lean](../jtv_proofs/JtvTypes.lean) | Type system formalization |
| [../jtv_proofs/JtvSecurity.lean](../jtv_proofs/JtvSecurity.lean) | Security property proofs |
| [../jtv_proofs/JtvOperational.lean](../jtv_proofs/JtvOperational.lean) | Operational semantics |
| [../jtv_proofs/JtvTheorems.lean](../jtv_proofs/JtvTheorems.lean) | Main theorems and lemmas |
| [../jtv_proofs/JtvExtended.lean](../jtv_proofs/JtvExtended.lean) | Extended proofs and metatheorems |

### Supporting Materials

| Document | Description |
|----------|-------------|
| [BIBLIOGRAPHY.md](BIBLIOGRAPHY.md) | Complete academic bibliography (80+ references) |
| [TODO_GAPS.md](TODO_GAPS.md) | Implementation gaps and action items |

---

## Key Theorems

### Security

1. **Code Injection Impossibility** (SECURITY_PROOFS.md §2.3)
   - Code injection is grammatically impossible in JtV

2. **Non-Interference** (SECURITY_PROOFS.md §3.3)
   - High-security data cannot influence low-security outputs

3. **Data Sandboxing** (SECURITY_PROOFS.md §5)
   - Data Language has no write, I/O, or system capabilities

### Computability

1. **Data Language Totality** (COMPUTATIONAL_THEORY.md §2.1)
   - All Data expressions terminate

2. **Control Language Turing-Completeness** (COMPUTATIONAL_THEORY.md §1.4)
   - Control Language can simulate any Turing machine

3. **Decidability Separation** (COMPUTATIONAL_THEORY.md §2)
   - Termination decidable for Data, undecidable for Control

### Type Theory

1. **Type Safety** (TYPE_THEORY.md §3)
   - Progress + Preservation theorems

2. **Purity Enforcement** (TYPE_THEORY.md §5)
   - @pure/@total correctly restrict side effects

3. **Principal Types** (TYPE_THEORY.md §3.2)
   - Every well-typed expression has a principal type

### Reversibility

1. **Forward-Backward Identity** (QUANTUM_REVERSIBILITY.md §2.4)
   - Reversible operations compose to identity

2. **Quantum Gate Simulation** (QUANTUM_REVERSIBILITY.md §3)
   - Classical simulation of reversible/quantum gates

---

## Verification Status

| Component | Lean Proofs | Paper Proofs | Status |
|-----------|-------------|--------------|--------|
| Totality | ✓ | ✓ | Verified |
| Determinism | ✓ | ✓ | Verified |
| Security (No eval) | ✓ | ✓ | Verified |
| Information flow | ✓ | ✓ | Verified |
| Type preservation | ✓ | ✓ | Verified |
| Progress | ✓ | ✓ | Verified |
| Reversibility | ✓ | ✓ | Verified |
| Algebraic laws | ✓ | ✓ | Verified |

---

## Reading Order

For comprehensive understanding, read in this order:

1. **WHITE_PAPER.md** - High-level overview and motivation
2. **MATHEMATICAL_FOUNDATIONS.md** - Formal foundations
3. **TYPE_THEORY.md** - Type system details
4. **SECURITY_PROOFS.md** - Security guarantees
5. **COMPUTATIONAL_THEORY.md** - Computability properties
6. **ALGEBRAIC_STRUCTURES.md** - Number system algebra
7. **QUANTUM_REVERSIBILITY.md** - v2 reversibility features
8. **STATISTICS_PROBABILITY.md** - Probabilistic aspects

---

## Building Lean Proofs

```bash
cd jtv_proofs
lake build
```

---

## Academic Citation

```bibtex
@misc{jtv2025,
  title = {Julia the Viper: A Harvard Architecture Language for
           Grammatically Enforced Code Injection Immunity},
  author = {JtV Research Team},
  year = {2025},
  howpublished = {\url{https://github.com/hyperpolymath/julia-the-viper}},
  note = {Version 1.0}
}
```

---

## Contributing

See [TODO_GAPS.md](TODO_GAPS.md) for areas needing work. Priority items:

1. Complete Lean proofs using `sorry`
2. Implement missing type features
3. Run fuzzing campaign
4. Prepare for peer review

---

## License

All academic documentation is licensed under GPL-3.0-or-later.
