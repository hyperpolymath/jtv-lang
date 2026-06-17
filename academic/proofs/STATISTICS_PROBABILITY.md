# Statistics and Probability Foundations for Julia the Viper

**SPDX-License-Identifier: CC-BY-SA-4.0

This document establishes the probabilistic and statistical foundations for JtV, including measure theory, probability distributions, statistical inference, and applications to program analysis.

---

## 1. Measure-Theoretic Foundations

### 1.1 Probability Spaces

**Definition 1.1 (Probability Space):**
A probability space is a triple (Ω, ℱ, P) where:
- Ω is the sample space
- ℱ is a σ-algebra of events
- P : ℱ → [0, 1] is a probability measure

**Axioms (Kolmogorov):**
1. P(Ω) = 1
2. P(A) ≥ 0 for all A ∈ ℱ
3. P(⋃ᵢ Aᵢ) = Σᵢ P(Aᵢ) for disjoint Aᵢ

### 1.2 Random Variables

**Definition 1.2 (Random Variable):**
A random variable X : Ω → ℝ is an ℱ-measurable function.

**Definition 1.3 (Expectation):**
```
E[X] = ∫_Ω X(ω) dP(ω)
```

For discrete distributions:
```
E[X] = Σᵢ xᵢ P(X = xᵢ)
```

### 1.3 JtV Probabilistic Semantics

**Definition 1.4 (Probabilistic Data Expression):**
Extend DataExpr with probabilistic choice:
```
e ::= ... | random(n)    -- Uniform random in [0, n)
```

**Note:** This extension requires careful handling to maintain determinism properties.

---

## 2. Probability Distributions

### 2.1 Discrete Distributions

**Definition 2.1 (Discrete Uniform):**
```
X ~ Uniform{0, 1, ..., n-1}
P(X = k) = 1/n for k ∈ {0, ..., n-1}
E[X] = (n-1)/2
Var[X] = (n²-1)/12
```

**Definition 2.2 (Bernoulli):**
```
X ~ Bernoulli(p)
P(X = 1) = p, P(X = 0) = 1-p
E[X] = p
Var[X] = p(1-p)
```

**Definition 2.3 (Binomial):**
```
X ~ Binomial(n, p)
P(X = k) = C(n,k) p^k (1-p)^(n-k)
E[X] = np
Var[X] = np(1-p)
```

**Definition 2.4 (Poisson):**
```
X ~ Poisson(λ)
P(X = k) = e^(-λ) λ^k / k!
E[X] = λ
Var[X] = λ
```

### 2.2 Continuous Distributions

**Definition 2.5 (Uniform Continuous):**
```
X ~ Uniform(a, b)
f(x) = 1/(b-a) for x ∈ [a, b]
E[X] = (a+b)/2
Var[X] = (b-a)²/12
```

**Definition 2.6 (Normal/Gaussian):**
```
X ~ Normal(μ, σ²)
f(x) = (1/√(2πσ²)) exp(-(x-μ)²/(2σ²))
E[X] = μ
Var[X] = σ²
```

**Definition 2.7 (Exponential):**
```
X ~ Exponential(λ)
f(x) = λ exp(-λx) for x ≥ 0
E[X] = 1/λ
Var[X] = 1/λ²
```

### 2.3 JtV Distribution Implementations

```jtv
// Discrete uniform sampling (using Control Language)
fn uniform_int(n: Int): Int {
    // Placeholder: actual random requires external source
    return hash(current_time()) % n
}

// Bernoulli trial simulation
fn bernoulli(p: Rational): Int {
    if uniform_int(1000) < p.num * (1000 / p.den) {
        return 1
    } else {
        return 0
    }
}

// Binomial simulation
fn binomial(n: Int, p: Rational): Int {
    count = 0
    for i in 0..n {
        count = count + bernoulli(p)
    }
    return count
}
```

---

## 3. Statistical Estimators

### 3.1 Point Estimation

**Definition 3.1 (Sample Mean):**
```
X̄ = (1/n) Σᵢ Xᵢ
```

**Theorem 3.1 (Unbiasedness):**
E[X̄] = μ (sample mean is unbiased for population mean)

**Definition 3.2 (Sample Variance):**
```
S² = (1/(n-1)) Σᵢ (Xᵢ - X̄)²
```

**Theorem 3.2 (Unbiasedness of S²):**
E[S²] = σ² (sample variance is unbiased for population variance)

### 3.2 JtV Statistics Functions

```jtv
// Sample mean (Data Language portion for summation)
fn sum(data: List<Float>): Float @pure {
    result = 0.0
    for x in data {
        result = result + x
    }
    return result
}

fn mean(data: List<Float>): Float @pure {
    return sum(data) / length(data)
}

// Sample variance (requires subtraction and multiplication)
fn variance(data: List<Float>): Float {
    m = mean(data)
    sum_sq = 0.0
    for x in data {
        diff = x + (-(m))  // x - m using addition
        sum_sq = sum_sq + diff * diff
    }
    return sum_sq / (length(data) + (-(1)))  // n - 1
}

// Standard deviation
fn std_dev(data: List<Float>): Float {
    return sqrt(variance(data))
}
```

### 3.3 Confidence Intervals

**Definition 3.3 (Confidence Interval):**
A (1-α) confidence interval for parameter θ:
```
P(L ≤ θ ≤ U) = 1 - α
```

**Theorem 3.3 (Normal CI for Mean):**
For X ~ Normal(μ, σ²) with known σ:
```
CI = [X̄ - z_{α/2} σ/√n, X̄ + z_{α/2} σ/√n]
```

---

## 4. Hypothesis Testing

### 4.1 Framework

**Definition 4.1 (Hypothesis Test):**
- H₀: Null hypothesis
- H₁: Alternative hypothesis
- α: Significance level
- Test statistic T
- Rejection region R

**Definition 4.2 (p-value):**
P(T ≥ t | H₀) where t is observed test statistic.

### 4.2 Common Tests

**Definition 4.3 (Z-test):**
```
Z = (X̄ - μ₀) / (σ/√n)
```
Under H₀: Z ~ Normal(0, 1)

**Definition 4.4 (t-test):**
```
t = (X̄ - μ₀) / (S/√n)
```
Under H₀: t ~ t_{n-1}

**Definition 4.5 (Chi-squared Test):**
```
χ² = Σᵢ (Oᵢ - Eᵢ)² / Eᵢ
```
For goodness-of-fit.

### 4.3 JtV Hypothesis Test Implementation

```jtv
// Z-test for mean
fn z_test(data: List<Float>, mu0: Float, sigma: Float): Float {
    x_bar = mean(data)
    n = length(data)
    z = (x_bar + (-(mu0))) / (sigma / sqrt(n))
    return z
}

// Two-tailed p-value (approximation)
fn two_tailed_p(z: Float): Float {
    // Standard normal CDF approximation
    // Using Abramowitz and Stegun approximation
    return 2 * (1 + (-(normal_cdf(abs(z)))))
}
```

---

## 5. Regression Analysis

### 5.1 Linear Regression

**Definition 5.1 (Simple Linear Regression):**
```
Y = β₀ + β₁X + ε
```
where ε ~ Normal(0, σ²)

**Theorem 5.1 (OLS Estimators):**
```
β̂₁ = Σ(xᵢ - x̄)(yᵢ - ȳ) / Σ(xᵢ - x̄)²
β̂₀ = ȳ - β̂₁x̄
```

### 5.2 JtV Regression Implementation

```jtv
// Simple linear regression
fn linear_regression(x: List<Float>, y: List<Float>): (Float, Float) {
    n = length(x)
    x_bar = mean(x)
    y_bar = mean(y)

    numerator = 0.0
    denominator = 0.0

    for i in 0..n {
        x_diff = x[i] + (-(x_bar))
        y_diff = y[i] + (-(y_bar))
        numerator = numerator + x_diff * y_diff
        denominator = denominator + x_diff * x_diff
    }

    beta1 = numerator / denominator
    beta0 = y_bar + (-(beta1 * x_bar))

    return (beta0, beta1)
}
```

### 5.3 Multiple Regression

**Definition 5.2 (Multiple Linear Regression):**
```
Y = Xβ + ε
```
where X is the design matrix, β is parameter vector.

**Theorem 5.2 (OLS Solution):**
```
β̂ = (X'X)⁻¹X'Y
```

---

## 6. Bayesian Statistics

### 6.1 Bayes' Theorem

**Theorem 6.1 (Bayes' Theorem):**
```
P(θ | data) = P(data | θ) P(θ) / P(data)
```

- P(θ): Prior
- P(data | θ): Likelihood
- P(θ | data): Posterior

### 6.2 Conjugate Priors

| Likelihood | Prior | Posterior |
|------------|-------|-----------|
| Binomial | Beta | Beta |
| Normal (known σ) | Normal | Normal |
| Poisson | Gamma | Gamma |
| Multinomial | Dirichlet | Dirichlet |

### 6.3 JtV Bayesian Updates

```jtv
// Beta-Binomial conjugacy
fn beta_update(
    prior_alpha: Float,
    prior_beta: Float,
    successes: Int,
    failures: Int
): (Float, Float) {
    post_alpha = prior_alpha + successes
    post_beta = prior_beta + failures
    return (post_alpha, post_beta)
}

// Posterior mean
fn beta_mean(alpha: Float, beta: Float): Float {
    return alpha / (alpha + beta)
}
```

---

## 7. Probabilistic Program Analysis

### 7.1 Probabilistic Semantics

**Definition 7.1 (Distribution Transformer):**
A probabilistic program transforms input distributions to output distributions:
```
⟦P⟧ : Dist(State) → Dist(State)
```

### 7.2 Expected Value Analysis

**Theorem 7.1 (Linearity of Expectation for Data Language):**
For DataExpr with random variables:
```
E[e₁ + e₂] = E[e₁] + E[e₂]
```

*Proof:* By linearity of expectation. The Data Language's addition-only nature preserves this property. ∎

### 7.3 Variance Analysis

**Theorem 7.2 (Variance of Sum for Independent Variables):**
If X, Y are independent:
```
Var[X + Y] = Var[X] + Var[Y]
```

**Corollary 7.3 (Data Expression Variance):**
For independent terms in a DataExpr:
```
Var[t₁ + t₂ + ... + tₙ] = Var[t₁] + Var[t₂] + ... + Var[tₙ]
```

---

## 8. Complexity of Statistical Algorithms

### 8.1 Time Complexity

| Algorithm | Time Complexity |
|-----------|-----------------|
| Sample mean | O(n) |
| Sample variance | O(n) |
| Linear regression | O(n) |
| Sorting (for median) | O(n log n) |
| Matrix inversion (regression) | O(n³) |

### 8.2 Space Complexity

| Algorithm | Space Complexity |
|-----------|------------------|
| Sample mean | O(1) streaming |
| Sample variance | O(1) Welford's |
| Histogram | O(bins) |
| Regression | O(features²) |

### 8.3 JtV Streaming Statistics

```jtv
// Welford's online variance algorithm
struct RunningStats {
    count: Int
    mean: Float
    m2: Float  // Sum of squared deviations
}

fn update_stats(stats: RunningStats, x: Float): RunningStats {
    new_count = stats.count + 1
    delta = x + (-(stats.mean))
    new_mean = stats.mean + delta / new_count
    delta2 = x + (-(new_mean))
    new_m2 = stats.m2 + delta * delta2

    return RunningStats(new_count, new_mean, new_m2)
}

fn get_variance(stats: RunningStats): Float {
    if stats.count < 2 {
        return 0.0
    }
    return stats.m2 / (stats.count + (-(1)))
}
```

---

## 9. Monte Carlo Methods

### 9.1 Monte Carlo Integration

**Theorem 9.1 (Monte Carlo Estimator):**
For integral I = ∫ f(x) dx over domain D:
```
Î = (Volume(D)/n) Σᵢ f(Xᵢ)
```
where Xᵢ ~ Uniform(D).

**Theorem 9.2 (Convergence Rate):**
```
|Î - I| = O(1/√n)
```

### 9.2 Markov Chain Monte Carlo (MCMC)

**Definition 9.1 (Metropolis-Hastings):**
1. Propose x' from q(x' | x)
2. Accept with probability min(1, α) where:
   ```
   α = (π(x') q(x | x')) / (π(x) q(x' | x))
   ```

### 9.3 JtV Monte Carlo Pi Estimation

```jtv
// Estimate π using Monte Carlo
fn estimate_pi(samples: Int): Float {
    inside = 0
    for i in 0..samples {
        x = random_float()  // Uniform(0, 1)
        y = random_float()
        if x * x + y * y <= 1 {
            inside = inside + 1
        }
    }
    return 4 * inside / samples
}
```

---

## 10. Information Theory

### 10.1 Entropy

**Definition 10.1 (Shannon Entropy):**
```
H(X) = -Σᵢ p(xᵢ) log₂ p(xᵢ)
```

**Properties:**
- H(X) ≥ 0
- H(X) = 0 iff X is deterministic
- H(X) ≤ log₂ |X| (maximized for uniform)

### 10.2 Mutual Information

**Definition 10.2 (Mutual Information):**
```
I(X; Y) = H(X) + H(Y) - H(X, Y)
         = H(X) - H(X | Y)
```

### 10.3 JtV Entropy Calculation

```jtv
// Empirical entropy of a distribution
fn entropy(probs: List<Float>): Float {
    h = 0.0
    for p in probs {
        if p > 0 {
            h = h + (-(p * log2(p)))
        }
    }
    return h
}
```

---

## 11. Applications to JtV

### 11.1 Probabilistic Type Checking

**TODO:** Extend type system with probabilistic types:
```
x : Int @ P(x > 0) = 0.95
```

### 11.2 Probabilistic Termination

**Definition 11.1 (Almost Sure Termination):**
A program terminates almost surely if:
```
P(terminates) = 1
```

**Theorem 11.1 (Data Language Almost Sure Termination):**
All DataExpr terminate with probability 1.

*Proof:* DataExpr terminate deterministically. P(terminates) = 1. ∎

### 11.3 Statistical Testing Framework

```jtv
// Property-based testing with statistical guarantees
fn property_test(
    property: Fn(Input) -> Bool,
    generator: Fn() -> Input,
    samples: Int,
    confidence: Float
): TestResult {
    successes = 0
    for i in 0..samples {
        input = generator()
        if property(input) {
            successes = successes + 1
        }
    }

    // Wilson score interval for binomial proportion
    p_hat = successes / samples
    // ... compute confidence interval ...

    return TestResult(p_hat, lower_bound, upper_bound)
}
```

---

## 12. Machine Learning Foundations

### 12.1 Empirical Risk Minimization

**Definition 12.1 (Empirical Risk):**
```
R̂(h) = (1/n) Σᵢ L(h(xᵢ), yᵢ)
```

### 12.2 PAC Learning

**Definition 12.2 (PAC Learnable):**
A concept class C is PAC learnable if there exists an algorithm that, for any ε, δ > 0, produces a hypothesis h with:
```
P(error(h) > ε) < δ
```
using poly(1/ε, 1/δ, n) samples.

### 12.3 VC Dimension

**Definition 12.3 (VC Dimension):**
The maximum number of points that can be shattered by a hypothesis class.

**Theorem 12.1 (VC Generalization Bound):**
```
error(h) ≤ R̂(h) + O(√(d log(n/d) / n))
```
where d is VC dimension.

---

## 13. Open Problems

### 13.1 Probabilistic JtV Extension

**TODO:** Design probabilistic Data Language extension:
- Maintain totality guarantees
- Preserve security properties
- Enable probabilistic reasoning

### 13.2 Statistical Verification

**TODO:** Integrate statistical testing with formal verification:
- QuickCheck-style property testing
- Statistical model checking
- Probabilistic assertions

### 13.3 Uncertainty Quantification

**TODO:** Track uncertainty through computations:
- Interval arithmetic for floats
- Probability distributions for outputs
- Sensitivity analysis

---

## 14. Summary

Statistical and probabilistic foundations for JtV:

| Concept | Application | Status |
|---------|-------------|--------|
| Probability spaces | Semantic foundation | Theoretical |
| Distributions | Sampling/simulation | Implementable |
| Estimation | Statistics library | Implementable |
| Hypothesis testing | Analysis tools | Implementable |
| Bayesian inference | Uncertainty modeling | Implementable |
| Monte Carlo | Numerical integration | Implementable |
| Information theory | Compression, ML | Implementable |
| PAC learning | ML guarantees | Theoretical |

The Data Language's linearity makes it particularly amenable to probabilistic analysis—linearity of expectation holds directly.

---

## References

1. Casella, G., Berger, R.L. (2002). *Statistical Inference*
2. Murphy, K.P. (2012). *Machine Learning: A Probabilistic Perspective*
3. Ross, S.M. (2019). *Introduction to Probability Models*
4. Wasserman, L. (2004). *All of Statistics*
5. Cover, T.M., Thomas, J.A. (2006). *Elements of Information Theory*
6. Kozen, D. (1981). Semantics of probabilistic programs. *JCSS*
