use super::token_type::TokenType;

#[derive(Debug, PartialEq, Clone)]
pub enum DataType {
  String,
  Int,
  Float,
  Boolean,
  Char,
  Null,
  None,
  Pending,
  Void,
  Variable(String),
  Array(Box<DataType>),
  Callable(Vec<DataType>, Box<DataType>),
  ClassType(String),
  // TODO: Type non-primitive
  GenericType {
    base: Box<DataType>,
    parameters: Vec<DataType>,
  },
  UnionType(Vec<DataType>),
  IntersectionType(Vec<DataType>),
  TupleType(Vec<DataType>),
  AliasType(String),
}

impl DataType {
  pub fn from_token_type(kind: TokenType) -> Self {
    match kind {
      TokenType::StringType => DataType::String,
      TokenType::FloatType => DataType::Float,
      TokenType::CharType => DataType::Char,
      TokenType::BooleanType => DataType::Boolean,
      TokenType::IntType => DataType::Int,
      TokenType::Void => DataType::Void,
      TokenType::Null => DataType::Null,
      _ => DataType::None,
    }
  }

  pub fn to_c_type(&self, is_mutable: bool) -> String {
    let mut kind: String = if !is_mutable {
      String::from("const ")
    } else {
      "".to_string()
    };

    match self {
      DataType::Int | DataType::Boolean => kind.push_str("int"),
      DataType::Float => kind.push_str("float"),
      DataType::Char => kind.push_str("char"),
      DataType::String => kind.push_str("char*"),
      DataType::Void | DataType::Null | DataType::None | DataType::Pending => kind.push_str("void"),
      DataType::Variable(name) => todo!(),
      DataType::ClassType(name) => todo!(),
      DataType::Array(array) => kind.push_str(format!("{}", array.to_c_type(true)).as_str()),
      DataType::Callable(_, _) => todo!(),
      DataType::GenericType { base, parameters } => todo!(),
      DataType::UnionType(_) => todo!(),
      DataType::IntersectionType(_) => todo!(),
      DataType::TupleType(_) => todo!(),
      DataType::AliasType(_) => todo!(),
    };

    kind
  }

  pub fn to_string(&self) -> String {
    match self {
      DataType::String => "String".to_string(),
      DataType::Int => "Int".to_string(),
      DataType::Float => "Float".to_string(),
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
        format!("Array<{}>", types.to_string())
      }
      DataType::IntersectionType(types) => {
        let type_strings: Vec<String> = types.iter().map(|t| t.to_string()).collect();
        format!("Intersection<{}>", type_strings.join(" & "))
      }
      DataType::TupleType(types) => {
        let type_strings: Vec<String> = types.iter().map(|t| t.to_string()).collect();
        format!("Tuple<{}>", type_strings.join(", "))
      }
      DataType::AliasType(alias) => alias.clone(),
      DataType::Null => "Null".to_string(),
      DataType::Void => "Void".to_string(),
      DataType::Callable(params, ret) => {
        let params: Vec<String> = params.iter().map(|p| p.to_string()).collect();
        format!("({}) -> {}", params.join(", "), ret.to_string())
      }
    }
  }
}
