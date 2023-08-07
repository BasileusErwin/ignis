use crate::ast::expression::Expression;
use super::Visitor;

#[derive(Debug)]
pub struct ExpressionStatement {
  expression: Box<Expression>,
}

impl ExpressionStatement {
  pub fn new(expression: Box<Expression>) -> Self {
    Self { expression }
  }

	pub fn accept<R>(&self, visitor: &dyn Visitor<R>) -> R {
			visitor.visit_expression_statement(self)
  }
}
