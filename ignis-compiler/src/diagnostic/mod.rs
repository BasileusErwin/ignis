use crate::ast::{
  lexer::{text_span::TextSpan, token_type::TokenType, token::Token},
  data_type::DataType,
  evaluator::EvaluatorValue,
};

#[derive(Debug)]
pub enum DiagnosticLevel {
  Warning,
  Error,
}

#[derive(Debug)]
pub struct Diagnostic {
  code: DiagnosticLevel,
  span: Box<TextSpan>,
  hint: Option<String>,
}

impl Diagnostic {
  pub fn new(code: DiagnosticLevel, span: Box<TextSpan>, hint: Option<String>) -> Self {
    Self { code, span, hint }
  }
}

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

  pub fn report_undeclared_variable(&mut self, token: &Token) {
    self.report_error(
      format!("Undeclared variable '{}'", token.span.literal),
      token.span.clone(),
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

  pub fn report_invalid_redeclared_variable(&mut self, token: &Token) {
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
}
