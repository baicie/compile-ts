use tsc_driver_impl::{parser::Parser, type_checker::TypeChecker, Lexer};

#[test]
fn test_type_checking() {
    let source = r#"
        function add(x: number, y: number): number {
            return x + y;
        }
    "#;

    let lexer = Lexer::new(source.to_string());
    let mut parser = Parser::new(lexer);
    let ast = parser.parse_program();

    let mut type_checker = TypeChecker::new();
    assert!(type_checker.check(&ast).is_ok());
}
