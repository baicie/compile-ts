//! 抽象语法树
//!
//! Nexa 语言的 AST 节点定义。

/// 源位置信息
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: (usize, usize), // (line, column)
    pub end: (usize, usize),
}

impl Span {
    pub fn merge(&self, other: &Span) -> Self {
        Self { start: self.start, end: other.end }
    }
}

/// 类型定义
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
    Void,
    Array(Box<Type>),
    Pointer(Box<Type>),
    Function(Vec<Type>, Box<Type>),
}

/// 表达式
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// 字面量
    Number(i64, Span),
    Float(f64, Span),
    String(String, Span),
    Boolean(bool, Span),
    Identifier(String, Span),

    /// 赋值表达式
    Assignment {
        target: Box<Expression>,
        value: Box<Expression>,
        span: Span,
    },

    /// 二元运算
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },

    /// 一元运算
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
        span: Span,
    },

    /// 函数调用
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
        span: Span,
    },

    /// 数组索引访问
    Index {
        array: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },

    /// 成员访问
    Member {
        object: Box<Expression>,
        member: String,
        span: Span,
    },
}

/// 二元运算符
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LogicalAnd,
    LogicalOr,
    BitAnd,
    BitOr,
    BitXor,
    LeftShift,
    RightShift,
}

/// 一元运算符
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,      // -x
    LogicalNot,  // !x
    BitNot,      // ~x
    Dereference, // *x
    AddressOf,   // &x
}

/// 语句
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// 变量声明
    VariableDeclaration {
        name: String,
        type_annotation: Option<Type>,
        initializer: Option<Expression>,
        mutable: bool,
        span: Span,
    },

    /// 赋值
    Assignment { target: Expression, value: Expression, span: Span },

    /// 表达式语句
    ExpressionStatement(Expression),

    /// if 语句
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
        span: Span,
    },

    /// while 循环
    While { condition: Expression, body: Box<Statement>, span: Span },

    /// for 循环
    For {
        initializer: Box<Statement>,
        condition: Option<Expression>,
        update: Option<Expression>,
        body: Box<Statement>,
        span: Span,
    },

    /// return 语句
    Return(Option<Expression>, Span),

    /// break 语句
    Break(Span),

    /// continue 语句
    Continue(Span),

    /// 块语句
    Block(Vec<Statement>, Span),

    /// 空语句
    Empty(Span),
}

/// 函数定义
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub body: Statement,
    pub span: Span,
}

/// 函数参数
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Type,
}

/// 程序
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
    pub statements: Vec<Statement>,
}
