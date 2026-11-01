//! Abstract Syntax Tree (AST) definitions for the Apsis programming language.

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveType {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Char,
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Custom(String),
    Generic(String, Vec<Type>),
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    RawPointer { is_mut: bool, target: Box<Type> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub is_mut: bool,
    pub param_type: Type,
    pub default_value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub is_pub: bool,
    pub is_async: bool,
    pub generics: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Option<Block>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        is_mut: bool,
        declared_type: Option<Type>,
        init_expr: Expr,
    },
    Const {
        name: String,
        declared_type: Option<Type>,
        value: Expr,
    },
    Assign {
        target: Expr,
        op: Option<BinaryOperator>, // None for =, Some(Add) for +=
        value: Expr,
    },
    Defer(Expr),
    Expr(Expr),
    Return(Option<Expr>),
    Break {
        label: Option<String>,
        value: Option<Box<Expr>>,
    },
    Continue(Option<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLiteral(i128),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    CharLiteral(char),
    Identifier(String),
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Pipe {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<CallArg>,
    },
    MethodCall {
        receiver: Box<Expr>,
        method_name: String,
        args: Vec<CallArg>,
    },
    Block(Block),
    Loop {
        label: Option<String>,
        body: Block,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallArg {
    pub name: Option<String>,
    pub is_mut: bool,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv, // //
    Mod,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<FunctionDecl>,
}
