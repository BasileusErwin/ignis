use enums::data_type::DataType;

use crate::{callable::Callable, environment::VariableEnvironment};

#[derive(Debug)]
pub enum EvaluatorValue {
  String(String),
  Int(i64),
  Double(f64),
  Boolean(bool),
  Callable(Box<dyn Callable>),
  Return(Box<EvaluatorValue>),
  Null,
  None,
}

impl EvaluatorValue {
  pub fn to_variable_environment(&self) -> VariableEnvironment {
    VariableEnvironment::new(self.clone(), true)
  }
}

impl Clone for EvaluatorValue {
  fn clone(&self) -> Self {
    match self {
      EvaluatorValue::String(s) => EvaluatorValue::String(s.clone()),
      EvaluatorValue::Int(i) => EvaluatorValue::Int(*i),
      EvaluatorValue::Double(d) => EvaluatorValue::Double(*d),
      EvaluatorValue::Boolean(b) => EvaluatorValue::Boolean(*b),
      EvaluatorValue::Null => EvaluatorValue::Null,
      EvaluatorValue::None => EvaluatorValue::None,
      EvaluatorValue::Callable(c) => EvaluatorValue::Callable(c.clone_box()),
      EvaluatorValue::Return(r) => r.as_ref().clone(),
    }
  }
}

impl EvaluatorValue {
  pub fn to_string(&self) -> String {
    match self {
      EvaluatorValue::String(_) => "string".to_string(),
      EvaluatorValue::Int(_) => "int".to_string(),
      EvaluatorValue::Double(_) => "double".to_string(),
      EvaluatorValue::Boolean(_) => "boolean".to_string(),
      EvaluatorValue::None | EvaluatorValue::Null => "null".to_string(),
      EvaluatorValue::Callable(_) => "callable".to_string(),
      EvaluatorValue::Return(_) => "return".to_string(),
    }
  }

  pub fn to_data_type(&self) -> DataType {
    match self {
      EvaluatorValue::String(_) => DataType::String,
      EvaluatorValue::Int(_) => DataType::Int,
      EvaluatorValue::Double(_) => DataType::Double,
      EvaluatorValue::Boolean(_) => DataType::Boolean,
      EvaluatorValue::None | EvaluatorValue::Null => DataType::None,
      EvaluatorValue::Callable(callee) => callee.get_type().unwrap(),
      EvaluatorValue::Return(r) => r.to_data_type(),
    }
  }
}
