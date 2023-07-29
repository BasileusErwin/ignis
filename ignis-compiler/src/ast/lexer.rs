use std::{iter::Peekable, str::Chars};

use super::{token_type::TokenType, token::Token, text_span::TextSpan};


/*
  * Scanner
  *
  * The `start` and `current` variables allow each **string** to be indexed.
  *
  * - start: points to the first character of the lexeme being scanned
  * - current: points to the character currently being checked.
  * - line: traces the source line of `current` to know the location of the
  * **tokens**.
*/
pub struct Lexer<'a> {
  source: &'a str,
  pub tokens: Vec<Token>,
  start: usize,
  line: usize,
  current: usize,
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str) -> Self {
    Self {
      source,
      start: 0,
      line: 0,
      tokens: vec![],
      current: 0,
    }
  }

  /**
  *
  */
  pub fn scanTokens(&mut self) {
    while !self.is_at_end() {
      self.start = self.current;

      self.scan_token();
    }

    self.tokens.push(Token::new(
      TokenType::Eof,
      TextSpan::new(0, 0, 0, '\0'.to_string()),
    ));
  }

  fn is_at_end(&self) -> bool {
    self.current >= self.source.len()
  }

  fn scan_token(&mut self) {
    let c = self.peek();
    println!("Character: {}", &c);

    match c {
      '(' => self.add_token(TokenType::LeftParen),
      ')' => self.add_token(TokenType::RightParen),
      '{' => self.add_token(TokenType::LeftBrace),
      '}' => self.add_token(TokenType::RightBrace),
      '[' => self.add_token(TokenType::LeftBrack),
      ']' => self.add_token(TokenType::RightBrack),
      ',' => self.add_token(TokenType::Comma),
      '.' => self.add_token(TokenType::Dot),
      ';' => self.add_token(TokenType::SemiColon),
      '-' => self.add_token(TokenType::Minus),
      '+' => self.add_token(TokenType::Plus),
      '*' => self.add_token(TokenType::Asterisk),
      ':' => self.add_token(TokenType::Colon),
      '!' => {
        let token = if self.match_char('=') {
          TokenType::BangEqual
        } else {
          TokenType::Bang
        };

        self.add_token(token)
      }
      '=' => {
        let token = if self.match_char('=') {
          TokenType::EqualEqual
        } else {
          TokenType::Equal
        };

        self.add_token(token)
      }
      '<' => {
        let token = if self.match_char('=') {
          TokenType::LessEqual
        } else {
          TokenType::Less
        };

        self.add_token(token)
      }
      '>' => {
        let token = if self.match_char('=') {
          TokenType::GreaterEqual
        } else {
          TokenType::Greater
        };

        self.add_token(token)
      }
      '|' => {
        let token = if self.match_char('|') {
          TokenType::Or
        } else {
          TokenType::Bad
        };

        self.add_token(token)
      }
      '&' => {
        let token = if self.match_char('&') {
          TokenType::And
        } else {
          TokenType::Bad
        };

        self.add_token(token)
      }
      '/' => {
        if self.match_char('*') {
          while !self.is_at_end() {
            if self.match_char('*') && self.match_char('/') {
              break;
            }

            self.advance();
          }
        } else if self.match_char('/') {
          while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
          }
        } else {
          self.add_token(TokenType::Slash)
        }
      }
      // '"' => Self::string(self),
      '\n' => self.line += 1,
      ' ' | '\r' | '\t' => (),
      _ => {
        if Self::is_number(&c) {
          let number = self.number();

          self.add_token(TokenType::Number(number));
        } else {
          self.add_token(TokenType::Bad);
        }
      }
    }

    self.current += 1;
  }

  fn match_char(&mut self, c: char) -> bool {
    if Self::is_at_end(self) || Self::peek(self) != c {
      return false;
    }

    self.current += 1;

    true
  }

  fn advance(&mut self) -> char {
    self.current += 1;
    self.source.chars().nth(self.current).unwrap()
  }

  fn number(&mut self) -> f32 {
    while self.peek().is_ascii_digit() {
      self.advance();
    }

    let next_char = self.peek_next().unwrap();

    if self.match_char('.') && next_char.is_ascii_digit() {
      self.advance();

      while self.peek().is_ascii_digit() {
        self.advance();
      }
    }

    self.source[self.start..self.current].parse().unwrap()
  }

  fn peek_next(&self) -> Option<char> {
    if self.current + 1 > self.source.len() {
      return None;
    }

    self.source.chars().nth(self.current + 1)
  }

  fn is_number(c: &char) -> bool {
    c.is_ascii_digit()
  }

  fn peek(&self) -> char {
    if self.is_at_end() {
      '\0'
    } else {
      self.source.chars().nth(self.current).unwrap()
    }
  }

  fn add_token(&mut self, kind: TokenType) {
    let literal = self.source[self.start..self.current + 1].to_string();

    self.tokens.push(Token::new(
      kind,
      TextSpan::new(self.start, self.current, self.line, literal),
    ));
  }
}
