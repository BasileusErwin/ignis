use super::{Expression, Visitor};

#[derive(Debug)]
pub struct Grouping {
  pub expression: Box<Expression>,
}

impl Grouping {
  pub fn new(expression: Box<Expression>) -> Self {
    Self { expression }
  }

  pub fn accept<R>(&self, visitor: &dyn Visitor<R>) -> R {
    visitor.visit_grouping_expression(self)
  }
}
