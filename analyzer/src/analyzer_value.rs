use ast::statement::function::FunctionStatement;
use enums::{data_type::DataType, literal_value::LiteralValue};

#[derive(Debug)]
pub enum AnalyzerValue {
  String(String),
  Int(i64),
  Double(f64),
  Boolean(bool),
  Return(Box<AnalyzerValue>),
  Function(Box<FunctionStatement>),
  Null,
  None,
}

impl Clone for AnalyzerValue {
  fn clone(&self) -> Self {
    match self {
      AnalyzerValue::String(s) => AnalyzerValue::String(s.clone()),
      AnalyzerValue::Int(i) => AnalyzerValue::Int(*i),
      AnalyzerValue::Double(d) => AnalyzerValue::Double(*d),
      AnalyzerValue::Boolean(b) => AnalyzerValue::Boolean(*b),
      AnalyzerValue::Null => AnalyzerValue::Null,
      AnalyzerValue::None => AnalyzerValue::None,
      AnalyzerValue::Return(r) => r.as_ref().clone(),
      AnalyzerValue::Function(f) => AnalyzerValue::Function(f.clone()),
    }
  }
}

impl AnalyzerValue {
  pub fn to_string(&self) -> String {
    match self {
      AnalyzerValue::String(_) => "string".to_string(),
      AnalyzerValue::Int(_) => "int".to_string(),
      AnalyzerValue::Double(_) => "double".to_string(),
      AnalyzerValue::Boolean(_) => "boolean".to_string(),
      AnalyzerValue::None | AnalyzerValue::Null => "null".to_string(),
      AnalyzerValue::Return(_) => "return".to_string(),
      AnalyzerValue::Function(_) => "function".to_string(),
    }
  }

  pub fn to_data_type(&self) -> DataType {
    match self {
      AnalyzerValue::String(_) => DataType::String,
      AnalyzerValue::Int(_) => DataType::Int,
      AnalyzerValue::Double(_) => DataType::Double,
      AnalyzerValue::Boolean(_) => DataType::Boolean,
      AnalyzerValue::None | AnalyzerValue::Null => DataType::None,
      AnalyzerValue::Return(r) => r.to_data_type(),
      AnalyzerValue::Function(f) => {
        let value = f.return_type.as_ref();

        value.unwrap_or(&DataType::Void).clone()
      }
    }
  }
  
  pub fn from_literation_value(value: LiteralValue) -> Self {
    match value {
        LiteralValue::Int(i) => AnalyzerValue::Int(i),
        LiteralValue::Double(d) => AnalyzerValue::Double(d),
        LiteralValue::String(s) => AnalyzerValue::String(s),
        LiteralValue::Boolean(b) => AnalyzerValue::Boolean(b),
        LiteralValue::Null => AnalyzerValue::Null,
        LiteralValue::Char(_) => todo!(),
    }
  }
}
