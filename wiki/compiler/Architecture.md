<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# JtV Compiler Architecture

The JtV compiler transforms source code into executable form while enforcing the Harvard Architecture guarantees.

## Overview

```
Source Code (.jtv)
        │
        ▼
┌───────────────────┐
│      LEXER        │  Tokenization
└───────────────────┘
        │ Tokens
        ▼
┌───────────────────┐
│      PARSER       │  Syntax Analysis
└───────────────────┘
        │ AST
        ▼
┌───────────────────┐
│    NAME RESOLVER  │  Scope Analysis
└───────────────────┘
        │ Resolved AST
        ▼
┌───────────────────┐
│   TYPE CHECKER    │  Type Analysis
└───────────────────┘
        │ Typed AST
        ▼
┌───────────────────┐
│  PURITY CHECKER   │  Purity Analysis
└───────────────────┘
        │ Verified AST
        ▼
┌───────────────────┐
│     OPTIMIZER     │  IR Optimization
└───────────────────┘
        │ Optimized IR
        ▼
┌───────────────────┐
│   CODE GENERATOR  │  Target Code
└───────────────────┘
        │
        ▼
    Executable/WASM
```

## Frontend

### Lexer

Transforms source text into tokens.

**Input**: Source code string
**Output**: Token stream

```rust
pub enum Token {
    // Literals
    IntLit(i64),
    FloatLit(f64),
    RationalLit(i64, i64),
    ComplexLit(f64, f64),
    StringLit(String),

    // Identifiers and Keywords
    Ident(String),
    Keyword(Keyword),

    // Operators
    Plus, Equals, DoubleEquals,
    Less, Greater, LessEq, GreaterEq,
    And, Or, Not,

    // Delimiters
    LParen, RParen, LBrace, RBrace,
    Comma, Colon, Semicolon, Arrow,

    // Special
    EOF,
}
```

**Key Features**:
- Recognizes 7 number formats (decimal, hex, binary, rational, complex, float, symbolic)
- Handles UTF-8 source files
- Tracks source positions for error reporting
- Skips whitespace and comments

### Parser

Builds Abstract Syntax Tree from tokens.

**Input**: Token stream
**Output**: Untyped AST

```rust
pub enum DataExpr {
    Lit(Value),
    Var(String),
    Add(Box<DataExpr>, Box<DataExpr>),
    Call(String, Vec<DataExpr>),
}

pub enum ControlStmt {
    Skip,
    Assign(String, DataExpr),
    Seq(Box<ControlStmt>, Box<ControlStmt>),
    If(Condition, Box<ControlStmt>, Box<ControlStmt>),
    While(Condition, Box<ControlStmt>),
    For(String, DataExpr, DataExpr, Box<ControlStmt>),
    Print(DataExpr),
    Return(Option<DataExpr>),
    Reverse(Vec<RevOp>),
}

pub struct Function {
    pub name: String,
    pub purity: Option<Purity>,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: ControlStmt,
}
```

**Key Features**:
- Enforces Harvard Architecture at grammar level
- Separate productions for DataExpr and ControlStmt
- No parse tree for `eval()` or code-from-strings

### Name Resolver

Resolves identifiers to their definitions.

**Input**: Untyped AST
**Output**: Resolved AST with symbol table

```rust
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

pub struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<usize>,
}

pub enum Symbol {
    Variable { ty: Type, mutable: bool },
    Function { signature: FunctionSig },
    Type { definition: TypeDef },
}
```

**Key Features**:
- Builds scope hierarchy
- Detects undefined variables
- Detects duplicate definitions
- Supports shadowing with warnings

## Middle-End

### Type Checker

Infers and checks types.

**Input**: Resolved AST
**Output**: Typed AST

```rust
pub struct TypeChecker {
    context: TypeContext,
}

impl TypeChecker {
    pub fn infer_data_expr(&self, expr: &DataExpr) -> Result<Type, TypeError> {
        match expr {
            DataExpr::Lit(v) => Ok(v.get_type()),
            DataExpr::Var(x) => self.context.lookup(x),
            DataExpr::Add(e1, e2) => {
                let t1 = self.infer_data_expr(e1)?;
                let t2 = self.infer_data_expr(e2)?;
                self.coerce(t1, t2)
            }
            DataExpr::Call(f, args) => {
                let sig = self.context.lookup_function(f)?;
                self.check_call(sig, args)
            }
        }
    }
}
```

**Key Features**:
- Bidirectional type inference
- Automatic coercion between number types
- Function signature checking
- Generic type instantiation (future)

### Purity Checker

Verifies purity annotations.

**Input**: Typed AST
**Output**: Verified AST

```rust
pub struct PurityChecker {
    function_purities: HashMap<String, Purity>,
}

impl PurityChecker {
    pub fn check_function(&self, func: &Function) -> Result<Purity, PurityError> {
        let body_purity = self.analyze_stmt(&func.body)?;

        if let Some(annotation) = &func.purity {
            if body_purity > *annotation {
                return Err(PurityError::ViolatesAnnotation {
                    expected: annotation.clone(),
                    actual: body_purity,
                });
            }
        }

        Ok(body_purity)
    }

    pub fn check_data_context(&self, expr: &DataExpr) -> Result<(), PurityError> {
        // Verify all function calls are pure/total
        for call in expr.calls() {
            let callee_purity = self.function_purities.get(&call.name)?;
            if *callee_purity == Purity::Impure {
                return Err(PurityError::ImpureInDataContext(call.name.clone()));
            }
        }
        Ok(())
    }
}
```

**Key Features**:
- Enforces @total, @pure annotations
- Checks Data context purity constraints
- Prevents I/O in pure functions
- Prevents loops in total functions

### Optimizer

Transforms IR for efficiency.

**Input**: Verified AST
**Output**: Optimized IR

```rust
pub struct Optimizer {
    passes: Vec<Box<dyn OptimizationPass>>,
}

pub trait OptimizationPass {
    fn optimize(&self, ir: &mut IR) -> bool;
}
```

**Optimization Passes**:

1. **Constant Folding**: Evaluate compile-time constants
   ```jtv
   x = 2 + 3  // Optimized to: x = 5
   ```

2. **Dead Code Elimination**: Remove unreachable code
   ```jtv
   if false { x = 1 }  // Removed entirely
   ```

3. **Common Subexpression Elimination**:
   ```jtv
   a = x + y
   b = x + y  // Reuses computation of x + y
   ```

4. **Loop Unrolling** (bounded loops only):
   ```jtv
   for i in 0..3 { sum = sum + i }
   // Unrolled to:
   sum = sum + 0
   sum = sum + 1
   sum = sum + 2
   ```

5. **Inlining** (small pure functions):
   ```jtv
   @total fn inc(x: Int): Int { return x + 1 }
   y = inc(5)  // Inlined to: y = 5 + 1
   ```

## Backend

### Code Generator

Produces target code from IR.

**Targets**:
- **Interpreter**: Direct AST execution
- **Bytecode**: Stack-based VM instructions
- **WASM**: WebAssembly binary
- **Native**: LLVM IR → Machine code (future)

#### WASM Code Generator

```rust
pub struct WasmCodeGen {
    module: walrus::Module,
}

impl WasmCodeGen {
    pub fn generate(&mut self, program: &Program) -> Vec<u8> {
        // Generate function imports
        self.generate_imports();

        // Generate each function
        for func in &program.functions {
            self.generate_function(func);
        }

        // Generate main entry point
        self.generate_main(&program.main);

        // Emit binary
        self.module.emit_wasm()
    }

    fn generate_data_expr(&mut self, expr: &DataExpr, fb: &mut FunctionBuilder) {
        match expr {
            DataExpr::Lit(Value::Int(n)) => fb.i64_const(*n),
            DataExpr::Var(x) => fb.local_get(self.locals[x]),
            DataExpr::Add(e1, e2) => {
                self.generate_data_expr(e1, fb);
                self.generate_data_expr(e2, fb);
                fb.binop(BinaryOp::I64Add);
            }
            // ...
        }
    }
}
```

## Error Handling

Errors are categorized by compilation phase:

```rust
pub enum CompileError {
    Lex(LexError),
    Parse(ParseError),
    Name(NameError),
    Type(TypeError),
    Purity(PurityError),
}

pub struct ErrorReport {
    pub level: Level,  // Error, Warning, Hint
    pub code: String,  // E001, W002, etc.
    pub message: String,
    pub location: SourceSpan,
    pub notes: Vec<Note>,
    pub suggestions: Vec<Suggestion>,
}
```

### Error Recovery

The compiler continues after errors where possible:
- Lexer: Skip invalid characters
- Parser: Synchronize at statement boundaries
- Type checker: Use error type for cascading

## Incremental Compilation

For IDE integration, the compiler supports incremental builds:

```rust
pub struct IncrementalCompiler {
    cache: CompilationCache,
}

impl IncrementalCompiler {
    pub fn update(&mut self, file: &Path, changes: &[TextChange]) {
        // Invalidate affected cache entries
        self.cache.invalidate(file);

        // Reparse only changed functions
        let affected = self.find_affected_functions(changes);
        for func in affected {
            self.recompile_function(func);
        }
    }
}
```

## Pipeline Configuration

```rust
pub struct CompilerConfig {
    pub target: Target,
    pub optimization_level: OptLevel,
    pub debug_info: bool,
    pub strict_mode: bool,
    pub emit_source_maps: bool,
}

pub enum Target {
    Interpreter,
    Bytecode,
    Wasm { optimize: bool },
    Native { target_triple: String },
}

pub enum OptLevel {
    None,    // -O0: No optimization
    Basic,   // -O1: Basic optimizations
    Full,    // -O2: All optimizations
    Size,    // -Os: Optimize for size
}
```

## See Also

- [Lexer Implementation](../internals/Lexer.md)
- [Parser Implementation](../internals/Parser.md)
- [Type System](../internals/Type-System.md)
- [Purity System](../internals/Purity-System.md)
- [WASM Target](./WASM.md)
