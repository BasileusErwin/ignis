use crate::{
  ast::{
    lexer::{token::Token, text_span::TextSpan},
    expression::variable::VariableExpression,
  },
  evaluator::EvaluatorValue,
  enums::{data_type::DataType, token_type::TokenType},
};

use super::DiagnosticList;

#[derive(Debug)]
pub enum DiagnosticError {
  // Parser
  ExpectedExpression(Token),
  ExpectedToken(TokenType, Token),
  ExpectedVariableName(Token),
  ExpectedReturnTypeAfterFunction(Token),
  ExpectedAfterExpression(TokenType, Token, Token),
  ExpectedExpressionAfter(Token),
  UnexpectedToken(TokenType, Token),
  InvalidAssignmentTarget(TextSpan),
  ExpectedTypeAfterVariable(Token),

  // Evaluator | Analizer
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
}

impl DiagnosticError {
  pub fn report(&self, diagnostics: &mut DiagnosticList) {
    match self {
      DiagnosticError::ExpectedExpression(token) => {
        diagnostics.report_expected_expression(&token);
      }
      DiagnosticError::ExpectedToken(kind, token) => {
        diagnostics.report_expected_token(&kind, &token);
      }
      DiagnosticError::ExpectedVariableName(token) => {
        diagnostics.report_expected_variable_name(&token);
      }
      DiagnosticError::ExpectedReturnTypeAfterFunction(token) => {
        diagnostics.report_expected_return_type_after_function(&token);
      }
      DiagnosticError::ExpectedAfterExpression(kind, expression, token) => {
        diagnostics.report_expected_after_expression(&kind, &expression, &token);
      }
      DiagnosticError::ExpectedExpressionAfter(token) => {
        diagnostics.report_expected_expression(&token);
      }
      DiagnosticError::UnexpectedToken(kind, token) => {
        diagnostics.report_unexpected_token(&kind, &token);
      }
      DiagnosticError::InvalidAssignmentTarget(span) => {
        diagnostics.report_invalid_assignment_target(&span);
      }
      DiagnosticError::UndeclaredVariable(expression) => {
        diagnostics.report_undeclared_variable(&expression);
      }
      DiagnosticError::InvalidUnaryOperatorForDataType(operator, right) => {
        diagnostics.report_invalid_unary_operator_for_data_type(&operator, &right);
      }
      DiagnosticError::NotCallable(expression) => {
        diagnostics.report_not_callable(&expression);
      }
      DiagnosticError::InvalidNumberOfArguments(arity, arguments, token) => {
        diagnostics.report_invalid_number_of_arguments(&arity, &arguments, &token);
      }
      DiagnosticError::AssingInvalidType(argument, data_type, name) => {
        diagnostics.report_assing_invalid_type(&argument, &data_type, &name);
      }
      DiagnosticError::InvalidArgumentType(argument) => {
        diagnostics.report_invalid_argument_type(&argument);
      }
      DiagnosticError::MissingArgument(name, token) => {
        diagnostics.report_missing_argument(&name, &token.clone());
      }
      DiagnosticError::InvalidComparison(left, right, token) => {
        diagnostics.report_invalid_comparison(&left, &right, &token);
      }
      DiagnosticError::InvalidOperator(token) => {
        diagnostics.report_invalid_operator(&token);
      }
      DiagnosticError::InvalidUnaryOperator(token) => {
        diagnostics.report_invalid_unary_operator(&token);
      }
      DiagnosticError::UndefinedVariable(token) => {
        diagnostics.report_undefined_variable(&token);
      }
      DiagnosticError::VariableAlreadyDefined(name, data_type) => {
        diagnostics.report_variable_already_defined(name, data_type);
      }
      DiagnosticError::ExpectedTypeAfterVariable(token) => {
        diagnostics.report_expected_type_after_variable(&token);
      }
    }
  }
}
