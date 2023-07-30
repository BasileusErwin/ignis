use super::{token_type::TokenType, token::Token, text_span::TextSpan};

/*
 * Lexer
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
      tokens: vec![],
      start: 0,
      line: 0,
      current: 0,
    }
  }

  /**
  The scanner checks all characters in the code and
  enlarges tokens until it runs out of characters.
  At the end a final token of type **EOF** is added.
  */
  pub fn scan_tokens(&mut self) {
    loop {
      self.start = self.current;

      self.scan_token();
      if self.is_at_end() {
        break;
      }
    }

    self.tokens.push(Token::new(
      TokenType::Eof,
      TextSpan::new(0, 0, self.line, '\0'.to_string()),
    ));
  }

  /**
  Help function that checks that all characters have been completed.
  */
  fn is_at_end(&self) -> bool {
    self.current >= self.source.len()
  }

  fn scan_token(&mut self) {
    let c = self.advance();

    if c.is_ascii_digit() {
      let number = self.number();

      self.add_token(TokenType::Number(number));
      return;
    }

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

        self.add_token(token);
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
      _ => self.add_token(TokenType::Bad),
    }
  }

  /**
  This method receives a character.
  It checks if the next character is a space or a line break
  or if the next character does not match the one passed by parameter,
  if these cases are met then it returns `false`.
  Otherwise, it increments `current` by one and returns true.
  */
  fn match_char(&mut self, c: char) -> bool {
    if self.is_at_end() || self.peek() != c {
      return false;
    }

    self.current += 1;

    true
  }

  /**
  Method that gets the current character in the source code and increments it into a `current`.
  */
  fn advance(&mut self) -> char {
    self.current += 1;
    self.source.chars().nth(self.current - 1).unwrap_or('\0')
  }

  fn number(&mut self) -> f32 {
    while self.peek().is_ascii_digit() {
      self.advance();
    }

    if self.peek() == '.' && self.peek_next().is_ascii_digit() {
      self.advance();

      while self.peek().is_ascii_digit() {
        self.advance();
      }
    }

    self.source[self.start..self.current].parse().unwrap()
  }

  fn peek_next(&self) -> char {
    self.source.chars().nth(self.current + 1).unwrap_or('\0')
  }

  /**
  This method takes returns the current character
  if it is not a line break.
  */
  fn peek(&self) -> char {
    if self.is_at_end() {
      '\0'
    } else {
      self.source.chars().nth(self.current).unwrap()
    }
  }

  /**
  Where `advance()` is for input, `addToken()` is for output.
  It takes the text of the current lexeme and creates a new token.
  */
  fn add_token(&mut self, kind: TokenType) {
    let literal = self.source[self.start..self.current].to_string();

    self.tokens.push(Token::new(
      kind,
      TextSpan::new(self.start, self.current, self.line, literal),
    ));
  }
}