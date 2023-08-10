use crate::ast::expression::Expression;

#[derive(Debug)]
pub struct ExpressionStatement {
  pub expression: Box<Expression>,
}

impl ExpressionStatement {
  pub fn new(expression: Box<Expression>) -> Self {
    Self { expression }
  }
}
