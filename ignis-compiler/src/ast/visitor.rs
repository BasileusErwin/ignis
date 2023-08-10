use super::{
  expression::{Expression, binary::Binary, literal::Literal, unary::Unary, grouping::Grouping},
  statement::{expression::ExpressionStatement, variable::Variable},
};

pub trait Visitor<R> {
  fn visit_binary_expression(&self, expression: &Binary) -> R;
  fn visit_grouping_expression(&self, expression: &Grouping) -> R;
  fn visit_literal_expression(&self, expression: &Literal) -> R;
  fn visit_unary_expression(&self, expression: &Unary) -> R;
  fn visit_expression_statement(&self, statement: &ExpressionStatement) -> R;
  fn visit_variable_statement(&self, variable: &Variable) -> R;
}
