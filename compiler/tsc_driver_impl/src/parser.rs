use crate::lexer::{Lexer, Token, TokenKind};

#[derive(Debug)]
pub enum AstNode {
    // 程序根节点
    Program(Vec<AstNode>),

    // 声明
    FunctionDecl {
        name: String,
        params: Vec<String>,
        body: Vec<AstNode>,
    },
    VariableDecl {
        name: String,
        initializer: Box<AstNode>,
    },
    VarDecl {
        name: String,
        init: Option<Box<AstNode>>,
        typ: Option<String>, // 可选的类型标注
    },

    // 语句
    ReturnStmt(Box<AstNode>),
    ExpressionStmt(Box<AstNode>),

    // 表达式
    BinaryExpr {
        left: Box<AstNode>,
        operator: String,
        right: Box<AstNode>,
    },
    Identifier(String),
    NumberLiteral(f64),
    CallExpr {
        callee: String,
        args: Vec<Box<AstNode>>,
    },
}

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        let mut parser = Parser {
            lexer,
            current_token: Token {
                kind: TokenKind::EOF,
                span: Default::default(),
            },
        };
        parser.advance(); // 获取第一个 token
        parser
    }

    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    // 解析程序
    pub fn parse_program(&mut self) -> AstNode {
        let mut statements = Vec::new();

        while self.current_token.kind != TokenKind::EOF {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
        }

        AstNode::Program(statements)
    }

    // 解析语句
    fn parse_statement(&mut self) -> Option<AstNode> {
        match self.current_token.kind {
            TokenKind::Function => Some(self.parse_function_declaration()),
            TokenKind::Let => Some(self.parse_variable_declaration()),
            TokenKind::Return => Some(self.parse_return_statement()),
            _ => Some(self.parse_expression_statement()),
        }
    }

    // 解析函数声明
    fn parse_function_declaration(&mut self) -> AstNode {
        self.advance(); // 跳过 'function' 关键字

        let name = if let TokenKind::Identifier(name) = self.current_token.kind.clone() {
            name
        } else {
            panic!("Expected function name");
        };
        self.advance();

        self.expect_token(TokenKind::LParen);
        let params = self.parse_parameters();
        self.expect_token(TokenKind::RParen);

        // 解析返回类型
        if self.current_token.kind == TokenKind::Colon {
            self.advance();
            self.expect_token(TokenKind::Identifier("number".to_string())); // 简化版
        }

        self.expect_token(TokenKind::LBrace);
        let body = self.parse_block();
        self.expect_token(TokenKind::RBrace);

        AstNode::FunctionDecl { name, params, body }
    }

    // 解析表达式
    fn parse_expression(&mut self) -> AstNode {
        self.parse_binary_expression()
    }

    // 解析二元表达式
    fn parse_binary_expression(&mut self) -> AstNode {
        let left = self.parse_primary();

        match self.current_token.kind {
            TokenKind::Plus | TokenKind::Minus => {
                let operator = format!("{:?}", self.current_token.kind);
                self.advance();

                let right = self.parse_expression();
                AstNode::BinaryExpr {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                }
            }
            _ => left,
        }
    }

    // 解析变量声明
    fn parse_variable_declaration(&mut self) -> AstNode {
        self.advance(); // 跳过 'let' 关键字

        let name = if let TokenKind::Identifier(name) = self.current_token.kind.clone() {
            name
        } else {
            panic!("Expected variable name");
        };
        self.advance();

        self.expect_token(TokenKind::Assign);
        let initializer = Box::new(self.parse_expression());
        self.expect_token(TokenKind::Semicolon);

        AstNode::VariableDecl { name, initializer }
    }

    fn parse_return_statement(&mut self) -> AstNode {
        self.advance(); // 跳过 'return' 关键字
        let value = Box::new(self.parse_expression());
        self.expect_token(TokenKind::Semicolon);
        AstNode::ReturnStmt(value)
    }

    fn parse_expression_statement(&mut self) -> AstNode {
        let expr = self.parse_expression();
        // 将表达式包装为表达式语句
        let stmt = AstNode::ExpressionStmt(Box::new(expr));
        self.expect_token(TokenKind::Semicolon);
        stmt
    }

    // 辅助方法
    fn expect_token(&mut self, kind: TokenKind) {
        if self.current_token.kind == kind {
            self.advance();
        } else {
            panic!(
                "Unexpected token: expected {:?}, got {:?}",
                kind, self.current_token.kind
            );
        }
    }

    fn parse_parameters(&mut self) -> Vec<String> {
        let mut params = Vec::new();

        while self.current_token.kind != TokenKind::RParen {
            if let TokenKind::Identifier(name) = self.current_token.kind.clone() {
                params.push(name);
                self.advance();

                // 处理参数类型注解
                if self.current_token.kind == TokenKind::Colon {
                    self.advance();
                    self.expect_token(TokenKind::Identifier("number".to_string()));
                    // 简化版
                }

                if self.current_token.kind == TokenKind::RParen {
                    break;
                }
                self.expect_token(TokenKind::Comma);
            }
        }

        params
    }

    fn parse_block(&mut self) -> Vec<AstNode> {
        let mut statements = Vec::new();

        while self.current_token.kind != TokenKind::RBrace {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
        }

        statements
    }

    fn parse_primary(&mut self) -> AstNode {
        match &self.current_token.kind {
            TokenKind::Number(n) => {
                let node = AstNode::NumberLiteral(*n);
                self.advance();
                node
            }
            TokenKind::Identifier(name) => {
                let mut name_clone = name.clone();
                self.advance();

                // 处理成员访问，如 console.log
                if self.current_token.kind == TokenKind::Dot {
                    self.advance(); // 跳过 '.'
                    if let TokenKind::Identifier(member) = &self.current_token.kind {
                        name_clone = format!("{}.{}", name_clone, member);
                        self.advance();
                    }
                }

                // 检查是否是函数调用
                if self.current_token.kind == TokenKind::LParen {
                    self.advance(); // 跳过 '('
                    let mut args = Vec::new();

                    // 解析参数
                    if self.current_token.kind != TokenKind::RParen {
                        args.push(Box::new(self.parse_expression()));
                    }

                    self.expect_token(TokenKind::RParen);
                    AstNode::CallExpr {
                        callee: name_clone,
                        args,
                    }
                } else {
                    AstNode::Identifier(name_clone)
                }
            }
            _ => panic!("Unexpected token in primary expression"),
        }
    }
}
