import Lake
open Lake DSL

package JtvProofs where
  -- Lean 4 package for JtV formal semantics proofs
  leanOptions := #[
    ⟨`pp.unicode.fun, true⟩,
    ⟨`autoImplicit, false⟩
  ]

@[default_target]
lean_lib JtvCore where
  -- Core definitions: semantic domains, evaluation functions
  roots := #[`JtvCore]

lean_lib JtvTheorems where
  -- Theorems: totality, determinism, security, algebraic properties
  roots := #[`JtvTheorems]

lean_lib JtvOperational where
  -- Operational semantics: small-step and big-step
  roots := #[`JtvOperational]

lean_lib JtvTypes where
  -- Type system: typing rules, purity, type soundness
  roots := #[`JtvTypes]

lean_lib JtvSecurity where
  -- Security properties: injection impossibility, sandboxing
  roots := #[`JtvSecurity]

lean_lib JtvEcho where
  -- Echo Types: structured-loss lattice + fibre correspondence
  -- (aligns with the echo-types Agda library / EchoTypes.jl)
  roots := #[`JtvEcho]

-- Build all proofs
@[default_target]
lean_lib JtvAll where
  roots := #[`JtvCore, `JtvTheorems, `JtvOperational, `JtvTypes, `JtvSecurity, `JtvEcho]
