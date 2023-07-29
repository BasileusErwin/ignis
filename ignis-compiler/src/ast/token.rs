use super::{token_type::TokenType, text_span::TextSpan};

#[derive(Debug)]
pub struct Token {
  kind: TokenType,
  span: TextSpan,
}

impl Token {
  pub fn new(kind: TokenType, span: TextSpan) -> Self {
    Self { kind, span }
  }
}
