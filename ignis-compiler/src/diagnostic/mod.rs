pub mod error;

use std::fmt::Display;

use crate::{ast::{
  lexer::{text_span::TextSpan, token::Token},
  expression::variable::VariableExpression,
}, enums::{data_type::DataType, token_type::TokenType}, evaluator::EvaluatorValue};

#[derive(Debug)]
pub enum DiagnosticLevel {
  Warning,
  Error,
}

impl Display for DiagnosticLevel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DiagnosticLevel::Warning => write!(f, "Warning"),
      DiagnosticLevel::Error => write!(f, "Error"),
    }
  }
}

#[derive(Debug)]
pub struct Diagnostic {
  pub code: DiagnosticLevel,
  pub span: Box<TextSpan>,
  pub hint: Option<String>,
}

impl Diagnostic {
  pub fn new(code: DiagnosticLevel, span: Box<TextSpan>, hint: Option<String>) -> Self {
    Self { code, span, hint }
  }
}

#[derive(Debug)]
pub struct DiagnosticList {
  pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticList {
  pub fn new() -> Self {
    Self {
      diagnostics: Vec::new(),
    }
  }

  pub fn clean_diagnostic(&mut self) {
    self.diagnostics = Vec::new();
  }

  pub fn report_error(&mut self, message: String, span: TextSpan) {
    self.diagnostics.push(Diagnostic::new(
      DiagnosticLevel::Error,
      Box::new(span),
      Some(message),
    ));
  }

  pub fn report_warning(&mut self, message: String, span: TextSpan) {
    self.diagnostics.push(Diagnostic::new(
      DiagnosticLevel::Warning,
      Box::new(span),
      Some(message),
    ));
  }

  pub fn report_unexpected_token(&mut self, expected: &TokenType, token: &Token) {
    self.report_error(
      format!("Expected '{}', found '{}'", expected, token.kind),
      token.span.clone(),
    );
  }

  pub fn report_unexpected_token_multiple(&mut self, expecteds: &[TokenType], token: &Token) {
    let expected: String = expecteds
      .iter()
      .map(|t| format!("'{}'", t))
      .collect::<Vec<_>>()
      .join(", ");

    self.report_error(
      format!("Expected {}, found '{}'", expected, token.kind),
      token.span.clone(),
    );
  }

  pub fn report_expected_expression(&mut self, token: &Token) {
    self.report_error(
      format!("Expected expression, found '{}'", token.kind),
      token.span.clone(),
    );
  }

  pub fn report_undeclared_variable(&mut self, expression: &VariableExpression) {
    self.report_error(
      format!("Undeclared variable '{}'", expression.name.span.literal),
      expression.name.span.clone(),
    );
  }

  pub fn report_assing_invalid_type(
    &mut self,
    value_type: &DataType,
    expression_type: &DataType,
    token: &Token,
  ) {
    self.report_error(
      format!(
        "Cannot assign {} to {}",
        value_type.to_string(),
        expression_type.to_string()
      ),
      token.span.clone(),
    );
  }

  pub fn report_expected_variable_name(&mut self, token: &Token) {
    self.report_error(
      format!("Expected variable name, found '{}'", token.kind),
      token.span.clone(),
    );
  }

  pub fn report_expected_type_after_variable(&mut self, token: &Token) {
    self.report_error(
      format!("Expected type after variable name, found '{}'", token.kind),
      token.span.clone(),
    );
  }

  pub fn report_invalid_assignment_target(&mut self, span: &TextSpan) {
    self.report_error(format!("Invalid assignment target"), span.clone());
  }

  pub fn report_invalid_operator(&mut self, operator: &Token) {
    self.report_error(
      format!("Invalid operator '{}'", operator.span.literal),
      operator.span.clone(),
    );
  }

  pub fn report_invalid_operator_for_data_type(
    &mut self,
    operator: &Token,
    left: &EvaluatorValue,
    right: &EvaluatorValue,
  ) {
    self.report_error(
      format!(
        "Invalid operator '{}' for data types '{}' and '{}'",
        operator.span.literal,
        left.to_string(),
        right.to_string()
      ),
      operator.span.clone(),
    );
  }

  pub fn report_invalid_unary_operator_for_data_type(
    &mut self,
    operator: &Token,
    right: &EvaluatorValue,
  ) {
    self.report_error(
      format!(
        "Invalid operator unary '{}' for data types '{}'",
        operator.span.literal,
        right.to_string()
      ),
      operator.span.clone(),
    );
  }

  pub fn report_invalid_reassigned_variable(&mut self, token: &Token) {
    self.report_error(
      format!(
        "An immutable variable was reassigned '{}'",
        token.span.literal
      ),
      token.span.clone(),
    );
  }

  pub fn report_redeclared_variable(&mut self, token: &Token) {
    self.report_error(
      format!("Variable '{}' was already declared", token.span.literal),
      token.span.clone(),
    );
  }

  pub fn report_expected_after_expression(
    &mut self,
    expected: &TokenType,
    expression: &Token,
    token: &Token,
  ) {
    self.report_error(
      format!(
        "Expected '{}' after {}', found '{}'",
        expected.to_string(),
        expression.kind.to_string(),
        token.kind.to_string()
      ),
      token.span.clone(),
    );
  }

  pub fn report_expected_token(&mut self, expected: &TokenType, token: &Token) {
    self.report_error(
      format!(
        "Expected '{}', found '{}'",
        expected.to_string(),
        token.kind.to_string()
      ),
      token.span.clone(),
    );
  }

  pub fn report_invalid_number_of_arguments(
    &mut self,
    expected: &usize,
    found: &usize,
    token: &Token,
  ) {
    self.report_error(
      format!(
        "Expected {} arguments, found {}",
        expected.to_string(),
        found.to_string()
      ),
      token.span.clone(),
    );
  }

  pub fn report_expected_return_type_after_function(&mut self, token: &Token) {
    self.report_error(
      format!(
        "Expected return type after function, found '{}'",
        token.kind.to_string()
      ),
      token.span.clone(),
    );
  }

  fn report_not_callable(&mut self, expression: &Token) {
    self.report_error(
      format!("'{}' is not callable", expression.span.literal),
      expression.span.clone(),
    );
  }

  pub fn report_missing_argument(&mut self, name: &str, token: &Token) {
    self.report_error(format!("Missing argument '{}'", name), token.span.clone());
  }

  fn report_invalid_argument_type(&mut self, argument: &EvaluatorValue) {
    self.report_error(
      format!("Invalid argument type '{}'", argument.to_string()),
      TextSpan::new(0, 0, 0, argument.to_string(), 0),
    );
  }

  fn report_invalid_comparison(
    &mut self,
    left: &&EvaluatorValue,
    right: &&EvaluatorValue,
    token: &&Token,
  ) {
    self.report_error(
      format!(
        "Invalid comparison between '{}' and '{}'",
        left.to_string(),
        right.to_string()
      ),
      token.span.clone(),
    )
  }

  fn report_invalid_unary_operator(&mut self, token: &&Token) {
    self.report_error(
      format!("Invalid unary operator '{}'", token.span.literal),
      token.span.clone(),
    );
  }

  fn report_undefined_variable(&mut self, token: &Token) {
    self.report_error(
      format!("Undefined variable '{}'", token.span.literal),
      token.span.clone(),
    );
  }

  fn report_variable_already_defined(&mut self, name: &str, data_type: &DataType) {
    self.report_error(
      format!(
        "Variable '{}' was already defined as '{}'",
        name,
        data_type.to_string()
      ),
      TextSpan::new(0, 0, 0, name.to_string(), 0),
    );
  }
}
