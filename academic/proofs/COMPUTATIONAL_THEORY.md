# Computational Theory of Julia the Viper

**SPDX-License-Identifier: CC-BY-SA-4.0

This document establishes the computational-theoretic properties of JtV, including Chomsky hierarchy classification, decidability results, complexity analysis, and connections to automata theory.

---

## 1. Chomsky Hierarchy Classification

### 1.1 The Four Levels

The Chomsky hierarchy classifies formal languages by generative power:

| Type | Name | Automaton | Restrictions |
|------|------|-----------|--------------|
| 0 | Recursively Enumerable | Turing Machine | None |
| 1 | Context-Sensitive | LBA | |α| ≤ |β| |
| 2 | Context-Free | PDA | A → γ |
| 3 | Regular | DFA/NFA | A → aB or A → a |

### 1.2 JtV Grammar Classification

**Theorem 1.1 (JtV Syntax is Context-Free):**
The JtV grammar is context-free (Type 2).

*Proof:* The EBNF grammar in `jtv.ebnf` consists exclusively of production rules of the form:
```
A ::= α₁ | α₂ | ... | αₙ
```
where A is a non-terminal and αᵢ are strings of terminals and non-terminals.

This is the canonical form for context-free grammars. No production requires context (e.g., no rules like αAβ → αγβ where context α,β matters). ∎

**Corollary 1.2 (Parseable by PDA):**
JtV programs can be recognized by a pushdown automaton.

**Theorem 1.3 (JtV is Not Regular):**
The JtV language is not regular (not Type 3).

*Proof:* Consider matched parentheses in expressions: `(((...)))`.
By the pumping lemma for regular languages, if L is regular, then for sufficiently long strings w ∈ L, there exist x,y,z with w = xyz where:
1. |xy| ≤ p
2. |y| ≥ 1
3. xyⁿz ∈ L for all n ≥ 0

Take w = `(ⁿ1)ⁿ` (n left parens, `1`, n right parens).
Any pumping decomposition within the first p characters pumps only left parens, breaking the balance. ∎

### 1.3 Data Language Sublanguage

**Theorem 1.4 (Data Language is Context-Free):**
The Data Language grammar is a proper subset of context-free grammars.

**Theorem 1.5 (Data Semantics is Primitive Recursive):**
While the syntax is CF, the *semantics* of DataExpr are weaker:

The evaluation function `eval : DataExpr × State → ℤ` is primitive recursive.

*Proof:* Define eval by primitive recursion on expression structure:
```
eval(n, σ) = n
eval(x, σ) = lookup(x, σ)
eval(e₁ + e₂, σ) = eval(e₁, σ) + eval(e₂, σ)
eval(-e, σ) = negate(eval(e, σ))
```
Each case uses only:
- Constant functions
- Successor (for literals)
- Projection (for variables)
- Composition
- Primitive recursion (structural)

No unbounded search (μ-recursion) is needed. ∎

### 1.4 Control Language Classification

**Theorem 1.6 (Control Language is Turing-Complete):**
The Control Language can simulate any Turing machine.

*Proof:* We show that Control Language can compute any μ-recursive function.

**Step 1:** Primitive recursive functions are computable:
- Zero: `x = 0`
- Successor: `x = x + 1`
- Projection: `result = xₖ`
- Composition: `temp = g(...); result = f(temp)`
- Primitive recursion: `for i in 0..n { ... }`

**Step 2:** Unbounded search (μ-operator) is computable:
```jtv
fn mu_search(): Int {
    i = 0
    while predicate(i) != 0 {
        i = i + 1
    }
    return i
}
```

By Kleene's normal form theorem, any computable function can be expressed using composition and μ-search on primitive recursive predicates. ∎

---

## 2. Decidability Results

### 2.1 Decision Problems for Data Language

**Theorem 2.1 (Data Termination is Decidable):**
Given DataExpr e, the question "Does eval(e, σ) terminate?" is decidable.

*Proof:* The answer is always YES. By Theorem 5.1 of the White Paper (Totality), all Data expressions terminate. Thus the decision procedure is the trivial algorithm that always returns TRUE. ∎

**Theorem 2.2 (Data Equivalence is Decidable):**
Given DataExpr e₁, e₂, the question "∀σ. eval(e₁, σ) = eval(e₂, σ)?" is decidable.

*Proof:*
1. Extract free variables V = freeVars(e₁) ∪ freeVars(e₂)
2. Symbolically evaluate e₁ and e₂
3. Apply algebraic simplification using the equational theory
4. Check syntactic equality of normal forms

The equational theory of integer addition is decidable (Presburger arithmetic). ∎

**Theorem 2.3 (Data Type Checking is Decidable):**
Given DataExpr e and Type τ, "Γ ⊢ e : τ?" is decidable.

*Proof:* The typing rules are syntax-directed. Algorithm:
```
typecheck(n) = Int
typecheck(x) = Γ(x)
typecheck(e₁ + e₂) = if typecheck(e₁) = typecheck(e₂) = Int then Int else ERROR
typecheck(-e) = if typecheck(e) = Int then Int else ERROR
```
This terminates in O(size(e)) time. ∎

### 2.2 Decision Problems for Control Language

**Theorem 2.4 (Control Termination is Undecidable):**
Given ControlStmt s and State σ, "Does exec(s, σ) terminate?" is undecidable.

*Proof:* By reduction from the Halting Problem.

Given Turing machine M and input w, construct ControlStmt s_M that simulates M on w:
- Variables encode tape contents
- While loops simulate state transitions
- Halting states become `return`

s_M terminates ⟺ M halts on w.

If Control termination were decidable, we could decide the Halting Problem. Contradiction. ∎

**Theorem 2.5 (Control Equivalence is Undecidable):**
Given ControlStmt s₁, s₂, "∀σ. exec(s₁, σ) = exec(s₂, σ)?" is undecidable.

*Proof:* Reduce termination to equivalence:
- s₁ = the given program
- s₂ = infinite loop

s₁ ≢ s₂ ⟺ s₁ terminates on some input.

By Rice's theorem, non-trivial semantic properties of Turing-complete languages are undecidable. ∎

### 2.3 Decidability Summary

| Problem | Data Language | Control Language |
|---------|---------------|------------------|
| Termination | ✓ Decidable (always YES) | ✗ Undecidable |
| Equivalence | ✓ Decidable | ✗ Undecidable |
| Type Checking | ✓ Decidable | ✓ Decidable |
| Type Inference | ✓ Decidable | ✓ Decidable |
| Purity Analysis | ✓ Decidable | ✓ Decidable |

### 2.4 The Halting Boundary

**Definition 2.1 (Halting Boundary):**
The *halting boundary* is the syntactic division between constructs that always terminate and constructs that may diverge.

In JtV:
- **Safe side:** DataExpr (always terminates)
- **Unsafe side:** while loops in ControlStmt (may diverge)

**Theorem 2.6 (Halting Boundary Coincides with Harvard Separation):**
```
Terminates(e) ⟺ e ∈ DataExpr
MayDiverge(s) ⟺ s ∈ ControlStmt with loops
```

---

## 3. Complexity Analysis

### 3.1 Time Complexity

**Definition 3.1 (Data Evaluation Complexity):**
```
T_data(e) = number of reduction steps to evaluate e
```

**Theorem 3.1 (Linear Data Complexity):**
T_data(e) = O(size(e))

*Proof:* Each node in the expression tree is visited exactly once. Each visit performs O(1) work (integer addition or lookup). ∎

**Theorem 3.2 (Parsing Complexity):**
JtV parsing is O(n) where n is input length.

*Proof:* Pest generates an LL(k) parser. With memoization, parsing is linear. ∎

**Theorem 3.3 (Type Checking Complexity):**
Type checking is O(n) for Data Language, O(n) for well-structured Control Language.

### 3.2 Space Complexity

**Theorem 3.4 (Data Space Complexity):**
Space for evaluating DataExpr e is O(depth(e)).

*Proof:* Evaluation uses a stack proportional to expression depth. Maximum stack depth equals maximum nesting. ∎

**Theorem 3.5 (Control Space Complexity):**
Space for executing ControlStmt may be unbounded.

*Proof:* Consider:
```jtv
while 1 {
    push_to_list(x)  // Unbounded list growth
}
```

### 3.3 Complexity Classes

**Definition 3.2 (JtV Complexity Classes):**

For Data Language:
```
DATA-TIME = DTIME(n) ⊆ P
DATA-SPACE = DSPACE(log n) ⊆ L
```

For Control Language:
```
CTRL-TIME = RE (recursively enumerable)
CTRL-SPACE = unbounded
```

**Theorem 3.6 (Data is in NC):**
DataExpr evaluation is in NC (efficiently parallelizable).

*Proof:* Addition is associative. Expression tree can be evaluated bottom-up with O(log n) parallel steps using n processors. ∎

### 3.4 Complexity of Security Checking

**Theorem 3.7 (Injection Check Complexity):**
Verifying absence of injection vulnerabilities is O(1).

*Proof:* The grammar structurally prohibits injection. No runtime check needed. The "algorithm" is the parser itself, which accepts only safe programs. ∎

---

## 4. Automata Theory

### 4.1 Finite Automata for Lexing

**Definition 4.1 (JtV Lexer DFA):**
The lexer is a DFA recognizing token types:
- Identifiers: `[a-zA-Z][a-zA-Z0-9_]*`
- Integers: `-?[0-9]+`
- Operators: `+`, `-`, `=`, etc.
- Keywords: `if`, `while`, `fn`, etc.

**Theorem 4.1 (Lexing is Regular):**
JtV lexical analysis is regular (Type 3).

### 4.2 Pushdown Automata for Parsing

**Definition 4.2 (JtV Parser PDA):**
The parser is a deterministic PDA:
- States: parser states from grammar
- Stack: tracks nested structures
- Transitions: shift/reduce operations

**Theorem 4.2 (JtV is LR(1)):**
The JtV grammar is LR(1)-parseable.

*Proof:* The Pest parser generator produces a PEG parser, which is strictly more powerful than LR(1). The grammar has no LR conflicts. ∎

### 4.3 Tree Automata for AST

**Definition 4.3 (DataExpr Tree Automaton):**
A bottom-up tree automaton A = (Q, Σ, δ, F) for DataExpr:
- Q = {q_int, q_var, q_expr}
- Σ = {Lit, Var, Add, Neg}
- δ: transitions
- F = {q_expr}

```
δ(Lit(n), ε) = q_int
δ(Var(x), ε) = q_var
δ(Add, q_expr, q_expr) = q_expr
δ(Neg, q_expr) = q_expr
```

**Theorem 4.3 (Well-Formed AST Recognition):**
The set of valid DataExpr ASTs is recognizable by a tree automaton.

### 4.4 Linear Bounded Automata

**Theorem 4.4 (Data Evaluation by LBA):**
DataExpr evaluation can be performed by a linear bounded automaton.

*Proof:* The evaluation of a DataExpr of size n requires only O(n) space to store intermediate results. An LBA can simulate this evaluation. ∎

**Theorem 4.5 (Control Requires Full TM):**
Control Language execution may require unbounded tape.

*Proof:* The while loop `while 1 { x = x + 1 }` requires unbounded space for x. ∎

---

## 5. Recursion Theory

### 5.1 Primitive Recursive Functions

**Definition 5.1 (Primitive Recursive):**
A function is primitive recursive if built from:
- Zero: Z(x) = 0
- Successor: S(x) = x + 1
- Projection: Pᵢⁿ(x₁,...,xₙ) = xᵢ
- Composition: h(x̄) = f(g₁(x̄),...,gₘ(x̄))
- Primitive Recursion: h(x̄,0) = f(x̄), h(x̄,y+1) = g(x̄,y,h(x̄,y))

**Theorem 5.1 (Data Functions are Primitive Recursive):**
Every function expressible in the Data Language is primitive recursive.

*Proof:*
- Literals: constant functions
- Variables: projections
- Addition: built from successor
- Negation: built from subtraction (primitive recursive)

The Data Language has no unbounded loops, so it cannot express non-primitive-recursive functions. ∎

**Corollary 5.2 (Data ⊊ Primitive Recursive):**
The Data Language is strictly weaker than primitive recursive functions.

*Proof:* Multiplication is primitive recursive but not expressible in Data Language without Control loops. ∎

### 5.2 Total Recursive Functions

**Definition 5.2 (Total Recursive):**
A function is total recursive if it is computable by a Turing machine that halts on all inputs.

**Theorem 5.3 (Data Functions are Total):**
All Data Language functions are total.

*Proof:* By the Totality Theorem (structural induction). ∎

### 5.3 Partial Recursive Functions

**Definition 5.3 (Partial Recursive):**
A function is partial recursive if computable by a Turing machine (may not halt).

**Theorem 5.4 (Control Functions are Partial Recursive):**
Control Language functions are partial recursive (may not terminate).

*Proof:* While loops may not terminate:
```jtv
while 1 { skip }  // Never terminates
```

### 5.4 Μ-Recursion

**Definition 5.4 (Μ-Operator):**
μy.P(x̄,y) = least y such that P(x̄,y) = 0 (if exists)

**Theorem 5.5 (Control Can Express Μ-Recursion):**
```jtv
fn mu(predicate: Fn(Int) -> Int): Int {
    y = 0
    while predicate(y) != 0 {
        y = y + 1
    }
    return y
}
```

**Theorem 5.6 (Data Cannot Express Μ-Recursion):**
The Data Language cannot express the μ-operator.

*Proof:* The μ-operator requires potentially unbounded search. Data Language has no looping constructs. ∎

---

## 6. Computability Boundaries

### 6.1 The Expressiveness Hierarchy

```
Data Language ⊂ Primitive Recursive ⊂ Total Recursive ⊂ Partial Recursive
     ↑                   ↑                    ↑                   ↑
 Addition only     +Multiplication      +Ackermann         +While loops
                   +Bounded loops       +Total μ           (Control Lang)
```

### 6.2 What Data Language Cannot Compute

**Theorem 6.1 (No Multiplication):**
The Data Language cannot compute x × y directly.

*Proof:* Multiplication is not definable from addition alone without iteration. ∎

**Workaround:** Use Control Language:
```jtv
fn multiply(x: Int, y: Int): Int {
    result = 0
    for i in 0..y {
        result = result + x
    }
    return result
}
```

**Theorem 6.2 (No Exponentiation):**
The Data Language cannot compute xʸ.

**Theorem 6.3 (No Ackermann Function):**
The Data Language cannot compute the Ackermann function.

*Proof:* Ackermann is not primitive recursive. Data Language is weaker than PR. ∎

### 6.3 What Data Language CAN Compute

Despite limitations, Data Language is useful:
- Linear combinations: a₁x₁ + a₂x₂ + ... + aₙxₙ
- Polynomial evaluation (with Control for powers)
- Distance calculations (Manhattan)
- Checksums (additive)
- Financial calculations (with rationals)

### 6.4 The Security-Expressiveness Tradeoff

**Theorem 6.4 (Security-Expressiveness Tradeoff):**
```
More Security ⟺ Less Expressiveness
More Expressiveness ⟺ More Attack Surface
```

JtV's approach:
- Data Language: Maximum security, limited expressiveness
- Control Language: Full expressiveness, controlled interface

The key insight is that *data processing* often needs less power than *control flow*.

---

## 7. Formal Language Theory

### 7.1 Languages Defined by JtV

**Definition 7.1 (JtV Languages):**
```
L_JtV = {w | w is a valid JtV program}
L_Data = {w | w is a valid DataExpr}
L_Ctrl = {w | w is a valid ControlStmt}
```

**Theorem 7.1 (Language Relationships):**
```
L_Data ∩ L_Ctrl = ∅
L_Data ∪ L_Ctrl ⊊ L_JtV
```

### 7.2 Closure Properties

**Theorem 7.2 (Data Language Closure):**
L_Data is closed under:
- Union: e₁ or e₂ ∈ L_Data (disjunction via encoding)
- Concatenation: e₁; e₂ not in L_Data (no sequencing!)
- Kleene star: Not closed (no repetition)

**Theorem 7.3 (Control Language Closure):**
L_Ctrl is closed under:
- Union: if cond { s₁ } else { s₂ }
- Concatenation: s₁; s₂
- Kleene star: while cond { s }

### 7.3 Pumping Lemmas

**Theorem 7.4 (Data Pumping):**
L_Data satisfies the context-free pumping lemma.

*Proof:* As a CF language, standard pumping applies. ∎

---

## 8. Parallel Computation

### 8.1 Data Parallelism

**Theorem 8.1 (Data Expressions are Parallelizable):**
DataExpr evaluation can be parallelized with O(log n) depth.

*Proof:*
1. Parse expression into tree
2. Evaluate leaves (O(1))
3. Propagate values upward (log n levels)
4. Each level can be computed in parallel

Work: O(n), Depth: O(log n), Parallelism: O(n/log n) ∎

### 8.2 PRAM Model

**Definition 8.1 (Data Evaluation in PRAM):**
```
Algorithm EvalParallel(e):
  1. For all leaves l in parallel: val[l] = evaluate(l)
  2. For level = depth-1 downto 0:
       For all nodes n at level in parallel:
         val[n] = val[left(n)] + val[right(n)]
  3. Return val[root]
```

**Theorem 8.2 (EREW Complexity):**
Data evaluation is in EREW-PRAM with O(n) processors and O(log n) time.

### 8.3 Control Language is P-Complete

**Theorem 8.3 (Control is P-Complete):**
Control Language evaluation is P-complete under log-space reductions.

*Proof:* Control Language can simulate any polynomial-time computation (Turing-complete). The circuit value problem (P-complete) reduces to Control evaluation. ∎

**Corollary 8.4 (Control is Inherently Sequential):**
Control Language is not efficiently parallelizable (unless P = NC).

---

## 9. Information Theory

### 9.1 Kolmogorov Complexity

**Definition 9.1 (Descriptive Complexity):**
K(e) = length of shortest program that outputs e

**Theorem 9.1 (Data Expression Complexity):**
For DataExpr e: K(e) ≤ size(e) + O(1)

*Proof:* The expression itself is a description. Add constant for interpreter. ∎

**Theorem 9.2 (Incompressibility of Random Data):**
Most n-bit Data expressions have K(e) ≥ n - O(1).

### 9.2 Entropy

**Definition 9.2 (Evaluation Entropy):**
H(E) = entropy of evaluation outcomes for expression distribution E

**Theorem 9.3 (Data Preserves Information):**
DataExpr evaluation is deterministic, so:
H(eval(e)) = 0 for fixed e, σ

---

## 10. Connections to Other Areas

### 10.1 Database Theory

**Theorem 10.1 (Data Language ≈ RA without θ-join):**
The Data Language is comparable to relational algebra restricted to selection, projection, and union.

### 10.2 Logic Programming

**Theorem 10.2 (Data Language ⊂ Datalog):**
Data Language is strictly weaker than Datalog (no recursion).

### 10.3 Constraint Satisfaction

**Theorem 10.3 (Linear Constraints):**
Data expressions define linear constraints over integers.

**Corollary 10.4 (Satisfiability is Decidable):**
SAT for Data Language constraints is in P (linear programming).

---

## 11. Open Problems

### 11.1 Optimal Parallelization

**TODO:** Investigate optimal work-depth tradeoffs for DataExpr evaluation.

### 11.2 Cache Complexity

**TODO:** Analyze cache-oblivious algorithms for large DataExpr trees.

### 11.3 Quantum Complexity

**TODO:** Classify reversible JtV operations in quantum complexity classes (BQP, QMA).

---

## 12. Summary Tables

### 12.1 Language Properties

| Property | Data Language | Control Language |
|----------|---------------|------------------|
| Chomsky Type | 2 (Context-Free) | 2 (Context-Free) |
| Semantics | Primitive Recursive | Turing-Complete |
| Termination | Always | Undecidable |
| Parallelizable | Yes (NC) | Unlikely (P-complete) |
| Space | O(depth) | Unbounded |
| Time | O(size) | Unbounded |

### 12.2 Decidability Summary

| Problem | Data | Control |
|---------|------|---------|
| Parsing | ✓ O(n) | ✓ O(n) |
| Type Checking | ✓ O(n) | ✓ O(n) |
| Termination | ✓ (always yes) | ✗ |
| Equivalence | ✓ (Presburger) | ✗ |
| Value Prediction | ✓ | ✗ |

---

## References

1. Hopcroft, J., Motwani, R., Ullman, J. (2006). *Automata Theory, Languages, and Computation*
2. Sipser, M. (2012). *Introduction to the Theory of Computation*
3. Arora, S., Barak, B. (2009). *Computational Complexity: A Modern Approach*
4. Cutland, N. (1980). *Computability: An Introduction to Recursive Function Theory*
5. Papadimitriou, C. (1994). *Computational Complexity*
