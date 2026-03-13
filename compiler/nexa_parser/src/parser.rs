//! 语法分析器
//!
//! 将 Token 流解析为 AST。

use crate::ast::*;
use crate::lexer::{Lexer, Token};

/// 解析错误
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

/// 解析器
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    position: (usize, usize),
}

impl<'a> Parser<'a> {
    /// 创建新的解析器
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let current = lexer.next_token();
        let position = lexer.position();
        Self { lexer, current, position }
    }

    /// 获取当前位置
    fn position(&self) -> (usize, usize) {
        self.position
    }

    /// 前进到下一个 Token
    fn advance(&mut self) {
        self.position = self.lexer.position();
        self.current = self.lexer.next_token();
    }

    /// 查看当前 Token
    fn peek(&self) -> &Token {
        &self.current
    }

    /// 检查当前 Token 是否为指定类型
    fn check(&self, expected: &Token) -> bool {
        match (&self.current, expected) {
            (Token::Number(_), Token::Number(_)) => true,
            (Token::Float(_), Token::Float(_)) => true,
            (Token::StringLiteral(_), Token::StringLiteral(_)) => true,
            (Token::Boolean(_), Token::Boolean(_)) => true,
            (Token::Identifier(_), Token::Identifier(_)) => true,
            (t, e) => std::mem::discriminant(t) == std::mem::discriminant(e),
        }
    }

    /// 创建 span
    fn span(&self, start: (usize, usize)) -> Span {
        Span { start, end: self.position }
    }

    /// 解析程序
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut functions = Vec::new();
        let mut statements = Vec::new();
        let mut structs = Vec::new();

        while *self.peek() != Token::Eof {
            match self.peek() {
                Token::Function => match self.parse_function() {
                    Ok(func) => functions.push(func),
                    Err(e) => return Err(e),
                },
                Token::Struct => match self.parse_struct_definition() {
                    Ok(s) => structs.push(s),
                    Err(e) => return Err(e),
                },
                _ => match self.parse_statement() {
                    Ok(stmt) => statements.push(stmt),
                    Err(e) => return Err(e),
                },
            }
        }

        Ok(Program { functions, structs, statements })
    }

    /// 解析结构体定义
    fn parse_struct_definition(&mut self) -> Result<StructDefinition, ParseError> {
        let start = self.position();
        self.expect_token(&Token::Struct)?;

        let name = match self.peek() {
            Token::Identifier(name) => name.clone(),
            _ => {
                return Err(ParseError {
                    message: "Expected struct name".to_string(),
                    span: self.span(start),
                })
            },
        };
        self.advance();

        self.expect_token(&Token::LeftBrace)?;

        let mut fields = Vec::new();
        while *self.peek() != Token::RightBrace {
            let field_name = match self.peek() {
                Token::Identifier(name) => name.clone(),
                _ => {
                    return Err(ParseError {
                        message: "Expected field name".to_string(),
                        span: self.span(self.position()),
                    })
                },
            };
            self.advance();

            self.expect_token(&Token::Colon)?;
            let field_type = self.parse_type()?;

            fields.push(StructField { name: field_name, field_type });

            // 处理逗号分隔
            if *self.peek() == Token::Comma {
                self.advance();
            }
        }

        self.expect_token(&Token::RightBrace)?;

        Ok(StructDefinition { name, fields, span: self.span(start) })
    }

    /// 解析函数
    fn parse_function(&mut self) -> Result<Function, ParseError> {
        let start = self.position();
        self.advance(); // 跳过 function

        let name = match self.peek() {
            Token::Identifier(name) => name.clone(),
            _ => {
                return Err(ParseError {
                    message: "Expected function name".to_string(),
                    span: self.span(start),
                })
            },
        };
        self.advance();

        // 参数列表
        self.expect_token(&Token::LeftParen)?;
        let mut parameters = Vec::new();
        while *self.peek() != Token::RightParen {
            let param_name = match self.peek() {
                Token::Identifier(name) => name.clone(),
                _ => {
                    return Err(ParseError {
                        message: "Expected parameter name".to_string(),
                        span: self.span(self.position()),
                    })
                },
            };
            self.advance();

            self.expect_token(&Token::Colon)?;
            let param_type = self.parse_type()?;

            parameters.push(Parameter { name: param_name, type_annotation: param_type });

            if *self.peek() == Token::Comma {
                self.advance();
            }
        }
        self.advance(); // 跳过 )

        // 返回类型 (TypeScript 风格: : number)
        let return_type = if *self.peek() == Token::Colon {
            self.advance();
            self.parse_type()?
        } else {
            Type::Void
        };

        // 函数体
        let body = self.parse_block()?;

        Ok(Function { name, parameters, return_type, body, span: self.span(start) })
    }

    /// 解析类型 (TypeScript 风格)
    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let start = self.position();
        let ty = match self.peek() {
            Token::NumberType => Type::Number,
            Token::BooleanType => Type::Boolean,
            Token::StringType => Type::String,
            Token::Void => Type::Void,
            Token::Undefined => Type::Undefined,
            Token::Null => Type::Null,
            Token::Any => Type::Any,
            Token::Identifier(_) => {
                // 自定义类型名（struct 名称）
                // 先获取名称，再 advance
                let type_name = match self.peek() {
                    Token::Identifier(name) => name.clone(),
                    _ => {
                        return Err(ParseError {
                            message: "Expected type name".to_string(),
                            span: self.span(start),
                        })
                    },
                };
                self.advance();
                return Ok(Type::Struct(type_name));
            },
            Token::LeftBracket => {
                // 数组类型 T[]
                self.advance();
                self.expect_token(&Token::RightBracket)?;
                let element_type = self.parse_type()?;
                Type::Array(Box::new(element_type))
            },
            Token::Star => {
                // 指针类型 *T
                self.advance();
                let pointee_type = self.parse_type()?;
                Type::Pointer(Box::new(pointee_type))
            },
            _ => {
                return Err(ParseError {
                    message: "Expected type".to_string(),
                    span: self.span(start),
                })
            },
        };
        self.advance();
        Ok(ty)
    }

    /// 解析语句
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek() {
            Token::Let | Token::Const => self.parse_variable_declaration(),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::For => self.parse_for_statement(),
            Token::Match => self.parse_match_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Break => {
                let start = self.position();
                self.advance();
                Ok(Statement::Break(self.span(start)))
            },
            Token::Continue => {
                let start = self.position();
                self.advance();
                Ok(Statement::Continue(self.span(start)))
            },
            Token::LeftBrace => self.parse_block_statement(),
            Token::SemiColon => {
                let start = self.position();
                self.advance();
                Ok(Statement::Empty(self.span(start)))
            },
            _ => self.parse_expression_statement(),
        }
    }

    /// 解析块语句
    fn parse_block(&mut self) -> Result<Statement, ParseError> {
        let start = self.position();
        self.expect_token(&Token::LeftBrace)?;

        let mut statements = Vec::new();
        while *self.peek() != Token::RightBrace && *self.peek() != Token::Eof {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => return Err(e),
            }
        }

        self.expect_token(&Token::RightBrace)?;
        Ok(Statement::Block(statements, self.span(start)))
    }

    /// 解析块语句语句
    fn parse_block_statement(&mut self) -> Result<Statement, ParseError> {
        self.parse_block()
    }

    /// 解析变量声明
    fn parse_variable_declaration(&mut self) -> Result<Statement, ParseError> {
        let start = self.position();
        let mutable = match self.peek() {
            Token::Let => {
                self.advance();
                false
            },
            Token::Const => {
                self.advance();
                true
            },
            _ => unreachable!(),
        };

        let name = match self.peek() {
            Token::Identifier(name) => name.clone(),
            _ => {
                return Err(ParseError {
                    message: "Expected variable name".to_string(),
                    span: self.span(start),
                })
            },
        };
        self.advance();

        // 类型注解
        let type_annotation = if *self.peek() == Token::Colon {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // 初始化值
        let initializer = if *self.peek() == Token::Equals {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect_token(&Token::SemiColon)?;

        Ok(Statement::VariableDeclaration {
            name,
            type_annotation,
            initializer,
            mutable,
            span: self.span(start),
        })
    }

    /// 解析 if 语句
    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.position();
        self.advance(); // 跳过 if

        self.expect_token(&Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect_token(&Token::RightParen)?;

        let then_branch = Box::new(self.parse_statement()?);

        let else_branch = if *self.peek() == Token::Else {
            self.advance();
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Statement::If { condition, then_branch, else_branch, span: self.span(start) })
    }

    /// 解析 while 语句
    fn parse_while_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.position();
        self.advance(); // 跳过 while

        self.expect_token(&Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect_token(&Token::RightParen)?;

        let body = Box::new(self.parse_statement()?);

        Ok(Statement::While { condition, body, span: self.span(start) })
    }

    /// 解析 for 语句
    fn parse_for_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.position();
        self.advance(); // 跳过 for

        self.expect_token(&Token::LeftParen)?;

        let initializer = Box::new(self.parse_statement()?);

        let condition =
            if *self.peek() != Token::SemiColon { Some(self.parse_expression()?) } else { None };
        self.expect_token(&Token::SemiColon)?;

        let update =
            if *self.peek() != Token::RightParen { Some(self.parse_expression()?) } else { None };

        self.expect_token(&Token::RightParen)?;

        let body = Box::new(self.parse_statement()?);

        Ok(Statement::For { initializer, condition, update, body, span: self.span(start) })
    }

    /// 解析 match 语句
    fn parse_match_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.position();
        self.advance(); // 跳过 match

        let value = Box::new(self.parse_expression()?);

        self.expect_token(&Token::LeftBrace)?;

        let mut arms = Vec::new();
        while *self.peek() != Token::RightBrace {
            arms.push(self.parse_match_arm()?);
        }

        self.expect_token(&Token::RightBrace)?;

        Ok(Statement::Match { value, arms, span: self.span(start) })
    }

    /// 解析 match 分支
    fn parse_match_arm(&mut self) -> Result<MatchArm, ParseError> {
        let start = self.position();

        // 解析模式
        let pattern = match self.peek() {
            Token::Number(n) => {
                let num = *n;
                self.advance();
                MatchPattern::Number(num)
            },
            Token::Identifier(name) => {
                let name_clone = name.clone();
                self.advance();
                // 检查是否是通配符 _
                if name_clone == "_" {
                    MatchPattern::Wildcard
                } else {
                    MatchPattern::Identifier(name_clone)
                }
            },
            _ => {
                return Err(ParseError {
                    message: format!("Expected pattern, got {:?}", self.peek()),
                    span: self.span(start),
                });
            },
        };

        // 跳过 =>
        self.expect_token(&Token::EqualsGreaterThan)?;

        // 解析分支体 - 允许没有分号的简单表达式
        // 如果是分号则跳过
        if *self.peek() == Token::SemiColon {
            self.advance();
        }

        // 解析表达式作为分支体
        let expr = self.parse_expression()?;

        // 如果后面有分号则跳过
        if *self.peek() == Token::SemiColon {
            self.advance();
        }

        let body = Box::new(Statement::ExpressionStatement(expr));

        Ok(MatchArm { pattern, body, span: self.span(start) })
    }

    /// 解析 return 语句
    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.position();
        self.advance(); // 跳过 return

        let value =
            if *self.peek() != Token::SemiColon { Some(self.parse_expression()?) } else { None };

        self.expect_token(&Token::SemiColon)?;

        Ok(Statement::Return(value, self.span(start)))
    }

    /// 解析表达式语句
    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_expression()?;
        self.expect_token(&Token::SemiColon)?;
        Ok(Statement::ExpressionStatement(expr))
    }

    /// 解析表达式
    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_assignment()
    }

    /// 解析赋值表达式
    fn parse_assignment(&mut self) -> Result<Expression, ParseError> {
        let start = self.position();
        let left = self.parse_logical_or()?;

        match self.peek() {
            Token::Equals => {
                self.advance();
                let right = self.parse_assignment()?;
                Ok(Expression::Assignment {
                    target: Box::new(left),
                    value: Box::new(right),
                    span: self.span(start),
                })
            },
            _ => Ok(left),
        }
    }

    /// 解析逻辑或
    fn parse_logical_or(&mut self) -> Result<Expression, ParseError> {
        let start = self.position();
        let mut left = self.parse_logical_and()?;

        while *self.peek() == Token::PipePipe {
            self.advance();
            let right = self.parse_logical_and()?;
            left = Expression::Binary {
                op: BinaryOp::LogicalOr,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span(start),
            };
        }

        Ok(left)
    }

    /// 解析逻辑与
    fn parse_logical_and(&mut self) -> Result<Expression, ParseError> {
        let start = self.position();
        let mut left = self.parse_bitwise_or()?;

        while *self.peek() == Token::AmpersandAmpersand {
            self.advance();
            let right = self.parse_bitwise_or()?;
            left = Expression::Binary {
                op: BinaryOp::LogicalAnd,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span(start),
            };
        }

        Ok(left)
    }

    /// 解析位或
    fn parse_bitwise_or(&mut self) -> Result<Expression, ParseError> {
        let start = self.position();
        let mut left = self.parse_bitwise_xor()?;

        while *self.peek() == Token::Pipe {
            self.advance();
            let right = self.parse_bitwise_xor()?;
            left = Expression::Binary {
                op: BinaryOp::BitOr,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span(start),
            };
        }

        Ok(left)
    }

    /// 解析位异或
    fn parse_bitwise_xor(&mut self) -> Result<Expression, ParseError> {
        let start = self.position();
        let mut left = self.parse_bitwise_and()?;

        while *self.peek() == Token::Caret {
            self.advance();
            let right = self.parse_bitwise_and()?;
            left = Expression::Binary {
                op: BinaryOp::BitXor,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span(start),
            };
        }

        Ok(left)
    }

    /// 解析位与
    fn parse_bitwise_and(&mut self) -> Result<Expression, ParseError> {
        let start = self.position();
        let mut left = self.parse_equality()?;

        while *self.peek() == Token::Ampersand {
            self.advance();
            let right = self.parse_equality()?;
            left = Expression::Binary {
                op: BinaryOp::BitAnd,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span(start),
            };
        }

        Ok(left)
    }

    /// 解析相等性比较
    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let start = self.position();
        let mut left = self.parse_comparison()?;

        while let Token::EqualsEquals | Token::BangEquals = self.peek().clone() {
            let op = match self.peek() {
                Token::EqualsEquals => BinaryOp::Equals,
                Token::BangEquals => BinaryOp::NotEquals,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span(start),
            };
        }

        Ok(left)
    }

    /// 解析比较运算
    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let start = self.position();
        let mut left = self.parse_shift()?;

        while let Token::LessThan
        | Token::LessThanOrEqual
        | Token::GreaterThan
        | Token::GreaterThanOrEqual = self.peek().clone()
        {
            let op = match self.peek() {
                Token::LessThan => BinaryOp::LessThan,
                Token::LessThanOrEqual => BinaryOp::LessThanOrEqual,
                Token::GreaterThan => BinaryOp::GreaterThan,
                Token::GreaterThanOrEqual => BinaryOp::GreaterThanOrEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_shift()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span(start),
            };
        }

        Ok(left)
    }

    /// 解析位移
    fn parse_shift(&mut self) -> Result<Expression, ParseError> {
        let _start = self.position();
        let left = self.parse_addition()?;

        // 暂时不支持位移运算符
        Ok(left)
    }

    /// 解析加减运算
    fn parse_addition(&mut self) -> Result<Expression, ParseError> {
        let start = self.position();
        let mut left = self.parse_multiplication()?;

        while let Token::Plus | Token::Minus | Token::PlusPlus = self.peek().clone() {
            let op = match self.peek() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Subtract,
                Token::PlusPlus => BinaryOp::Concat,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span(start),
            };
        }

        Ok(left)
    }

    /// 解析乘除运算
    fn parse_multiplication(&mut self) -> Result<Expression, ParseError> {
        let start = self.position();
        let mut left = self.parse_unary()?;

        while let Token::Star | Token::Slash | Token::Percent = self.peek().clone() {
            let op = match self.peek() {
                Token::Star => BinaryOp::Multiply,
                Token::Slash => BinaryOp::Divide,
                Token::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span(start),
            };
        }

        Ok(left)
    }

    /// 解析一元运算
    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        let start = self.position();

        match self.peek() {
            Token::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Negate,
                    operand: Box::new(operand),
                    span: self.span(start),
                })
            },
            Token::Bang => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::LogicalNot,
                    operand: Box::new(operand),
                    span: self.span(start),
                })
            },
            Token::Ampersand => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::AddressOf,
                    operand: Box::new(operand),
                    span: self.span(start),
                })
            },
            Token::Star => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Dereference,
                    operand: Box::new(operand),
                    span: self.span(start),
                })
            },
            _ => self.parse_call(),
        }
    }

    /// 解析函数调用
    fn parse_call(&mut self) -> Result<Expression, ParseError> {
        let start = self.position();
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek() {
                Token::LeftParen => {
                    self.advance();
                    let mut arguments = Vec::new();
                    while *self.peek() != Token::RightParen {
                        arguments.push(self.parse_expression()?);
                        if *self.peek() == Token::Comma {
                            self.advance();
                        }
                    }
                    self.advance();
                    expr = Expression::Call {
                        callee: Box::new(expr),
                        arguments,
                        span: self.span(start),
                    };
                },
                Token::LeftBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect_token(&Token::RightBracket)?;
                    expr = Expression::Index {
                        array: Box::new(expr),
                        index: Box::new(index),
                        span: self.span(start),
                    };
                },
                Token::Dot => {
                    self.advance();
                    let member = match self.peek() {
                        Token::Identifier(name) => name.clone(),
                        _ => {
                            return Err(ParseError {
                                message: "Expected member name".to_string(),
                                span: self.span(start),
                            })
                        },
                    };
                    self.advance();
                    expr = Expression::Member {
                        object: Box::new(expr),
                        member,
                        span: self.span(start),
                    };
                },
                Token::LeftBrace => {
                    // 可能是 struct 字面量 { field: value, ... }
                    // 需要回退并重新解析
                    if let Expression::Identifier(type_name, _) = &expr {
                        // 检查是否是 struct 字面量
                        self.advance(); // 跳过 {
                        let mut fields = Vec::new();
                        while *self.peek() != Token::RightBrace {
                            let field_name = match self.peek() {
                                Token::Identifier(name) => name.clone(),
                                _ => {
                                    return Err(ParseError {
                                        message: "Expected field name".to_string(),
                                        span: self.span(self.position()),
                                    })
                                },
                            };
                            self.advance();
                            self.expect_token(&Token::Colon)?;
                            let field_value = self.parse_expression()?;
                            fields.push((field_name, field_value));
                            if *self.peek() == Token::Comma {
                                self.advance();
                            }
                        }
                        self.expect_token(&Token::RightBrace)?;
                        expr = Expression::StructLiteral {
                            name: type_name.clone(),
                            fields,
                            span: self.span(start),
                        };
                    } else {
                        break;
                    }
                },
                _ => break,
            }
        }

        Ok(expr)
    }

    /// 解析基本表达式
    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let start = self.position();

        let token = self.peek().clone();
        self.advance();

        match token {
            Token::Number(n) => Ok(Expression::Number(n, self.span(start))),
            Token::Float(f) => Ok(Expression::Float(f, self.span(start))),
            Token::StringLiteral(s) => Ok(Expression::String(s, self.span(start))),
            Token::Boolean(b) => Ok(Expression::Boolean(b, self.span(start))),
            Token::Identifier(name) => Ok(Expression::Identifier(name, self.span(start))),
            Token::LeftParen => {
                let expr = self.parse_expression()?;
                self.expect_token(&Token::RightParen)?;
                Ok(expr)
            },
            _ => Err(ParseError {
                message: format!("Unexpected token: {:?}", token),
                span: self.span(start),
            }),
        }
    }

    /// 期望特定 Token
    fn expect_token(&mut self, expected: &Token) -> Result<(), ParseError> {
        if self.check(expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, got {:?}", expected, self.peek()),
                span: self.span(self.position()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function() {
        let source = "function add(a: number, b: number): number { return a + b; }";
        let mut parser = Parser::new(source);
        let result = parser.parse_program();
        assert!(result.is_ok(), "Error: {:?}", result.err());
    }

    #[test]
    fn test_parse_if() {
        let source = "if (x) { y; } else { z; }";
        let mut parser = Parser::new(source);
        let result = parser.parse_program();
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_expression() {
        let source = "1 + 2 * 3";
        let mut parser = Parser::new(source);
        let result = parser.parse_expression();
        assert!(result.is_ok());
    }
}
