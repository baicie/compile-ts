use tsc_driver_impl::lexer::{Lexer, TokenKind};

#[test]
fn test_basic_tokens() {
    let input = "let x = 42;";
    let mut lexer = Lexer::new(input.to_string());

    assert_eq!(lexer.next_token().kind, TokenKind::Let);
    assert_eq!(
        lexer.next_token().kind,
        TokenKind::Identifier("x".to_string())
    );
    assert_eq!(lexer.next_token().kind, TokenKind::Assign);
    assert_eq!(lexer.next_token().kind, TokenKind::Number(42.0));
    assert_eq!(lexer.next_token().kind, TokenKind::Semicolon);
    assert_eq!(lexer.next_token().kind, TokenKind::EOF);
}
