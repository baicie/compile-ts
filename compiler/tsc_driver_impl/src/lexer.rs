#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    // 字面量
    Number(f64),
    String(String),
    Identifier(String),

    // 关键字
    Let,
    Const,
    Function,
    If,
    Return,

    // 运算符
    Plus,   // +
    Minus,  // -
    Assign, // =

    // 分隔符
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    Semicolon, // ;
    Comma,     // ,
    Colon,     // :
    Dot,       // .

    // 其他
    EOF,     // 文件结束
    Invalid, // 无效字符
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return self.create_token(TokenKind::EOF);
        }

        let ch = self.current_char();
        match ch {
            '0'..='9' => self.read_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(),
            '"' => self.read_string(),
            '+' => self.single_char_token(TokenKind::Plus),
            '-' => self.single_char_token(TokenKind::Minus),
            '=' => self.single_char_token(TokenKind::Assign),
            '(' => self.single_char_token(TokenKind::LParen),
            ')' => self.single_char_token(TokenKind::RParen),
            '{' => self.single_char_token(TokenKind::LBrace),
            '}' => self.single_char_token(TokenKind::RBrace),
            ';' => self.single_char_token(TokenKind::Semicolon),
            ':' => self.single_char_token(TokenKind::Colon),
            ',' => self.single_char_token(TokenKind::Comma),
            '.' => {
                self.advance();
                self.create_token(TokenKind::Dot)
            }
            _ => self.single_char_token(TokenKind::Invalid),
        }
    }

    fn read_number(&mut self) -> Token {
        let start_pos = self.position;
        while self.position < self.input.len() && self.current_char().is_digit(10) {
            self.advance();
        }

        let number_str: String = self.input[start_pos..self.position].iter().collect();
        let number = number_str.parse::<f64>().unwrap();

        self.create_token(TokenKind::Number(number))
    }

    fn read_identifier(&mut self) -> Token {
        let start_pos = self.position;
        while self.position < self.input.len()
            && (self.current_char().is_alphanumeric() || self.current_char() == '_')
        {
            self.advance();
        }

        let ident: String = self.input[start_pos..self.position].iter().collect();

        // 检查是否是关键字
        let kind = match ident.as_str() {
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            "function" => TokenKind::Function,
            "if" => TokenKind::If,
            "return" => TokenKind::Return,
            _ => TokenKind::Identifier(ident),
        };

        self.create_token(kind)
    }

    fn read_string(&mut self) -> Token {
        self.advance(); // 跳过开始的引号
        let start_pos = self.position;

        while self.position < self.input.len() && self.current_char() != '"' {
            self.advance();
        }

        let string = self.input[start_pos..self.position].iter().collect();
        self.advance(); // 跳过结束的引号

        self.create_token(TokenKind::String(string))
    }

    // 辅助方法
    fn current_char(&self) -> char {
        self.input[self.position]
    }

    fn advance(&mut self) {
        self.position += 1;
        self.column += 1;
    }

    fn create_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            span: Span {
                start: self.position,
                end: self.position + 1,
                line: self.line,
                column: self.column,
            },
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.current_char().is_whitespace() {
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            self.advance();
        }
    }

    fn single_char_token(&mut self, kind: TokenKind) -> Token {
        let token = self.create_token(kind);
        self.advance();
        token
    }
}
