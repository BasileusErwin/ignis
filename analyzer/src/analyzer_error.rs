use ast::{expression::variable::VariableExpression, statement::import::ImportSymbol};
use enums::data_type::DataType;
use lexer::{token::Token, text_span::TextSpan};

use crate::analyzer_value::AnalyzerValue;

#[derive(Debug, Clone)]
pub enum AnalyzerDiagnosticError {
  UndeclaredVariable(VariableExpression),
  InvalidUnaryOperatorForDataType(Token, AnalyzerValue),
  NotCallable(Token),
  InvalidNumberOfArguments(usize, usize, Token),
  AssingInvalidType(DataType, DataType, Token),
  InvalidArgumentType(AnalyzerValue),
  MissingArgument(String, Token),
  InvalidComparison(AnalyzerValue, AnalyzerValue, Token),
  InvalidOperator(Token),
  InvalidUnaryOperator(Token),
  UndefinedVariable(Token),
  VariableAlreadyDefined(String, DataType),
  InvalidAssignmentTarget(TextSpan),
  InvalidReassignedVariable(TextSpan),
  TypeMismatch(DataType, DataType, Token),
  TypeMismatchUnary(DataType, Token),
  CannotSubtract(AnalyzerValue, AnalyzerValue, Token),
  CannotMultiply(AnalyzerValue, AnalyzerValue, Token),
  CannotDivide(AnalyzerValue, AnalyzerValue, Token),
  CannotModulo(AnalyzerValue, AnalyzerValue, Token),
  FunctionAlreadyDefined(String, Token),
  ClassAlreadyDefined(String),
  ArgumentTypeMismatch(DataType, DataType, Token),
  ImmutableVariableAsMutableParameter(String, String, Token),
  ReturnOutsideFunction(Token),
  NotIterable(Token),
  ArrayElementTypeMismatch(Token),
  ModuleNotFound(Token),
  ImportedFunctionIsNotExported(Token),
}
