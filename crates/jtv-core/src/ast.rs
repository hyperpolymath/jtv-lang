// Abstract Syntax Tree for JtV
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<TopLevel>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TopLevel {
    Module(ModuleDecl),
    Import(ImportStmt),
    ExternCoproc(ExternCoprocBlock),
    Function(FunctionDecl),
    Control(ControlStmt),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleDecl {
    pub name: String,
    pub body: Vec<TopLevel>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportStmt {
    pub path: Vec<String>,
    pub alias: Option<String>,
}

// ===== COPROCESSOR DECLARATIONS =====

/// A top-level `extern coproc <gate-name> { ... }` block.
///
/// `gate_name` is the name of a gate in a companion `.pata` file.
/// PataCL evaluates the gate at compile time; if dead, this block is
/// dropped entirely. If live, `items` are registered in the function
/// namespace as `ExternCoproc` entries.
///
/// `resolved` is `None` until the PataCL resolution pass runs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternCoprocBlock {
    pub gate_name: String,
    pub items: Vec<CoprocItem>,
    pub resolved: Option<CoprocResolution>,
}

/// Outcome of PataCL gate evaluation for one `extern coproc` block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoprocResolution {
    pub live: bool,
    pub family: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoprocItem {
    Intrinsic(CoprocIntrinsic),
    Insn(CoprocInsn),
}

/// A coprocessor intrinsic — a named builtin with no encoding literal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoprocIntrinsic {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: TypeAnnotation,
    pub purity: Purity,
}

/// A custom instruction with an optional opaque encoding string.
/// The encoding is passed verbatim to the assembler backend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoprocInsn {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: TypeAnnotation,
    pub purity: Purity,
    pub encoding: Option<String>,
}

// ===== CONTROL LANGUAGE (Turing-complete) =====

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlStmt {
    Assignment(Assignment),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Return(Option<DataExpr>),
    Print(Vec<DataExpr>),
    ReverseBlock(ReverseBlock),
    /// v2 — `reversible { stmts } -> tok`: runs the body forward, recording a
    /// reversal log, and (optionally) binds a reversal token to `tok`.
    /// Phase 2: grammar pending — produced by the interpreter API, not yet by
    /// the parser.
    ReversibleBlock(ReversibleBlockStmt),
    /// v2 — `reverse tok`: linearly consumes a reversal token, applying the
    /// inverse of its recorded operations. Phase 2: grammar pending.
    ReverseToken(String),
    /// v2 — `abandon tok`: linearly consumes a reversal token, discarding its
    /// recorded operations without applying inverses. Phase 2: grammar pending.
    AbandonToken(String),
    Block(Vec<ControlStmt>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub target: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfStmt {
    pub condition: ControlExpr,
    pub then_branch: Vec<ControlStmt>,
    pub else_branch: Option<Vec<ControlStmt>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhileStmt {
    pub condition: ControlExpr,
    pub body: Vec<ControlStmt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForStmt {
    pub variable: String,
    pub range: RangeExpr,
    pub body: Vec<ControlStmt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReverseBlock {
    pub body: Vec<ReversibleStmt>,
}

/// v2 — body of a `reversible { ... } -> tok` statement.
///
/// The forward pass runs `body` (the same reversible statements as a
/// `reverse` block), recording a concrete operation log. `token_binding` is
/// the optional name the resulting reversal token is bound to; if `None` the
/// log is effectively abandoned (forward state is committed, no token issued).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReversibleBlockStmt {
    pub body: Vec<ReversibleStmt>,
    pub token_binding: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReversibleStmt {
    AddAssign(String, DataExpr), // x += expr
    SubAssign(String, DataExpr), // x -= expr (auto-generated in reverse)
    If(IfStmt),
}

// ===== EXPRESSIONS =====

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Data(DataExpr),
    Control(ControlExpr),
}

// DATA LANGUAGE (Total, addition-only)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataExpr {
    Number(Number),
    StringLit(String),
    Identifier(String),
    Add(Box<DataExpr>, Box<DataExpr>),
    Negate(Box<DataExpr>),
    FunctionCall(FunctionCall),
    List(Vec<DataExpr>),
    Tuple(Vec<DataExpr>),
}

// CONTROL EXPRESSIONS (can have side effects)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlExpr {
    Data(DataExpr),
    Comparison(Box<DataExpr>, Comparator, Box<DataExpr>),
    Logical(Box<ControlExpr>, LogicalOp, Box<ControlExpr>),
    Not(Box<ControlExpr>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Comparator {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogicalOp {
    And,
    Or,
}

// ===== FUNCTIONS =====

/// The three loss classes of the Echo effect taxonomy (spec v2 §8–9); lattice
/// order `Safe ⊑ Neutral ⊑ Breaking`. Defined here, rather than in `echo.rs`, so
/// the AST can carry an `@echo(...)` annotation without a module cycle
/// (`echo.rs` imports `ast`). `echo.rs` re-exports it and owns its lattice ops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Echo {
    /// No loss — injective / reversible.
    Safe,
    /// Structured loss — non-total erasure, residue retained.
    Neutral,
    /// Total erasure — irreversible.
    Breaking,
}

/// The epistemic grade (ADR-0009 D2): how much a function reveals about its
/// inputs; lattice order `Opaque ⊑ Partial ⊑ Transparent`. Defined here for the
/// same cycle reason as `Echo`; `epistemic.rs` re-exports it and owns its ops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Epistemic {
    /// Reveals nothing about the inputs.
    Opaque,
    /// Reveals a bounded function of the inputs.
    Partial,
    /// Reveals the inputs fully (the output determines the input).
    Transparent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeAnnotation>,
    pub purity: Purity,
    /// Optional `@echo(...)` grade ceiling (ADR-0009 D1). The checker verifies
    /// the inferred (composed) echo does not exceed it: `inferred ⊑ annotated`.
    pub echo_annotation: Option<Echo>,
    /// Optional `@epi(...)` epistemic-grade ceiling (ADR-0009 D2), checked the
    /// same way as `echo_annotation`.
    pub epi_annotation: Option<Epistemic>,
    pub body: Vec<ControlStmt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Purity {
    Pure,   // @pure - no loops, no IO
    Total,  // @total - guaranteed to terminate
    Impure, // default - may loop, may have side effects
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Module path, e.g., ["Math"] for Math.add
    pub module: Option<Vec<String>>,
    /// Function name
    pub name: String,
    pub args: Vec<DataExpr>,
}

impl FunctionCall {
    /// Returns the fully qualified name (e.g., "Math::add" or just "add")
    pub fn qualified_name(&self) -> String {
        match &self.module {
            Some(path) => format!("{}::{}", path.join("::"), self.name),
            None => self.name.clone(),
        }
    }
}

// ===== TYPE SYSTEM =====

/// An optional `(echo, epi)` effect row decorating a function *type* (ADR-0009
/// D1/D3), written with the same `@echo(...)` / `@epi(...)` markers as on
/// declarations. `None` on an axis means unspecified (unconstrained).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EffectGrade {
    pub echo: Option<Echo>,
    pub epi: Option<Epistemic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeAnnotation {
    Basic(BasicType),
    List(Box<TypeAnnotation>),
    Tuple(Vec<TypeAnnotation>),
    /// `@echo(...) @epi(...) Fn(params) -> ret` — the trailing `EffectGrade`
    /// carries the optional grade row (ADR-0009 D1/D3).
    Function(Vec<TypeAnnotation>, Box<TypeAnnotation>, EffectGrade),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BasicType {
    Int,
    Float,
    Rational,
    Complex,
    Hex,
    Binary,
    Symbolic,
    Bool,
    String,
}

// ===== LITERALS =====

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Number {
    Int(i64),
    Float(f64),
    Rational(i64, i64), // numerator, denominator
    Complex(f64, f64),  // real, imaginary
    Hex(String),
    Binary(String),
    Symbolic(String), // For symbolic math (e.g., "x", "pi")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RangeExpr {
    pub start: Box<DataExpr>,
    pub end: Box<DataExpr>,
    pub step: Option<Box<DataExpr>>,
}

// ===== VISITOR PATTERN FOR TRAVERSAL =====

pub trait Visitor {
    fn visit_program(&mut self, program: &Program);
    fn visit_control_stmt(&mut self, stmt: &ControlStmt);
    fn visit_data_expr(&mut self, expr: &DataExpr);
    fn visit_function_decl(&mut self, func: &FunctionDecl);
}

impl Program {
    pub fn new() -> Self {
        Program { statements: vec![] }
    }

    pub fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_program(self);
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for AST construction
impl DataExpr {
    pub fn add(left: DataExpr, right: DataExpr) -> Self {
        DataExpr::Add(Box::new(left), Box::new(right))
    }

    pub fn negate(expr: DataExpr) -> Self {
        DataExpr::Negate(Box::new(expr))
    }

    pub fn number(n: Number) -> Self {
        DataExpr::Number(n)
    }

    pub fn identifier(name: impl Into<String>) -> Self {
        DataExpr::Identifier(name.into())
    }
}

impl Number {
    pub fn int(n: i64) -> Self {
        Number::Int(n)
    }

    pub fn float(n: f64) -> Self {
        Number::Float(n)
    }

    pub fn rational(num: i64, den: i64) -> Self {
        Number::Rational(num, den)
    }

    pub fn complex(real: f64, imag: f64) -> Self {
        Number::Complex(real, imag)
    }
}
