use ast::expression::variable::VariableExpression;
use enums::data_type::DataType;
use lexer::{token::Token, text_span::TextSpan};

use crate::evaluator_value::EvaluatorValue;

#[derive(Debug, Clone)]
pub enum EvaluatorDiagnosticError {
  UndeclaredVariable(VariableExpression),
  InvalidUnaryOperatorForDataType(Token, EvaluatorValue),
  NotCallable(Token),
  InvalidNumberOfArguments(usize, usize, Token),
  AssingInvalidType(DataType, DataType, Token),
  InvalidArgumentType(EvaluatorValue),
  MissingArgument(String, Token),
  InvalidComparison(EvaluatorValue, EvaluatorValue, Token),
  InvalidOperator(Token),
  InvalidUnaryOperator(Token),
  UndefinedVariable(Token),
  VariableAlreadyDefined(String, DataType),
  InvalidAssignmentTarget(TextSpan),
  InvalidReassignedVariable(TextSpan),
}
