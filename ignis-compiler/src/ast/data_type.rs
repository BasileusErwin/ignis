use super::lexer::token_type::TokenType;

#[derive(Debug, PartialEq, Clone)]
pub enum DataType {
  String,
  Int,
  Double,
  Boolean,
  Char,
  None,
  Pending,
  // TODO: Type non-primitive
  Variable(String),
  Array(Vec<DataType>),
  ClassType(String),
  GenericType {
    base: Box<DataType>,
    parameters: Vec<DataType>,
  },
  UnionType(Vec<DataType>),
  IntersectionType(Vec<DataType>),
  TupleType(Vec<DataType>),
  LiteralType(String),
  AliasType(String),
}

impl DataType {
  pub fn from_token_type(kind: TokenType) -> Self {
    match kind {
      TokenType::StringType => DataType::String,
      TokenType::DoubleType => DataType::Double,
      TokenType::CharType => DataType::Char,
      TokenType::BooleanType => DataType::Boolean,
      TokenType::IntType => DataType::Int,
      _ => DataType::None,
    }
  }

  pub fn to_string(&self) -> String {
    match self {
      DataType::String => "String".to_string(),
      DataType::Int => "Int".to_string(),
      DataType::Double => "Double".to_string(),
      DataType::Boolean => "Boolean".to_string(),
      DataType::Char => "Char".to_string(),
      DataType::None => "Null".to_string(),
      DataType::Pending => "Pending".to_string(),
      DataType::Variable(name) => name.to_string(),
      DataType::ClassType(name) => name.clone(),
      DataType::GenericType { base, parameters } => {
        let params: Vec<String> = parameters.iter().map(|p| p.to_string()).collect();
        format!("{}<{}>", base.to_string(), params.join(", "))
      }
      DataType::UnionType(types) => {
        let type_strings: Vec<String> = types.iter().map(|t| t.to_string()).collect();
        format!("Union<{}>", type_strings.join(" | "))
      }
      DataType::Array(types) => {
        let type_strings: Vec<String> = types.iter().map(|t| t.to_string()).collect();
        format!("Array[{}]", type_strings.join(", "))
      }
      DataType::IntersectionType(types) => {
        let type_strings: Vec<String> = types.iter().map(|t| t.to_string()).collect();
        format!("Intersection<{}>", type_strings.join(" & "))
      }
      DataType::TupleType(types) => {
        let type_strings: Vec<String> = types.iter().map(|t| t.to_string()).collect();
        format!("Tuple<{}>", type_strings.join(", "))
      }
      DataType::LiteralType(literal) => literal.clone(),
      DataType::AliasType(alias) => alias.clone(),
    }
  }
}
