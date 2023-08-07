use super::{Visitor, LiteralValue};

#[derive(Debug)]
pub struct Literal {
  pub value: LiteralValue,
}

impl Literal {
  pub fn new(value: LiteralValue) -> Self {
    Self { value }
  }

  pub fn accept<R>(&self, visitor: &dyn Visitor<R>) -> R {
    visitor.visit_literal_expression(self)
  }
}
