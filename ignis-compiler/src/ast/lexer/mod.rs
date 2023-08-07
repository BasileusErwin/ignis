pub mod text_span;
pub mod token;
pub mod token_type;

use super::lexer::{text_span::TextSpan, token::Token, token_type::TokenType};

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
    let c: char = self.advance();

    if c.is_ascii_digit() {
      self.number();

      self.add_token(TokenType::Number);
      return;
    }

    if c.is_alphabetic() {
      let kind: TokenType = self.identifier();
      self.add_token(kind);
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
        let token: TokenType = if self.match_char('=') {
          TokenType::BangEqual
        } else {
          TokenType::Bang
        };

        self.add_token(token)
      }
      '=' => {
        let token: TokenType = if self.match_char('=') {
          TokenType::EqualEqual
        } else if self.match_char('>') {
          TokenType::Arrow
        } else {
          TokenType::Equal
        };

        self.add_token(token)
      }
      '<' => {
        let token: TokenType = if self.match_char('=') {
          TokenType::LessEqual
        } else {
          TokenType::Less
        };

        self.add_token(token)
      }
      '>' => {
        let token: TokenType = if self.match_char('=') {
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
          TokenType::Pipe
        };

        self.add_token(token);
      }
      '&' => {
        let token = if self.match_char('&') {
          TokenType::And
        } else {
          TokenType::Ampersand
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
      '"' => {
        if let Some(_) = self.string() {
          self.add_token(TokenType::String);
        }
      }
      '\n' => self.line += 1,
      ' ' | '\r' | '\t' => (),
      _ => self.add_token(TokenType::Bad),
    }
  }

  fn get_keyword(key: &str) -> Option<TokenType> {
    match key {
      "class" => Some(TokenType::Class),
      "super" => Some(TokenType::Super),
      "else" => Some(TokenType::Else),
      "false" => Some(TokenType::False),
      "true" => Some(TokenType::True),
      "function" => Some(TokenType::Function),
      "for" => Some(TokenType::For),
      "if" => Some(TokenType::If),
      "null" => Some(TokenType::Null),
      "return" => Some(TokenType::Return),
      "this" => Some(TokenType::This),
      "let" => Some(TokenType::Let),
      "const" => Some(TokenType::Const),
      "while" => Some(TokenType::While),
      "enum" => Some(TokenType::Enum),
      "export" => Some(TokenType::Export),
      "import" => Some(TokenType::Import),
      "from" => Some(TokenType::From),
      "mut" => Some(TokenType::Mut),
      "as" => Some(TokenType::As),
      "break" => Some(TokenType::Break),
      "readonly" => Some(TokenType::ReadOnly),
      "static" => Some(TokenType::Static),
      "final" => Some(TokenType::Final),
      "public" => Some(TokenType::Public),
      "private" => Some(TokenType::Private),
      "interface" => Some(TokenType::Interface),
      "extends" => Some(TokenType::Extends),
      "implements" => Some(TokenType::Implements),
      "string" => Some(TokenType::StringType),
      "boolean" => Some(TokenType::BooleanType),
      "int" => Some(TokenType::IntType),
      "number" => Some(TokenType::NumberType),
      "double" => Some(TokenType::DoubleType),
      "char" => Some(TokenType::CharType),
      _ => None,
    }
  }

  fn is_identifier_starter(&self) -> bool {
    let c: char = self.peek();

    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
  }

  fn is_identifier_letter(&self) -> bool {
    let c: char = self.peek();

    (c >= '0' && c <= '9') || (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c == '_')
  }

  fn identifier(&mut self) -> TokenType {
    while self.is_identifier_starter() || self.is_identifier_letter() {
      self.advance();
    }

    let value: String = self.source[self.start..self.current].to_string();
    let kind: Option<TokenType> = Self::get_keyword(value.as_str());

    kind.unwrap_or(TokenType::Identifier)
  }

  fn string(&mut self) -> Option<String> {
    while self.peek() != '\"' && !self.is_at_end() {
      self.advance();
    }

    if self.is_at_end() {
      // TODO: Error
      return None;
    }

    self.advance();

    Some(self.source[self.start..self.current].to_string())
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

  fn number(&mut self) -> () {
    while self.peek().is_ascii_digit() {
      self.advance();
    }

    if self.peek() == '.' && self.peek_next().is_ascii_digit() {
      self.advance();

      while self.peek().is_ascii_digit() {
        self.advance();
      }
    }
  }

  fn peek_next(&self) -> char {
    self.source.chars().nth(self.current + 1).unwrap_or('\0')
  }

  /**
  This method takes returns the current character
  if it is not a line break.
  */
  fn peek(&self) -> char {
    self.source.chars().nth(self.current).unwrap_or('\0')
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_valid_indentifiers() {
    let source: &str = "let helloWorld = \"Hello World\";";
    let mut lexer: Lexer<'_> = Lexer::new(source);
    lexer.scan_tokens();

    assert_eq!(lexer.tokens.len(), 6);
    assert_eq!(lexer.tokens[0].kind, TokenType::Let);
    assert_eq!(lexer.tokens[1].kind, TokenType::Identifier);
    assert_eq!(lexer.tokens[2].kind, TokenType::Equal);

    assert_eq!(lexer.tokens[3].kind, TokenType::String);
    assert_eq!(lexer.tokens[3].span.literal, "\"Hello World\"".to_string());

    assert_eq!(lexer.tokens[4].kind, TokenType::SemiColon);
    assert_eq!(lexer.tokens[5].kind, TokenType::Eof);
  }

  #[test]
  fn test_valid_expression() {
    let source: &str = "(3 + 5) * 12;";
    let mut lexer: Lexer<'_> = Lexer::new(source);
    lexer.scan_tokens();

    assert_eq!(lexer.tokens.len(), 9);
    assert_eq!(lexer.tokens[0].kind, TokenType::LeftParen);
    assert_eq!(lexer.tokens[1].kind, TokenType::Number);
    assert_eq!(lexer.tokens[1].span.literal, "3".to_string());

    assert_eq!(lexer.tokens[2].kind, TokenType::Plus);

    assert_eq!(lexer.tokens[3].kind, TokenType::Number);
    assert_eq!(lexer.tokens[3].span.literal, "5".to_string());

    assert_eq!(lexer.tokens[4].kind, TokenType::RightParen);
    assert_eq!(lexer.tokens[5].kind, TokenType::Asterisk);

    assert_eq!(lexer.tokens[6].kind, TokenType::Number);
    assert_eq!(lexer.tokens[6].span.literal, "12".to_string());

    assert_eq!(lexer.tokens[7].kind, TokenType::SemiColon);
    assert_eq!(lexer.tokens[8].kind, TokenType::Eof);
  }

  #[test]
  fn test_valid_null() {
    let source: &str = "null";
    let mut lexer: Lexer<'_> = Lexer::new(source);
    lexer.scan_tokens();

    assert_eq!(lexer.tokens.len(), 2);
    assert_eq!(lexer.tokens[0].kind, TokenType::Null);
    assert_eq!(lexer.tokens[1].kind, TokenType::Eof);
  }

  #[test]
  fn test_valid_key_boolean() {
    let source: &str = "false; true;";
    let mut lexer: Lexer<'_> = Lexer::new(source);
    lexer.scan_tokens();

    assert_eq!(lexer.tokens.len(), 5);
    assert_eq!(lexer.tokens[0].kind, TokenType::False);
    assert_eq!(lexer.tokens[1].kind, TokenType::SemiColon);
    assert_eq!(lexer.tokens[2].kind, TokenType::True);
    assert_eq!(lexer.tokens[3].kind, TokenType::SemiColon);
    assert_eq!(lexer.tokens[4].kind, TokenType::Eof);
  }
}
