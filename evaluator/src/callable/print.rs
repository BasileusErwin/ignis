use enums::data_type::DataType;

use crate::{
  Evaluator, evaluator_value::EvaluatorValue, EvaluatorResult,
  evaluator_error::EvaluatorDiagnosticError, execution_error::ExecutionError,
};

use super::Callable;

#[derive(Debug, PartialEq, Clone)]
pub struct Println {}

impl Println {
  pub fn new() -> Self {
    Self {}
  }
}

impl Callable for Println {
  fn arity(&self) -> usize {
    return 1;
  }

  fn call(
    &self,
    arguments: Vec<EvaluatorValue>,
    _evaluator: &mut Box<Evaluator>,
  ) -> EvaluatorResult<EvaluatorValue> {
    let mut value: String = String::new();

    for argument in arguments {
      match argument {
        EvaluatorValue::String(s) => value = s,
        EvaluatorValue::Int(i) => value = i.to_string(),
        EvaluatorValue::Double(d) => value = d.to_string(),
        EvaluatorValue::Boolean(b) => value = b.to_string(),
        EvaluatorValue::Return(r) => value = r.to_string(),
        EvaluatorValue::Null => value = "null".to_string(),
        EvaluatorValue::Callable(_) | EvaluatorValue::None => {
          return Err(ExecutionError::DiagnosticError(
            EvaluatorDiagnosticError::InvalidArgumentType(argument),
          ))
        }
      };
    }

    println!("{}", value);

    Ok(EvaluatorValue::None)
  }

  fn get_type(&self) -> Option<DataType> {
    Some(DataType::Void)
  }

  fn clone_box(&self) -> Box<dyn Callable> {
    Box::new(self.clone())
  }
}
