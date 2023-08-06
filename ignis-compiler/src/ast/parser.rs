use super::{
  lexer::{token::Token, token_type::TokenType},
  expression::{
    Expression, binary::Binary, unary::Unary, literal::Literal, LiteralValue, grouping::Grouping,
  },
};

pub struct Parser {
  pub tokens: Vec<Token>,
  current: usize,
}

#[derive(Debug)]
pub struct ParserError {
  pub message: String,
  pub token: Token,
}

impl ParserError {
  pub fn new(message: String, token: Token) -> Self {
    Self { message, token }
  }
}

impl Parser {
  pub fn new(tokens: Vec<Token>) -> Self {
    Self { tokens, current: 0 }
  }

  pub fn parse(&mut self) -> Result<Expression, ParserError> {
    println!("{:?}", self.tokens);
    self.expression()
  }

  fn expression(&mut self) -> Result<Expression, ParserError> {
    self.equality()
  }

  // equelity -> comparison (("!=" | "==") comparison)*;
  fn equality(&mut self) -> Result<Expression, ParserError> {
    let mut expression = self.comparison()?;

    while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
      let operator: Token = self.previous();
      let right = self.comparison()?;

      expression = Expression::Binary(Binary::new(Box::new(expression), operator, Box::new(right)));
    }

    Ok(expression)
  }

  // comparison -> term ((">" | ">=" | "<" | "<=") term)*;
  fn comparison(&mut self) -> Result<Expression, ParserError> {
    let mut expression = self.term()?;

    while self.match_token(&[
      TokenType::Greater,
      TokenType::GreaterEqual,
      TokenType::Less,
      TokenType::LessEqual,
    ]) {
      let operator: Token = self.previous();
      let right = self.term()?;

      expression = Expression::Binary(Binary::new(Box::new(expression), operator, Box::new(right)));
    }

    Ok(expression)
  }

  // term -> factor (("-" | "+") factor)*;
  fn term(&mut self) -> Result<Expression, ParserError> {
    let mut expression = self.factor()?;

    while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
      let operator: Token = self.previous();
      let right = self.factor()?;

      expression = Expression::Binary(Binary::new(Box::new(expression), operator, Box::new(right)));
    }

    Ok(expression)
  }

  // factor -> ("!" | "-") unary | primary;
  fn factor(&mut self) -> Result<Expression, ParserError> {
    let mut expression = self.unary()?;

    while self.match_token(&[TokenType::Slash, TokenType::Asterisk]) {
      let operator: Token = self.previous();
      let right = self.unary()?;

      expression = Expression::Binary(Binary::new(Box::new(expression), operator, Box::new(right)));
    }

    Ok(expression)
  }

  // unary -> ("!" | "-") unary | primary;
  fn unary(&mut self) -> Result<Expression, ParserError> {
    if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
      let operator = self.previous();
      let right = self.unary()?;

      return Ok(Expression::Unary(Unary::new(operator, Box::new(right))));
    }

    self.primary()
  }

  fn primary(&mut self) -> Result<Expression, ParserError> {
    let token = self.peek();

    match token.kind {
      TokenType::True
      | TokenType::False
      | TokenType::Null
      | TokenType::Number
      | TokenType::String => {
        self.advance();
        Ok(Expression::Literal(Literal::new(LiteralValue::from_token(
          token.clone(),
        ))))
      }
      TokenType::LeftParen => {
        self.advance();
        let expression = self.expression()?;

        self.consume(
          TokenType::RightParen,
          String::from("Expect ')' after expression."),
        )?;

        Ok(Expression::Grouping(Grouping::new(Box::new(expression))))
      }
      _ => Err(ParserError::new(
        String::from("Expect expression."),
        self.peek(),
      )),
    }
  }

  fn consume(&mut self, kind: TokenType, message: String) -> Result<(), ParserError> {
    let token = self.peek();
    println!("Consume: {}", token.kind == kind);
    if token.kind == kind {
      self.advance();
      return Ok(());
    }

    Err(ParserError::new(message, self.peek()))
  }

  fn peek(&mut self) -> Token {
    self.tokens[self.current].clone()
  }

  fn is_at_end(&mut self) -> bool {
    self.peek().kind == TokenType::Eof
  }

  fn match_token(&mut self, kinds: &[TokenType]) -> bool {
    for kind in kinds {
      if self.check(kind.clone()) {
        self.advance();
        return true;
      }
    }

    false
  }

  fn check(&mut self, kind: TokenType) -> bool {
    if self.is_at_end() {
      return false;
    }

    self.peek().kind == kind
  }

  fn advance(&mut self) -> Token {
    if !self.is_at_end() {
      self.current += 1;
    }

    self.previous()
  }

  fn previous(&mut self) -> Token {
    self.tokens[self.current - 1].clone()
  }
}

#[cfg(test)]
mod tests {
  use crate::ast::lexer::text_span::TextSpan;
  use super::*;

  #[test]
  fn test_valid_expression_one() {
    // (3 + 3);
    let tokens = vec![
      Token {
        kind: TokenType::Number,
        span: TextSpan {
          start: 0,
          end: 1,
          literal: "3".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::Plus,
        span: TextSpan {
          start: 2,
          end: 3,
          literal: "+".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::Number,
        span: TextSpan {
          start: 4,
          end: 5,
          literal: "3".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::SemiColon,
        span: TextSpan {
          start: 5,
          end: 6,
          literal: ";".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::Eof,
        span: TextSpan {
          start: 0,
          end: 0,
          literal: "\0".to_string(),
          line: 1,
        },
      },
    ];

    let mut parser = Parser::new(tokens);

    match parser.parse() {
      Ok(result) => {
        assert_eq!(result.to_string(), "(+ 3 3)");
      }
      Err(error) => panic!("{:?}", error),
    };
  }

  #[test]
  fn test_valid_expression_two() {
    // 4 + 12 * 43;
    let tokens = vec![
      Token {
        kind: TokenType::Number,
        span: TextSpan {
          start: 0,
          end: 1,
          literal: "4".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::Plus,
        span: TextSpan {
          start: 2,
          end: 3,
          literal: "+".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::Number,
        span: TextSpan {
          start: 4,
          end: 6,
          literal: "12".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::Asterisk,
        span: TextSpan {
          start: 7,
          end: 8,
          literal: "*".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::Number,
        span: TextSpan {
          start: 9,
          end: 11,
          literal: "43".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::SemiColon,
        span: TextSpan {
          start: 11,
          end: 12,
          literal: ";".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::Eof,
        span: TextSpan {
          start: 0,
          end: 0,
          literal: "\0".to_string(),
          line: 1,
        },
      },
    ];

    let mut parser = Parser::new(tokens);

    match parser.parse() {
      Ok(result) => {
        assert_eq!(result.to_string(), "(+ 4 (* 12 43))");
      }
      Err(error) => panic!("{:?}", error),
    };
  }

  #[test]
  fn test_valid_expression_error() {
    // ( 8 +
    let tokens = vec![
      Token {
        kind: TokenType::LeftParen,
        span: TextSpan {
          start: 0,
          end: 1,
          literal: "(".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::Number,
        span: TextSpan {
          start: 2,
          end: 3,
          literal: "3".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::Plus,
        span: TextSpan {
          start: 4,
          end: 5,
          literal: "+".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::Number,
        span: TextSpan {
          start: 6,
          end: 7,
          literal: "3".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::Eof,
        span: TextSpan {
          start: 0,
          end: 0,
          literal: "\0".to_string(),
          line: 1,
        },
      },
    ];

    let mut parser = Parser::new(tokens);

    match parser.parse() {
      Ok(_) => (),
      Err(error) => assert_eq!(error.message, "Expect ')' after expression."),
    };
  }

  #[test]
  fn test_valid_expression_null() {
    // null
    let tokens = vec![
      Token {
        kind: TokenType::Null,
        span: TextSpan {
          start: 0,
          end: 4,
          literal: "null".to_string(),
          line: 0,
        },
      },
      Token {
        kind: TokenType::Eof,
        span: TextSpan {
          start: 0,
          end: 0,
          literal: "\0".to_string(),
          line: 1,
        },
      },
    ];

    let mut parser = Parser::new(tokens);

    match parser.parse() {
      Ok(result) => assert_eq!(result.to_string(), "null"),
      Err(error) => panic!("{:?}", error),
    };
  }
}
