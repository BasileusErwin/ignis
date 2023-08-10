use super::{
  lexer::{token::Token, token_type::TokenType},
  expression::{
    Expression, binary::Binary, unary::Unary, literal::Literal, LiteralValue, grouping::Grouping,
  },
  statement::{Statement, variable::Variable, expression::ExpressionStatement},
  data_type::DataType,
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

  pub fn parse(&mut self) -> Result<Vec<Statement>, Vec<ParserError>> {
    let mut statements: Vec<Statement> = vec![];
    let mut error: Vec<ParserError> = vec![];

    while !self.is_at_end() {
      match self.declaration() {
        Ok(statement) => statements.push(statement),
        Err(err) => {
          error.push(err);
          self.synchronize();
        }
      }
    }

    if error.len() == 0 {
      Ok(statements)
    } else {
      Err(error)
    }
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
      let data_type: DataType = DataType::Boolean;

      expression = Expression::Binary(Binary::new(
        Box::new(expression),
        operator,
        Box::new(right),
        data_type,
      ));
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
      let data_type: DataType = DataType::Boolean;

      expression = Expression::Binary(Binary::new(
        Box::new(expression),
        operator,
        Box::new(right),
        data_type,
      ));
    }

    Ok(expression)
  }

  // term -> factor (("-" | "+") factor)*;
  fn term(&mut self) -> Result<Expression, ParserError> {
    let mut expression = self.factor()?;

    while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
      let operator: Token = self.previous();
      let right = self.factor()?;

      let left_type = self.get_expression_type(&expression);
      let right_type = self.get_expression_type(&right);
      let operator_kind = operator.kind.clone();

      expression = Expression::Binary(Binary::new(
        Box::new(expression),
        operator,
        Box::new(right),
        self.get_data_type_by_operator(Some(left_type), right_type, operator_kind)?,
      ));
    }

    Ok(expression)
  }

  // factor -> ("!" | "-") unary | primary;
  fn factor(&mut self) -> Result<Expression, ParserError> {
    let mut expression = self.unary()?;

    while self.match_token(&[TokenType::Slash, TokenType::Asterisk]) {
      let operator: Token = self.previous();
      let right = self.unary()?;

      let left_type = self.get_expression_type(&expression);
      let right_type = self.get_expression_type(&right);

      let operator_kind = operator.kind.clone();

      expression = Expression::Binary(Binary::new(
        Box::new(expression),
        operator,
        Box::new(right),
        self.get_data_type_by_operator(Some(left_type), right_type, operator_kind)?,
      ));
    }

    Ok(expression)
  }

  // unary -> ("!" | "-") unary | primary;
  fn unary(&mut self) -> Result<Expression, ParserError> {
    if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
      let operator = self.previous();
      let right = self.unary()?;

      let right_type = self.get_expression_type(&right);
      let operator_kind = operator.kind.clone();

      return Ok(Expression::Unary(Unary::new(
        operator,
        Box::new(right),
        self.get_data_type_by_operator(None, right_type, operator_kind)?,
      )));
    }

    self.primary()
  }

  fn primary(&mut self) -> Result<Expression, ParserError> {
    let token = self.peek();

    match token.kind {
      TokenType::True
      | TokenType::False
      | TokenType::Null
      | TokenType::Int
      | TokenType::Double
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

  fn get_data_type_by_operator(
    &mut self,
    left: Option<DataType>,
    right: DataType,
    operator: TokenType,
  ) -> Result<DataType, ParserError> {
    match (left, right, operator) {
      (Some(DataType::Int), DataType::Int, TokenType::Plus)
      | (Some(DataType::Int), DataType::Int, TokenType::Minus)
      | (None, DataType::Int, TokenType::Minus) => Ok(DataType::Int),
      (Some(DataType::Double), DataType::Double, TokenType::Plus) => Ok(DataType::Double),
      (Some(DataType::Double), DataType::Double, TokenType::Minus)
      | (None, DataType::Double, TokenType::Minus) => Ok(DataType::Double),
      (Some(DataType::String), DataType::String, TokenType::Plus) => Ok(DataType::String),
      (None, DataType::Boolean, TokenType::Bang) | (None, DataType::String, TokenType::Bad) => {
        Ok(DataType::Boolean)
      }
      _ => Err(ParserError::new("Error type".to_string(), self.peek())),
    }
  }

  fn get_expression_type(&self, expression: &Expression) -> DataType {
    match expression {
      Expression::Binary(binary) => binary.data_type.clone(),
      Expression::Unary(unary) => unary.data_type.clone(),
      Expression::Literal(literal) => match literal.value {
        LiteralValue::Boolean(_) => DataType::Boolean,
        LiteralValue::Char(_) => DataType::Char,
        LiteralValue::Double(_) => DataType::Double,
        LiteralValue::Int(_) => DataType::Int,
        LiteralValue::String(_) => DataType::String,
        _ => DataType::Int,
      },
      Expression::Grouping(grouping) => self.get_expression_type(&grouping.expression),
    }
  }

  fn synchronize(&mut self) {
    self.advance();

    while !self.is_at_end() {
      if self.previous().kind == TokenType::SemiColon {
        return;
      }

      match self.peek().kind {
        TokenType::Class
        | TokenType::Function
        | TokenType::Let
        | TokenType::Const
        | TokenType::For
        | TokenType::If
        | TokenType::Return => return,
        _ => (),
      };

      self.advance();
    }
  }

  fn declaration(&mut self) -> Result<Statement, ParserError> {
    if self.match_token(&[TokenType::Let, TokenType::Const]) {
      return self.variable_declaration();
    }

    self.statement()
  }

  fn variable_declaration(&mut self) -> Result<Statement, ParserError> {
    let name: Token = self.consume(TokenType::Identifier, "Expect varible name.".to_string())?;

    let mut initializer: Option<Expression> = None;
    let type_annotation: DataType;

    match self.consume(
      TokenType::Colon,
      "Type expected before assignments".to_string(),
    ) {
      Ok(_) => {
        let kind = DataType::from_token_type(self.peek().kind);
        if kind == DataType::None {
          return Err(ParserError::new(
            "Type expected before assignments".to_string(),
            self.peek(),
          ));
        }

        type_annotation = kind;
      }
      Err(error) => return Err(error),
    }

    self.advance();

    if self.match_token(&[TokenType::Equal]) {
      initializer = Some(self.expression()?);
    }

    self.consume(
      TokenType::SemiColon,
      "Expect ';' after variable declaration".to_string(),
    )?;

    if let Some(ini) = initializer {
      Ok(Statement::Variable(Variable::new(
        Box::new(name),
        Box::new(ini),
        Box::new(type_annotation),
      )))
    } else {
      Err(ParserError::new("Expect expression.".to_string(), name))
    }
  }

  fn statement(&mut self) -> Result<Statement, ParserError> {
    self.expression_statement()
  }

  fn expression_statement(&mut self) -> Result<Statement, ParserError> {
    let expression = self.expression()?;

    self.consume(
      TokenType::SemiColon,
      "Expect ';' after expression.".to_string(),
    )?;

    Ok(Statement::Expression(ExpressionStatement::new(Box::new(
      expression,
    ))))
  }

  fn consume(&mut self, kind: TokenType, message: String) -> Result<Token, ParserError> {
    let token: Token = self.peek();
    if token.kind == kind {
      self.advance();
      return Ok(token);
    }

    Err(ParserError::new(message, token))
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
