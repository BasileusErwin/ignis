#[derive(Debug, Clone, PartialEq)]
pub struct TextSpan {
  pub start: usize,
  pub end: usize,
  pub literal: String,
  pub line: usize,
  pub column: usize,
}

impl TextSpan {
  pub fn new(start: usize, end: usize, line: usize, literal: String, column: usize) -> Self {
    Self {
      start,
      end,
      line,
      literal,
      column,
    }
  }
}
