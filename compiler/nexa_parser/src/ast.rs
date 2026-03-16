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

/// 类型定义 (TypeScript 风格)
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    Boolean,
    String,
    Void,
    Undefined,
    Null,
    Any,
    Never,
    Array(Box<Type>),
    Pointer(Box<Type>),
    Function(Vec<Type>, Box<Type>),
    Struct(String),
    Object(Vec<(String, Type)>),
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

    /// struct 字面量
    StructLiteral {
        name: String,
        fields: Vec<(String, Expression)>,
        span: Span,
    },

    /// 函数表达式（闭包）
    FunctionExpression {
        /// 参数列表
        parameters: Vec<Parameter>,
        /// 返回类型
        return_type: Box<Type>,
        /// 函数体
        body: Box<Statement>,
        /// 捕获的变量（闭包上下文）
        captures: Vec<Capture>,
        span: Span,
    },

    /// 数组字面量
    ArrayLiteral {
        elements: Vec<Expression>,
        span: Span,
    },
}

/// 闭包捕获的变量
#[derive(Debug, Clone, PartialEq)]
pub struct Capture {
    /// 捕获的变量名
    pub name: String,
    /// 捕获方式（按值或按引用）
    pub by_ref: bool,
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
    Concat,
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

    /// switch 表达式
    Switch {
        /// 要匹配的值
        value: Box<Expression>,
        /// 分支列表
        arms: Vec<SwitchArm>,
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

/// 结构体定义
#[derive(Debug, Clone, PartialEq)]
pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<StructField>,
    pub span: Span,
}

/// 结构体字段
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
}

/// switch 分支
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchArm {
    /// 匹配模式（数字或标识符）
    pub pattern: SwitchPattern,
    /// 对应的语句
    pub body: Box<Statement>,
    pub span: Span,
}

/// switch 模式
#[derive(Debug, Clone, PartialEq)]
pub enum SwitchPattern {
    /// 数字字面量
    Number(i64),
    /// 标识符（变量名或通配符）
    Identifier(String),
    /// 通配符（默认分支）
    Wildcard,
    /// default 分支
    Default,
}

/// 程序
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
    pub structs: Vec<StructDefinition>,
    pub interfaces: Vec<InterfaceDefinition>,
    pub statements: Vec<Statement>,
}

/// 接口定义 (TypeScript 风格)
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDefinition {
    pub name: String,
    pub fields: Vec<InterfaceField>,
    pub methods: Vec<InterfaceMethod>,
    pub span: Span,
}

/// 接口字段
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceField {
    pub name: String,
    pub field_type: Type,
}

/// 接口方法
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceMethod {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
}
