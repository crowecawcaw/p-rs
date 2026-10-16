//! Abstract syntax tree of a P program. The structure mirrors `PParser.g4` upstream.

use crate::span::Span;

/// An identifier with its source span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

/// A type expression as written in source.
#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Seq(Box<Type>),
    Set(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    NamedTuple(Vec<(Ident, Type)>),
    Bool,
    Int,
    Float,
    String,
    Event,
    Machine,
    Data,
    Any,
    Named(Ident),
}

/// A whole parsed file.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub decls: Vec<TopDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TopDecl {
    /// `type T;` (foreign, `ty == None`) or `type T = ...;`
    TypeDef { name: Ident, ty: Option<Type>, span: Span },
    Enum { name: Ident, elems: Vec<EnumElem>, span: Span },
    Event { name: Ident, payload: Option<Type>, span: Span },
    EventSet { name: Ident, events: Vec<Ident>, span: Span },
    /// `receives == None` means the universal event set.
    Interface { name: Ident, payload: Option<Type>, receives: Option<Vec<Ident>>, span: Span },
    Machine(MachineDecl),
    Fun(FunDecl),
    Pure(PureDecl),
    Invariant(InvariantDecl),
    InvariantGroup { name: Ident, is_theorem: bool, invariants: Vec<InvariantDecl>, span: Span },
    Axiom { body: Expr, span: Span },
    AssumeOnStart { body: Expr, span: Span },
    ProofBlock { name: Option<Ident>, items: Vec<ProofItem>, span: Span },
    Module { name: Ident, expr: ModExpr, span: Span },
    SafetyTest(SafetyTestDecl),
    RefinementTest { name: Ident, main: Ident, lhs: ModExpr, rhs: ModExpr, span: Span },
    Implementation { name: Ident, main: Option<Ident>, expr: ModExpr, span: Span },
    GlobalParam { names: Vec<Ident>, ty: Type, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumElem {
    pub name: Ident,
    /// Explicit numeric value (`enum E { A = 1 }`); its source text so range errors can be reported.
    pub value: Option<(String, Span)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineKind {
    Impl,
    Spec,
    Scenario,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MachineDecl {
    pub kind: MachineKind,
    pub name: Ident,
    /// Each `receives ...;` annotation (impl machines).
    pub receives: Vec<(Vec<Ident>, Span)>,
    /// Each `sends ...;` annotation (impl machines).
    pub sends: Vec<(Vec<Ident>, Span)>,
    /// `observes` list (spec/scenario machines).
    pub observes: Vec<Ident>,
    pub vars: Vec<VarDecl>,
    pub funs: Vec<FunDecl>,
    pub states: Vec<StateDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl {
    pub names: Vec<Ident>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunDecl {
    pub name: Ident,
    pub params: Vec<Param>,
    pub ret: Option<Type>,
    /// `None` for foreign functions.
    pub body: Option<FunBody>,
    /// Foreign function `creates` annotations.
    pub creates: Vec<Ident>,
    /// Foreign function `return (p: T)` parameter (verifier syntax).
    pub return_param: Option<Param>,
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunBody {
    pub vars: Vec<VarDecl>,
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PureDecl {
    pub name: Ident,
    pub params: Vec<Param>,
    pub ret: Type,
    pub body: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InvariantDecl {
    pub name: Ident,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProofItem {
    pub targets: Vec<Expr>,
    pub targets_all: bool,
    pub targets_default: bool,
    pub premises: Vec<Expr>,
    pub premises_all: bool,
    pub excludes: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Temperature {
    Warm,
    Hot,
    Cold,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateDecl {
    pub name: Ident,
    pub is_start: bool,
    pub temperature: Temperature,
    pub items: Vec<StateItem>,
    pub span: Span,
}

/// Either a reference to a named function or an inline anonymous handler.
#[derive(Debug, Clone, PartialEq)]
pub enum Handler {
    Named(Ident),
    Anon(AnonHandler),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnonHandler {
    pub param: Option<Param>,
    pub body: FunBody,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateItemKind {
    Entry(Handler),
    Exit(Handler),
    Defer(Vec<Ident>),
    Ignore(Vec<Ident>),
    OnDo { events: Vec<Ident>, handler: Handler },
    OnGoto { events: Vec<Ident>, target: Ident, with: Option<Handler> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateItem {
    pub kind: StateItemKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Compound(Vec<Stmt>),
    Assert { cond: Expr, message: Option<Expr> },
    Assume { cond: Expr, message: Option<Expr> },
    Print(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
    Assign { lhs: Expr, rhs: Expr },
    /// `lhs += (index, value);`
    Insert { lhs: Expr, index: Expr, value: Expr },
    /// `lhs += (value);`
    Add { lhs: Expr, value: Expr },
    /// `lhs -= value;`
    Remove { lhs: Expr, value: Expr },
    While { cond: Expr, body: Box<Stmt> },
    Foreach { item: Ident, collection: Expr, invariants: Vec<Expr>, body: Box<Stmt> },
    If { cond: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> },
    /// `new I(args);`
    Ctor { interface: Ident, args: Vec<Expr> },
    FunCall { fun: Ident, args: Vec<Expr> },
    Raise { event: Expr, args: Vec<Expr> },
    Send { target: Expr, event: Expr, args: Vec<Expr> },
    Announce { event: Expr, args: Vec<Expr> },
    Goto { state: Ident, args: Vec<Expr> },
    Receive { cases: Vec<RecvCase> },
    NoStmt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecvCase {
    pub events: Vec<Ident>,
    pub handler: AnonHandler,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    Lt,
    Gt,
    Ge,
    Le,
    In,
    Eq,
    Ne,
    And,
    Or,
    Then,
    Iff,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::Le => "<=",
            BinOp::In => "in",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::And => "&&",
            BinOp::Or => "||",
            BinOp::Then => "==>",
            BinOp::Iff => "<==>",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CastKind {
    /// `as`
    As,
    /// `to`
    To,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quantifier {
    Forall,
    Exists,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Name(Ident),
    /// Float literal; the text is kept so it can be parsed with invariant culture semantics.
    FloatLit(String),
    BoolLit(bool),
    /// Integer literal text (may overflow `int`, reported by the checker).
    IntLit(String),
    NullLit,
    Nondet,
    FairNondet,
    Halt,
    This,
    UnnamedTuple(Vec<Expr>),
    NamedTuple(Vec<(Ident, Expr)>),
    NamedTupleAccess { base: Box<Expr>, field: Ident },
    TupleAccess { base: Box<Expr>, index: String, index_span: Span },
    Test { instance: Box<Expr>, kind: Ident },
    Targets { instance: Box<Expr>, target: Box<Expr> },
    Flying(Box<Expr>),
    Sent(Box<Expr>),
    Index { base: Box<Expr>, index: Box<Expr> },
    Keys(Box<Expr>),
    Values(Box<Expr>),
    Sizeof(Box<Expr>),
    Default(Type),
    Ctor { interface: Ident, args: Vec<Expr> },
    FunCall { fun: Ident, args: Vec<Expr> },
    Unary { op: UnOp, operand: Box<Expr> },
    Binary { op: BinOp, op_span: Span, lhs: Box<Expr>, rhs: Box<Expr> },
    Cast { kind: CastKind, expr: Box<Expr>, ty: Type },
    Quant { quant: Quantifier, diff: bool, bound: Vec<Param>, body: Box<Expr> },
    Choose(Option<Box<Expr>>),
    /// String literal (raw text including quotes) with optional `format(...)` arguments.
    Str { raw: String, raw_span: Span, args: Vec<Expr>, is_format: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModExpr {
    pub kind: ModExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModExprKind {
    Paren(Box<ModExpr>),
    /// `{ M1, M2 -> I2 }`
    Bind(Vec<BindExpr>),
    Named(Ident),
    Compose(Vec<ModExpr>),
    Union(Vec<ModExpr>),
    HideEvents { events: Vec<Ident>, module: Box<ModExpr> },
    HideInterfaces { interfaces: Vec<Ident>, module: Box<ModExpr> },
    Assert { monitors: Vec<Ident>, module: Box<ModExpr> },
    Rename { old: Ident, new: Ident, module: Box<ModExpr> },
    Main { main: Ident, module: Box<ModExpr> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindExpr {
    pub machine: Ident,
    pub interface: Option<Ident>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SeqPrimitive {
    Bool(bool, Span),
    /// Integer literal text with optional leading minus.
    Int { negative: bool, text: String, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamBinding {
    pub name: Ident,
    pub values: Vec<SeqPrimitive>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Twise {
    Pairwise(Span),
    N(String, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SafetyTestDecl {
    pub name: Ident,
    pub main: Ident,
    pub expr: ModExpr,
    pub params: Option<(Vec<ParamBinding>, Span)>,
    pub assume: Option<Expr>,
    pub twise: Option<Twise>,
    pub span: Span,
}
