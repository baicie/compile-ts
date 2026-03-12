//! 词法分析器
//!
//! 将源代码字符串转换为 Token 流。

use std::iter::Peekable;
use std::str::Chars;

/// 词法记号
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // 关键字
    Fn,
    Let,
    Const,
    If,
    Else,
    While,
    For,
    Return,
    Break,
    Continue,

    // 类型
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
    Void,

    // 字面量
    Number(i64),
    Float(f64),
    StringLiteral(String),
    Boolean(bool),
    Identifier(String),

    // 符号
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    SemiColon,
    Dot,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equals,
    PlusEquals,
    MinusEquals,
    StarEquals,
    SlashEquals,
    EqualsEquals,
    Bang,
    BangEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Ampersand,
    AmpersandAmpersand,
    Pipe,
    PipePipe,
    Caret,
    Tilde,
    Question,
    QuestionDot,
    Arrow,

    // 特殊
    Eof,
    Error(String),
}

/// 词法分析器
pub struct Lexer<'a> {
    source: Peekable<Chars<'a>>,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    /// 创建新的词法分析器
    pub fn new(source: &'a str) -> Self {
        Self { source: source.chars().peekable(), position: 0, line: 1, column: 1 }
    }

    /// 获取当前位置
    pub fn position(&self) -> (usize, usize) {
        (self.line, self.column)
    }

    /// 获取下一个字符
    fn next_char(&mut self) -> Option<char> {
        let ch = self.source.next();
        if let Some(c) = ch {
            self.position += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    /// 查看下一个字符但不消费
    fn peek_char(&mut self) -> Option<&char> {
        self.source.peek()
    }

    /// 跳过空白字符
    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    /// 跳过单行注释
    fn skip_comment(&mut self) {
        // 使用 peek 检查是否是注释，不提前消费字符
        let mut iter = self.source.clone();
        if let Some(&ch) = iter.peek() {
            if ch == '/' {
                iter.next(); // 暂时消费 /
                if let Some(&next) = iter.peek() {
                    if next == '/' {
                        // 单行注释 //，消费直到换行
                        while let Some(c) = self.next_char() {
                            if c == '\n' {
                                break;
                            }
                        }
                    } else if next == '*' {
                        // 块注释 /* */
                        self.next_char(); // 消费 *
                        let mut prev = '\0';
                        while let Some(c) = self.next_char() {
                            if prev == '*' && c == '/' {
                                break;
                            }
                            prev = c;
                        }
                    }
                }
            }
        }
    }

    /// 读取标识符或关键字
    fn read_identifier(&mut self, first: char) -> Token {
        let mut name = String::new();
        name.push(first);

        while let Some(&ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' {
                name.push(self.next_char().unwrap());
            } else {
                break;
            }
        }

        match name.as_str() {
            "fn" => Token::Fn,
            "let" => Token::Let,
            "const" => Token::Const,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "return" => Token::Return,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "i32" => Token::I32,
            "i64" => Token::I64,
            "f32" => Token::F32,
            "f64" => Token::F64,
            "bool" => Token::Bool,
            "string" => Token::String,
            "void" => Token::Void,
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            _ => Token::Identifier(name),
        }
    }

    /// 读取数字字面量
    fn read_number(&mut self, first: char) -> Token {
        let mut num_str = String::new();
        num_str.push(first);
        let mut is_float = false;

        while let Some(&ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                num_str.push(self.next_char().unwrap());
            } else if ch == '.' && !is_float {
                // 检查是否是小数点（如下一个字符是数字）
                is_float = true;
                num_str.push(self.next_char().unwrap());
                // 检查小数点后是否有数字
                if let Some(&next_ch) = self.peek_char() {
                    if !next_ch.is_ascii_digit() {
                        // 可能是范围运算符 ..
                        if next_ch == '.' {
                            break;
                        }
                    }
                }
            } else if ch == '_' {
                // 允许数字中的下划线
                self.next_char();
                num_str.push('_');
            } else {
                break;
            }
        }

        // 移除下划线
        let num_str: String = num_str.chars().filter(|&c| c != '_').collect();

        if is_float {
            match num_str.parse::<f64>() {
                Ok(n) => Token::Float(n),
                Err(_) => Token::Error(format!("Invalid float: {}", num_str)),
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(n) => Token::Number(n),
                Err(_) => Token::Error(format!("Invalid number: {}", num_str)),
            }
        }
    }

    /// 读取字符串字面量
    fn read_string(&mut self, quote: char) -> Token {
        let mut value = String::new();

        while let Some(ch) = self.next_char() {
            if ch == quote {
                break;
            } else if ch == '\\' {
                // 转义字符
                if let Some(escape) = self.next_char() {
                    match escape {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '\'' => value.push('\''),
                        '"' => value.push('"'),
                        '0' => value.push('\0'),
                        _ => value.push(escape),
                    }
                }
            } else {
                value.push(ch);
            }
        }

        Token::StringLiteral(value)
    }

    /// 获取下一个 Token
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        self.skip_comment();

        let ch = match self.next_char() {
            Some(c) => c,
            None => return Token::Eof,
        };

        // 标识符或关键字
        if ch.is_alphabetic() || ch == '_' {
            return self.read_identifier(ch);
        }

        // 数字
        if ch.is_ascii_digit() {
            return self.read_number(ch);
        }

        // 字符串
        if ch == '"' || ch == '\'' {
            return self.read_string(ch);
        }

        // 运算符和符号
        match ch {
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            ',' => Token::Comma,
            ':' => Token::Colon,
            ';' => Token::SemiColon,
            '.' => Token::Dot,
            '+' => self.match_or(Token::Plus, '=', Token::PlusEquals, Token::Plus),
            '-' => self.match_or(Token::Minus, '=', Token::MinusEquals, Token::Minus),
            '*' => self.match_or(Token::Star, '=', Token::StarEquals, Token::Star),
            '/' => self.match_or(Token::Slash, '=', Token::SlashEquals, Token::Slash),
            '%' => Token::Percent,
            '=' => self.match_or(Token::Equals, '=', Token::EqualsEquals, Token::Equals),
            '!' => self.match_or(Token::Bang, '=', Token::BangEquals, Token::Bang),
            '<' => self.match_or(Token::LessThan, '=', Token::LessThanOrEqual, Token::LessThan),
            '>' => self.match_or(
                Token::GreaterThan,
                '=',
                Token::GreaterThanOrEqual,
                Token::GreaterThan,
            ),
            '&' => {
                self.match_or(Token::Ampersand, '&', Token::AmpersandAmpersand, Token::Ampersand)
            },
            '|' => self.match_or(Token::Pipe, '|', Token::PipePipe, Token::Pipe),
            '^' => Token::Caret,
            '~' => Token::Tilde,
            '?' => Token::Question,
            _ => Token::Error(format!("Unexpected character: {}", ch)),
        }
    }

    /// 匹配双字符运算符
    fn match_or(&mut self, _single: Token, next: char, double: Token, default: Token) -> Token {
        if let Some(&ch) = self.peek_char() {
            if ch == next {
                self.next_char();
                double
            } else {
                default
            }
        } else {
            default
        }
    }
}

/// Token 迭代器
impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token == Token::Eof {
            None
        } else {
            Some(token)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("fn let if else while for return");
        assert_eq!(lexer.next_token(), Token::Fn);
        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::If);
        assert_eq!(lexer.next_token(), Token::Else);
        assert_eq!(lexer.next_token(), Token::While);
        assert_eq!(lexer.next_token(), Token::For);
        assert_eq!(lexer.next_token(), Token::Return);
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14 100");
        assert_eq!(lexer.next_token(), Token::Number(42));
        assert_eq!(lexer.next_token(), Token::Float(3.14));
        assert_eq!(lexer.next_token(), Token::Number(100));
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new("\"hello world\"");
        assert_eq!(lexer.next_token(), Token::StringLiteral("hello world".to_string()));
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / == != <= >=");
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Minus);
        assert_eq!(lexer.next_token(), Token::Star);
        assert_eq!(lexer.next_token(), Token::Slash);
        assert_eq!(lexer.next_token(), Token::EqualsEquals);
        assert_eq!(lexer.next_token(), Token::BangEquals);
        assert_eq!(lexer.next_token(), Token::LessThanOrEqual);
        assert_eq!(lexer.next_token(), Token::GreaterThanOrEqual);
    }
}
