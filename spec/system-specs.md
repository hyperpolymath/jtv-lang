# SPDX-License-Identifier: CC-BY-SA-4.0
# Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

# Julia the Viper — System Specifications

Julia the Viper (JtV) is an interpreted language with a Rust interpreter and
pest-based parser, following a Harvard architecture where code and data are
separated.

## Memory Model

JtV uses a tree-walking interpreter with value-based memory management:

- **Cloned values**: The interpreter clones values on assignment and when passing
  arguments to functions. There is no shared mutable state between different parts
  of the program.
- **No shared state**: The Harvard architecture enforces separation between code
  (instruction memory) and data (data memory). Data cannot reference or modify code,
  and code cannot treat data as executable.
- **Stack-allocated numerics**: Integer and float values are stored directly on the
  Rust stack as native types (`i64`, `f64`). No heap allocation is needed for
  numeric operations.
- **Heap-allocated compounds**: Strings, lists, and maps are heap-allocated via
  Rust's standard allocator. They are owned by the environment that creates them.
- **Environment ownership**: Each scope owns its local variables. When a scope exits,
  its environment is dropped and all owned values are deallocated.
- **No garbage collector**: Memory is managed through Rust's ownership system in the
  interpreter. Values are dropped when their owning scope exits.
- **Copy semantics**: All values use deep-copy semantics. Passing a list to a
  function creates an independent copy; mutations inside the function do not affect
  the caller's version.

## Concurrency Model

JtV does not currently support concurrency:

- **Sequential execution**: The Harvard architecture is inherently sequential for
  data operations. Instructions execute one at a time in program order.
- **No threads or async**: There are no threading primitives, async/await, or
  coroutine mechanisms in the current implementation.
- **Reverse blocks**: Control flow includes reverse blocks that execute statements
  in reverse order. These are sequential — they do not introduce parallelism but
  provide unique control flow patterns.
- **Future considerations**: If concurrency is added, the Harvard architecture's
  data/code separation provides a natural isolation boundary. Each concurrent
  context could have its own data memory while sharing code memory (read-only).
- **Deterministic execution**: The lack of concurrency guarantees fully deterministic
  program execution. Given the same inputs, a JtV program always produces the same
  outputs in the same order.

## Effect System

JtV uses a purity annotation system for effect tracking:

- **`@total` annotation**: Marks a function as total — it must terminate for all
  valid inputs and produce no side effects. The interpreter enforces termination
  through recursion depth limits.
- **`@pure` annotation**: Marks a function as pure — it produces no side effects
  but is not required to terminate. Pure functions may not perform IO, modify
  global state, or call impure functions.
- **`impure` default**: Functions without annotations are implicitly impure. They
  may perform any effect including IO, mutation, and non-termination.
- **IO as an effect**: Input/output operations (print, read, file access) are
  classified as effects. Only impure functions may perform IO directly.
- **Effect violations**: Calling an impure function from a `@pure` or `@total`
  function is a compile-time error (caught during the analysis pass before
  interpretation).
- **Gradual adoption**: The purity system is opt-in. Existing code without
  annotations continues to work as impure by default. Annotations are added
  incrementally as the codebase matures.
- **No effect polymorphism**: Functions cannot be generic over their effect. A
  higher-order function that accepts a callback must choose whether it requires
  the callback to be pure or allows it to be impure.

## Module System

JtV has a minimal, file-based module system:

- **File-based modules**: Each source file acts as an implicit module. There is no
  explicit `module` declaration.
- **No explicit exports**: All top-level definitions in a file are visible to
  importers. There is no access control mechanism.
- **File inclusion**: Other files are loaded via a basic include mechanism that
  evaluates the target file in the current scope.
- **No namespacing**: Included definitions are placed directly in the importing
  scope. Name collisions are resolved by last-definition-wins.
- **No package manager**: Dependencies are managed by file path. There is no
  registry, version resolution, or dependency management tooling.
- **Future plans**: A proper module system with explicit exports, namespacing, and
  qualified imports is planned. The Harvard architecture suggests a natural split
  between code modules (shared, read-only) and data modules (per-context, mutable).
- **Standard library**: Built-in functions are available globally without import.
  They are implemented in Rust and registered in the interpreter's root environment
  at startup.
