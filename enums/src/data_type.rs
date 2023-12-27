use std::fmt::{Display, Formatter};

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
      DataType::Variable(_name) => todo!(),
      DataType::ClassType(_name) => todo!(),
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
}

impl Display for DataType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      DataType::String => write!(f, "String"),
      DataType::Int => write!(f, "Int"),
      DataType::Float => write!(f, "Float"),
      DataType::Boolean => write!(f, "Boolean"),
      DataType::Char => write!(f, "Char"),
      DataType::None => write!(f, "None"),
      DataType::Pending => write!(f, "Pending"),
      DataType::Variable(name) => write!(f, "{}", name),
      DataType::ClassType(name) => write!(f, "{}", name),
      DataType::GenericType { base, parameters } => {
        write!(
          f,
          "{}<{}>",
          base.to_string(),
          parameters
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(", ")
        )
      }
      DataType::UnionType(types) => {
        write!(
          f,
          "Union<{}>",
          types
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join(" | ")
        )
      }
      DataType::Array(types) => {
        write!(f, "Array<{}>", types.to_string())
      }
      DataType::IntersectionType(types) => {
        write!(
          f,
          "Intersection<{}>",
          types
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join(" & ")
        )
      }
      DataType::TupleType(types) => {
        write!(
          f,
          "Tuple<{}>",
          types
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join(", ")
        )
      }
      DataType::AliasType(alias) => write!(f, "{}", alias),
      DataType::Null => write!(f, "Null"),
      DataType::Void => write!(f, "Void"),
      DataType::Callable(params, ret) => write!(
        f,
        "({}) -> {}",
        params
          .iter()
          .map(|p| p.to_string())
          .collect::<Vec<String>>()
          .join(", "),
        ret.to_string()
      ),
    }
  }
}
