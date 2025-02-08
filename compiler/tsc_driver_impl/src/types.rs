#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    String,
    Boolean,
    Void,
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Unknown, // 用于类型推导
}

impl From<&str> for Type {
    fn from(s: &str) -> Self {
        match s {
            "number" => Type::Number,
            "void" => Type::Void,
            _ => Type::Unknown,
        }
    }
}
