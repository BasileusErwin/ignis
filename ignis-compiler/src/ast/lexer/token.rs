use super::{token_type::TokenType, text_span::TextSpan};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
  pub kind: TokenType,
  pub span: TextSpan,
}

impl Token {
  pub fn new(kind: TokenType, span: TextSpan) -> Self {
    Self { kind, span }
  }
}
