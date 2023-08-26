use {
  lexer::{token::Token, text_span::TextSpan},
  ast::expression::variable::VariableExpression,
  enums::{data_type::DataType, token_type::TokenType},
  evaluator::evaluator_value::EvaluatorValue,
};

use evaluator::evaluator_error::EvaluatorDiagnosticError;
use parser::ParserDiagnosticError;

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
  InvalidReassignedVariable(TextSpan),
}

impl DiagnosticError {
  pub fn from_evaluator_error(error: EvaluatorDiagnosticError) -> Self {
    match error {
      EvaluatorDiagnosticError::UndeclaredVariable(expression) => {
        DiagnosticError::UndeclaredVariable(expression)
      }
      EvaluatorDiagnosticError::InvalidUnaryOperatorForDataType(operator, right) => {
        DiagnosticError::InvalidUnaryOperatorForDataType(operator, right)
      }
      EvaluatorDiagnosticError::NotCallable(token) => DiagnosticError::NotCallable(token),
      EvaluatorDiagnosticError::InvalidNumberOfArguments(arity, arguments, token) => {
        DiagnosticError::InvalidNumberOfArguments(arity, arguments, token)
      }
      EvaluatorDiagnosticError::AssingInvalidType(argument, data_type, name) => {
        DiagnosticError::AssingInvalidType(argument, data_type, name)
      }
      EvaluatorDiagnosticError::InvalidArgumentType(argument) => {
        DiagnosticError::InvalidArgumentType(argument)
      }
      EvaluatorDiagnosticError::MissingArgument(name, token) => {
        DiagnosticError::MissingArgument(name, token)
      }
      EvaluatorDiagnosticError::InvalidComparison(left, right, token) => {
        (DiagnosticError::InvalidComparison(left, right, token))
      }
      EvaluatorDiagnosticError::InvalidOperator(token) => DiagnosticError::InvalidOperator(token),
      EvaluatorDiagnosticError::InvalidUnaryOperator(token) => {
        DiagnosticError::InvalidUnaryOperator(token)
      }
      EvaluatorDiagnosticError::UndefinedVariable(token) => {
        DiagnosticError::UndefinedVariable(token)
      }
      EvaluatorDiagnosticError::VariableAlreadyDefined(name, data_type) => {
        DiagnosticError::VariableAlreadyDefined(name, data_type)
      }
      EvaluatorDiagnosticError::InvalidAssignmentTarget(span) => {
        DiagnosticError::InvalidAssignmentTarget(span)
      }
      EvaluatorDiagnosticError::InvalidReassignedVariable(span) => {
        DiagnosticError::InvalidReassignedVariable(span)
      }
    }
  }

  pub fn from_parser_diagnostic(errors: Vec<ParserDiagnosticError>) -> Vec<Self> {
    let mut diagnostics: Vec<Self> = Vec::new();

    for error in errors {
      match error {
        ParserDiagnosticError::ExpectedExpression(token) => {
          diagnostics.push(DiagnosticError::ExpectedExpression(token));
        }
        ParserDiagnosticError::ExpectedToken(kind, token) => {
          diagnostics.push(DiagnosticError::ExpectedToken(kind, token));
        }
        ParserDiagnosticError::ExpectedVariableName(token) => {
          diagnostics.push(DiagnosticError::ExpectedVariableName(token));
        }
        ParserDiagnosticError::ExpectedReturnTypeAfterFunction(token) => {
          diagnostics.push(DiagnosticError::ExpectedReturnTypeAfterFunction(token));
        }
        ParserDiagnosticError::ExpectedAfterExpression(kind, expression, token) => {
          diagnostics.push(DiagnosticError::ExpectedAfterExpression(
            kind, expression, token,
          ));
        }
        ParserDiagnosticError::ExpectedExpressionAfter(token) => {
          diagnostics.push(DiagnosticError::ExpectedExpressionAfter(token));
        }
        ParserDiagnosticError::UnexpectedToken(kind, token) => {
          diagnostics.push(DiagnosticError::UnexpectedToken(kind, token));
        }
        ParserDiagnosticError::InvalidAssignmentTarget(span) => {
          diagnostics.push(DiagnosticError::InvalidAssignmentTarget(span));
        }
        ParserDiagnosticError::ExpectedTypeAfterVariable(token) => {
          diagnostics.push(DiagnosticError::ExpectedTypeAfterVariable(token));
        }
        ParserDiagnosticError::InvalidNumberOfArguments(arity, arguments, token) => {
          diagnostics.push(DiagnosticError::InvalidNumberOfArguments(
            arity, arguments, token,
          ));
        }
      };
    }

    diagnostics
  }

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
      DiagnosticError::InvalidReassignedVariable(span) => {
        diagnostics.report_invalid_reassigned_variable(&span);
      }
    }
  }
}
