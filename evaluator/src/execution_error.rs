use crate::{evaluator_error::EvaluatorDiagnosticError, evaluator_value::EvaluatorValue};

#[derive(Debug, Clone)]
pub enum ExecutionError {
  DiagnosticError(EvaluatorDiagnosticError),
  Return(EvaluatorValue),
  Break,
  Continue,
}
