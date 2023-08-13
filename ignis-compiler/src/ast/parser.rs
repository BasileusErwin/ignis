use crate::diagnostic::{DiagnosticList, Diagnostic};

use super::{
  lexer::{token::Token, token_type::TokenType},
  expression::{
    Expression, binary::Binary, unary::Unary, literal::Literal, LiteralValue, grouping::Grouping,
    logical::Logical, assign::Assign, variable::VariableExpression,
  },
  statement::{Statement, variable::Variable, expression::ExpressionStatement},
  data_type::DataType,
};

pub struct Parser {
  pub tokens: Vec<Token>,
  current: usize,
  pub diagnostics: DiagnosticList,
}

impl Parser {
  pub fn new(tokens: Vec<Token>) -> Self {
    Self {
      tokens,
      current: 0,
      diagnostics: DiagnosticList::new(),
    }
  }

  pub fn parse(&mut self) -> Result<Vec<Statement>, ()> {
    let mut statements: Vec<Statement> = vec![];
    while !self.is_at_end() {
      match self.declaration() {
        Ok(statement) => statements.push(statement),
        Err(_) => {
          self.synchronize();
        }
      }
    }

    if self.diagnostics.diagnostics.len() > 0 {
      Err(())
    } else {
      Ok(statements)
    }
  }

  fn expression(&mut self) -> Result<Expression, ()> {
    self.assignment()
  }

  // equelity -> comparison (("!=" | "==") comparison)*;
  fn equality(&mut self) -> Result<Expression, ()> {
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
  fn comparison(&mut self) -> Result<Expression, ()> {
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
  fn term(&mut self) -> Result<Expression, ()> {
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
        self.get_data_type_by_operator(Some(left_type), right_type, operator_kind),
      ));
    }

    Ok(expression)
  }

  // factor -> ("!" | "-") unary | primary;
  fn factor(&mut self) -> Result<Expression, ()> {
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
        self.get_data_type_by_operator(Some(left_type), right_type, operator_kind),
      ));
    }

    Ok(expression)
  }

  // unary -> ("!" | "-") unary | primary;
  fn unary(&mut self) -> Result<Expression, ()> {
    if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
      let operator = self.previous();
      let right = self.unary()?;

      let right_type = self.get_expression_type(&right);
      let operator_kind = operator.kind.clone();

      return Ok(Expression::Unary(Unary::new(
        operator,
        Box::new(right),
        self.get_data_type_by_operator(None, right_type, operator_kind),
      )));
    }

    self.primary()
  }

  fn primary(&mut self) -> Result<Expression, ()> {
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

        self.consume(TokenType::RightParen)?;

        Ok(Expression::Grouping(Grouping::new(Box::new(expression))))
      }
      TokenType::Identifier => {
        self.advance();
        let kind = token.kind.clone();
        return Ok(Expression::Variable(VariableExpression::new(
          token,
          DataType::from_token_type(kind),
        )));
      }
      _ => {
        self.diagnostics.report_expected_expression(&token);
        return Err(());
      }
    }
  }

  fn get_data_type_by_operator(
    &mut self,
    left: Option<DataType>,
    right: DataType,
    operator: TokenType,
  ) -> DataType {
    match (left, right, operator) {
      (Some(DataType::Int), DataType::Int, TokenType::Plus)
      | (Some(DataType::Int), DataType::Int, TokenType::Minus)
      | (None, DataType::Int, TokenType::Minus)
      | (Some(DataType::Int), DataType::Int, TokenType::Asterisk)
      | (Some(DataType::Int), DataType::Int, TokenType::Slash) => DataType::Int,
      (Some(DataType::Double), DataType::Double, TokenType::Plus)
      | (Some(DataType::Double), DataType::Double, TokenType::Minus)
      | (Some(DataType::Double), DataType::Double, TokenType::Slash)
      | (Some(DataType::Double), DataType::Double, TokenType::Asterisk)
      | (None, DataType::Double, TokenType::Minus) => DataType::Double,
      (Some(DataType::String), DataType::String, TokenType::Plus) => DataType::String,
      (None, DataType::Boolean, TokenType::Bang) | (None, DataType::String, TokenType::Bang) => {
        DataType::Boolean
      }
      _ => DataType::Pending,
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
      Expression::Variable(variable) => DataType::Variable(variable.name.span.literal.clone()),
      Expression::Assign(assign) => assign.data_type.clone(),
      Expression::Logical(logical) => logical.data_type.clone(),
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

  fn declaration(&mut self) -> Result<Statement, ()> {
    if self.match_token(&[TokenType::Let, TokenType::Const]) {
      return self.variable_declaration();
    }

    self.statement()
  }

  fn variable_declaration(&mut self) -> Result<Statement, ()> {
    let mut mutable = false;
    if self.peek().kind == TokenType::Mut {
      self.advance();
      mutable = true;
    }

    let name: Token = self.consume(TokenType::Identifier)?;

    let mut initializer: Option<Expression> = None;
    let type_annotation: DataType;

    match self.consume(TokenType::Colon) {
      Ok(_) => {
        let token = self.peek();

        let kind = DataType::from_token_type(token.kind.clone());

        if kind == DataType::None {
          self.diagnostics.report_expected_type_after_variable(&token);
          return Err(());
        }

        type_annotation = kind;
      }
      Err(error) => return Err(error),
    }

    self.advance();

    if self.match_token(&[TokenType::Equal]) {
      initializer = Some(self.expression()?);
    }

    match self.consume(TokenType::SemiColon) {
      Ok(_) => (),
      Err(_) => {
        let token = self.peek();

        self
          .diagnostics
          .report_unexpected_token(&TokenType::SemiColon, &token);

        return Err(());
      }
    };

    if let Some(ini) = initializer {
      Ok(Statement::Variable(Variable::new(
        Box::new(name),
        Some(Box::new(ini)),
        Box::new(type_annotation),
        mutable,
      )))
    } else {
      let token = self.peek();
      self.diagnostics.report_expected_expression(&token);

      return Err(());
    }
  }

  // statement -> expressionStatement;
  fn statement(&mut self) -> Result<Statement, ()> {
    self.expression_statement()
  }

  // expressionStatement -> expression ";";
  fn expression_statement(&mut self) -> Result<Statement, ()> {
    let expression = self.expression()?;

    self.consume(TokenType::SemiColon)?;

    Ok(Statement::Expression(ExpressionStatement::new(Box::new(
      expression,
    ))))
  }

  fn assignment(&mut self) -> Result<Expression, ()> {
    let expression = self.or_expression()?;

    if self.match_token(&[TokenType::Equal]) {
      let equals: Token = self.previous();
      let value = self.assignment()?;

      if let Expression::Variable(variable) = expression {
        return Ok(Expression::Assign(Assign::new(
          variable.name,
          Box::new(value),
          variable.data_type,
        )));
      }

      self
        .diagnostics
        .report_invalid_assignment_target(&equals.span);
      return Err(());
    }

    return Ok(expression);
  }

  fn or_expression(&mut self) -> Result<Expression, ()> {
    let mut expression = self.and_expression()?;

    while self.match_token(&[TokenType::Or]) {
      let operator: Token = self.previous();
      let right = self.and_expression()?;

      expression = Expression::Logical(Logical::new(
        Box::new(expression),
        operator,
        Box::new(right),
      ));
    }

    Ok(expression)
  }

  fn and_expression(&mut self) -> Result<Expression, ()> {
    let mut expression = self.equality()?;

    while self.match_token(&[TokenType::And]) {
      let operator: Token = self.previous();
      let right = self.equality()?;

      expression = Expression::Logical(Logical::new(
        Box::new(expression),
        operator,
        Box::new(right),
      ));
    }

    Ok(expression)
  }

  fn consume(&mut self, kind: TokenType) -> Result<Token, ()> {
    let token: Token = self.peek();
    if token.kind == kind {
      return Ok(self.advance());
    }

    match kind {
      TokenType::SemiColon => {
        self
          .diagnostics
          .report_unexpected_token(&TokenType::SemiColon, &token);
      }
      TokenType::Identifier => {
        self.diagnostics.report_expected_variable_name(&token);
      }
      _ => {
        self.diagnostics.report_unexpected_token(&kind, &token);
      }
    }

    Err(())
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
