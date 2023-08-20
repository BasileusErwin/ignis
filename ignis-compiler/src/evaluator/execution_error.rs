use crate::diagnostic::{error::DiagnosticError, DiagnosticList};

use super::EvaluatorValue;

pub enum ExecutionError {
  DiagnosticError(DiagnosticError),
  Return(EvaluatorValue),
  Break,
  Continue,
}

impl ExecutionError {
	pub fn report(&self, diagnostics: &mut DiagnosticList) {
		match self {
			Self::DiagnosticError(error) => error.report(diagnostics),
			Self::Return(_) => {}
			Self::Break => {}
			Self::Continue => {}
		}
	}
}
