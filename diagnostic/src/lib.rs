pub mod error;
pub mod warning;

use std::fmt::{Display, format};

use {
  lexer::{text_span::TextSpan, token::Token},
  ast::expression::variable::VariableExpression,
  enums::{data_type::DataType, token_type::TokenType},
  analyzer::analyzer_value::AnalyzerValue,
};

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
  pub module_path: Option<String>,
}

impl Diagnostic {
  pub fn new(
    code: DiagnosticLevel,
    span: Box<TextSpan>,
    hint: Option<String>,
    module_path: Option<String>,
  ) -> Self {
    Self {
      code,
      span,
      hint,
      module_path,
    }
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
    let module_path = span.file.clone();
    self.diagnostics.push(Diagnostic::new(
      DiagnosticLevel::Error,
      Box::new(span),
      Some(message),
      Some(module_path),
    ));
  }

  pub fn report_warning(&mut self, message: String, span: TextSpan) {
    let module_path = span.file.clone();
    self.diagnostics.push(Diagnostic::new(
      DiagnosticLevel::Warning,
      Box::new(span),
      Some(message),
      Some(module_path),
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
    left: &AnalyzerValue,
    right: &AnalyzerValue,
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
    right: &AnalyzerValue,
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

  pub fn report_invalid_reassigned_variable(&mut self, token: &TextSpan) {
    self.report_error(
      format!("An immutable variable was reassigned '{}'", token.literal),
      token.clone(),
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

  fn report_invalid_argument_type(&mut self, argument: &AnalyzerValue) {
    self.report_error(
      format!("Invalid argument type '{}'", argument.to_string()),
      TextSpan::new(0, 0, 0, argument.to_string(), 0, "".to_string()),
    );
  }

  fn report_invalid_comparison(
    &mut self,
    left: &AnalyzerValue,
    right: &AnalyzerValue,
    token: &Token,
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
      TextSpan::new(0, 0, 0, name.to_string(), 0, "".to_string()),
    );
  }

  fn report_type_mismatch(&mut self, expected: &DataType, found: &DataType, token: &Token) {
    self.report_error(
      format!(
        "Type mismatch, expected '{}', found '{}'",
        expected.to_string(),
        found.to_string()
      ),
      token.span.clone(),
    );
  }

  fn report_type_mismatch_unary(&mut self, right: &DataType, token: &Token) {
    self.report_error(
      format!("Type mismatch, expected '{}'", right.to_string()),
      token.span.clone(),
    );
  }

  fn report_cannot_subtract(&mut self, left: &AnalyzerValue, right: &AnalyzerValue, token: &Token) {
    self.report_error(
      format!(
        "Cannot subtract '{}' from '{}'",
        right.to_string(),
        left.to_string()
      ),
      token.span.clone(),
    );
  }

  fn report_cannot_multiply(&mut self, left: &AnalyzerValue, right: &AnalyzerValue, token: &Token) {
    self.report_error(
      format!(
        "Cannot multiply '{}' with '{}'",
        left.to_string(),
        right.to_string()
      ),
      token.span.clone(),
    );
  }

  fn report_cannot_divide(&mut self, left: &AnalyzerValue, right: &AnalyzerValue, token: &Token) {
    self.report_error(
      format!(
        "Cannot divide '{}' with '{}'",
        left.to_string(),
        right.to_string()
      ),
      token.span.clone(),
    );
  }

  fn report_argument_type_mismatch(
    &mut self,
    expected: &DataType,
    recived: &DataType,
    token: &Token,
  ) {
    self.report_error(
      format!(
        "Argument type mismatch, expected '{}', found '{}'",
        expected.to_string(),
        recived.to_string()
      ),
      token.span.clone(),
    );
  }

  fn report_class_already_defined(&mut self, name: &str) {
    self.report_error(
      format!("Class '{}' was already defined", name),
      TextSpan::new(0, 0, 0, name.to_string(), 0, "".to_string()),
    );
  }

  fn report_function_already_defined(&mut self, name: &str, token: &Token) {
    self.report_error(
      format!("Function '{}' was already defined", name),
      token.span.clone(),
    );
  }

  fn report_cannot_modulo(&mut self, left: &AnalyzerValue, right: &AnalyzerValue, token: &Token) {
    self.report_error(
      format!(
        "Cannot modulo '{}' with '{}'",
        left.to_string(),
        right.to_string()
      ),
      token.span.clone(),
    );
  }

  fn report_immutable_variable_as_mutable_parameter(
    &mut self,
    parameter_name: &str,
    variable_name: &str,
    token: &&Token,
  ) {
    self.report_error(
      format!(
        "Cannot use immutable variable '{}' as mutable parameter '{}'",
        variable_name, parameter_name
      ),
      token.span.clone(),
    );
  }

  fn report_return_outside_function(&mut self, token: &Token) {
    self.report_error(format!("Return outside function"), token.span.clone())
  }

  fn report_not_iterable(&mut self, token: &Token) {
    self.report_error(format!("Not iterable"), token.span.clone());
  }

  fn report_array_element_type_mismatch(&mut self, token: &Token) {
    self.report_error(format!("Array element type mismatch"), token.span.clone())
  }

  fn report_module_not_found(&mut self, token: &Token) {
    self.report_error(
      format!("Module not found: {}", token.span.literal),
      token.span.clone(),
    )
  }

  fn report_imported_function_is_not_exported(&mut self, token: &Token) {
    self.report_error(
      format!("Imported function is not exported: {}", token.span.literal),
      token.span.clone(),
    )
  }

  fn report_break_outside_loop(&mut self, token: &Token) {
    self.report_error(format!("Break outside loop"), token.span.clone());
  }

  fn report_continue_outside_loop(&mut self, token: &Token) {
    self.report_error(format!("Continue outside loop"), token.span.clone());
  }
}
