use super::LiteralValue;

#[derive(Debug, PartialEq)]
pub struct Literal {
  pub value: LiteralValue,
}

impl Literal {
  pub fn new(value: LiteralValue) -> Self {
    Self { value }
  }
}
