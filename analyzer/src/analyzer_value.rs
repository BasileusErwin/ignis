use std::fmt::{Display, Formatter};

use ast::statement::function::FunctionStatement;
use enums::{data_type::DataType, literal_value::LiteralValue};

use crate::ir::instruction::{class_instance::IRClassInstance};

#[derive(Debug)]
pub enum AnalyzerValue {
  String(String),
  Int(i64),
  Float(f64),
  Boolean(bool),
  Return(Box<AnalyzerValue>),
  Function(Box<FunctionStatement>),
  Class(Box<IRClassInstance>),
  Null,
  None,
}

impl Clone for AnalyzerValue {
  fn clone(&self) -> Self {
    match self {
      AnalyzerValue::String(s) => AnalyzerValue::String(s.clone()),
      AnalyzerValue::Int(i) => AnalyzerValue::Int(*i),
      AnalyzerValue::Float(d) => AnalyzerValue::Float(*d),
      AnalyzerValue::Boolean(b) => AnalyzerValue::Boolean(*b),
      AnalyzerValue::Null => AnalyzerValue::Null,
      AnalyzerValue::None => AnalyzerValue::None,
      AnalyzerValue::Return(r) => r.as_ref().clone(),
      AnalyzerValue::Function(f) => AnalyzerValue::Function(f.clone()),
        AnalyzerValue::Class(_) => todo!(),
    }
  }
}

impl Display for AnalyzerValue {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      AnalyzerValue::String(s) => write!(f, "String: {}", s),
      AnalyzerValue::Int(i) => write!(f, "Int: {}", i),
      AnalyzerValue::Float(d) => write!(f, "Float: {}", d),
      AnalyzerValue::Boolean(b) => write!(f, "Boolean: {}", b),
      AnalyzerValue::Null => write!(f, "null"),
      AnalyzerValue::None => write!(f, "none"),
      AnalyzerValue::Return(r) => write!(f, "Return: {}", r),
      AnalyzerValue::Function(_) => write!(f, "function"),
      AnalyzerValue::Class(class) => {
        let class = class.as_ref();
        write!(f, "Class: {}", class.name)
      }
    }
  }
}

impl AnalyzerValue {
  pub fn to_data_type(&self) -> DataType {
    match self {
      AnalyzerValue::String(_) => DataType::String,
      AnalyzerValue::Int(_) => DataType::Int,
      AnalyzerValue::Float(_) => DataType::Float,
      AnalyzerValue::Boolean(_) => DataType::Boolean,
      AnalyzerValue::None | AnalyzerValue::Null => DataType::None,
      AnalyzerValue::Return(r) => r.to_data_type(),
      AnalyzerValue::Function(f) => {
        let value = f.return_type.as_ref();

        value.unwrap_or(&DataType::Void).clone()
      }
      AnalyzerValue::Class(class) => {
        let class = class.as_ref();
        DataType::ClassType(class.name.clone())
      }
    }
  }

  pub fn from_literation_value(value: LiteralValue) -> Self {
    match value {
      LiteralValue::Int(i) => AnalyzerValue::Int(i),
      LiteralValue::Float(d) => AnalyzerValue::Float(d),
      LiteralValue::String(s) => AnalyzerValue::String(s),
      LiteralValue::Boolean(b) => AnalyzerValue::Boolean(b),
      LiteralValue::Null => AnalyzerValue::Null,
      LiteralValue::Char(_) => todo!(),
    }
  }
}
