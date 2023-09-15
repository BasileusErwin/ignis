use ast::statement::{class::Class, variable::VariableMetadata};
use enums::{data_type::DataType, token_type::TokenType};
use lexer::text_span::TextSpan;

use {
  lexer::token::Token,
  ast::expression::{
    Expression, binary::Binary, unary::Unary, literal::Literal, grouping::Grouping,
    logical::Logical, assign::Assign, variable::VariableExpression, ternary, call::Call,
  },
  enums::{literal_value::LiteralValue, function_kind::FunctionKind},
  ast::statement::{
    Statement,
    variable::Variable,
    expression::ExpressionStatement,
    if_statement::IfStatement,
    block::Block,
    while_statement::WhileStatement,
    function::{FunctionStatement, FunctionParameter},
    return_statement::Return,
  },
};

pub enum ParserDiagnosticError {
  ExpectedExpression(Token),
  ExpectedToken(TokenType, Token),
  ExpectedVariableName(Token),
  ExpectedReturnTypeAfterFunction(Token),
  ExpectedAfterExpression(TokenType, Token, Token),
  ExpectedExpressionAfter(Token),
  UnexpectedToken(TokenType, Token),
  InvalidAssignmentTarget(TextSpan),
  ExpectedTypeAfterVariable(Token),
  InvalidNumberOfArguments(usize, usize, Token),
}

type ParserResult<T> = Result<T, ParserDiagnosticError>;

pub struct Parser {
  pub tokens: Vec<Token>,
  current: usize,
  pub diagnostics: Vec<ParserDiagnosticError>,
}

impl Parser {
  pub fn new(tokens: Vec<Token>) -> Self {
    Self {
      tokens,
      current: 0,
      diagnostics: Vec::new(),
    }
  }

  fn report_error(&mut self, error: ParserDiagnosticError) {
    self.diagnostics.push(error);
  }

  pub fn parse(&mut self) -> Result<Vec<Statement>, ()> {
    let mut statements: Vec<Statement> = vec![];
    while !self.is_at_end() {
      match self.declaration() {
        Ok(result) => statements.push(result),
        Err(error) => {
          self.report_error(error);
        }
      };
    }

    if self.diagnostics.len() > 0 {
      return Err(());
    }

    Ok(statements)
  }

  fn expression(&mut self) -> ParserResult<Expression> {
    self.assignment()
  }

  // equelity -> comparison (("!=" | "==") comparison)*;
  fn equality(&mut self) -> ParserResult<Expression> {
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
  fn comparison(&mut self) -> ParserResult<Expression> {
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
  fn term(&mut self) -> ParserResult<Expression> {
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
  fn factor(&mut self) -> ParserResult<Expression> {
    let mut expression: Expression = self.unary()?;

    while self.match_token(&[TokenType::Slash, TokenType::Asterisk, TokenType::Mod]) {
      let operator: Token = self.previous();
      let right: Expression = self.unary()?;

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

  // unary -> ("!" | "-") unary | call;
  fn unary(&mut self) -> ParserResult<Expression> {
    if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
      let operator = self.previous();
      let right: Expression = self.unary()?;

      let right_type = self.get_expression_type(&right);
      let operator_kind = operator.kind.clone();

      return Ok(Expression::Unary(Unary::new(
        operator,
        Box::new(right),
        self.get_data_type_by_operator(None, right_type, operator_kind),
      )));
    }

    self.call()
  }

  fn call(&mut self) -> ParserResult<Expression> {
    let mut expression: Expression = self.primary()?;

    loop {
      if self.match_token(&[TokenType::LeftParen]) {
        expression = self.finish_call(expression)?;
      } else {
        break;
      }
    }

    Ok(expression)
  }

  fn primary(&mut self) -> ParserResult<Expression> {
    let token = self.peek();

    match token.kind {
      TokenType::True
      | TokenType::False
      | TokenType::Null
      | TokenType::Int
      | TokenType::Double
      | TokenType::String => {
        self.advance();
        Ok(Expression::Literal(Literal::new(
          LiteralValue::from_token_type(token.kind.clone(), token.span.literal.clone()),
        )))
      }
      TokenType::LeftParen => {
        self.advance();
        let expression = self.expression()?;
        self.consume(TokenType::RightParen)?;

        return Ok(Expression::Grouping(Grouping::new(Box::new(expression))));
      }
      TokenType::Identifier => {
        self.advance();
        let kind = token.kind.clone();
        return Ok(Expression::Variable(VariableExpression::new(
          token,
          DataType::from_token_type(kind),
        )));
      }
      _ => Err(ParserDiagnosticError::ExpectedExpression(token.clone())),
    }
  }

  fn finish_call(&mut self, callee: Expression) -> ParserResult<Expression> {
    let mut arguments: Vec<Expression> = Vec::new();

    if !self.check(TokenType::RightParen) {
      loop {
        if arguments.len() >= 255 {
          let token = &self.peek();

          return Err(ParserDiagnosticError::InvalidNumberOfArguments(
            255,
            arguments.len(),
            token.clone(),
          ));
        }

        arguments.push(self.expression()?);

        if !self.match_token(&[TokenType::Comma]) {
          break;
        }
      }
    }

    let token = self.consume(TokenType::RightParen)?;

    Ok(Expression::Call(Call::new(
      Box::new(callee),
      token,
      arguments,
      DataType::Pending,
    )))
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
      Expression::Ternary(ternary) => ternary.data_type.clone(),
      Expression::Call(call) => call.return_type.clone(),
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

  fn declaration(&mut self) -> ParserResult<Statement> {
    if self.match_token(&[TokenType::Let, TokenType::Const]) {
      return self.variable_declaration();
    }

    if self.match_token(&[TokenType::Class]) {
      return self.class_declaration();
    }

    if self.match_token(&[TokenType::Function]) {
      return self.function(FunctionKind::Function);
    }

    if self.match_token(&[TokenType::Return]) {
      return self.return_statement();
    }

    if self.match_token(&[TokenType::While]) {
      return self.while_statement();
    }

    match self.statement() {
      Ok(statement) => Ok(statement),
      Err(error) => {
        self.synchronize();
        return Err(error);
      }
    }
  }

  fn return_statement(&mut self) -> Result<Statement, ParserDiagnosticError> {
    let keyword = self.previous();

    if self.check(TokenType::SemiColon) {
      self.advance();
      return Ok(Statement::Return(Return::new(None, Box::new(keyword))));
    }

    let value = self.expression()?;

    self.consume(TokenType::SemiColon)?;

    let result = Return::new(Some(Box::new(value)), Box::new(keyword));
    Ok(Statement::Return(result))
  }

  fn function(&mut self, kind: FunctionKind) -> ParserResult<Statement> {
    match kind {
      FunctionKind::Function => {
        let name: Token = self.consume(TokenType::Identifier)?;

        self.consume(TokenType::LeftParen)?;

        let mut parameters: Vec<FunctionParameter> = Vec::new();

        if !self.check(TokenType::RightParen) {
          loop {
            if parameters.len() >= 255 {
              return Err(ParserDiagnosticError::InvalidNumberOfArguments(
                255,
                parameters.len(),
                name.clone(),
              ));
            }

            let is_mut: bool = if self.peek().kind == TokenType::Mut {
              self.advance();
              true
            } else {
              false
            };

            let param = self.consume(TokenType::Identifier)?;

            self.consume(TokenType::Colon)?;
            let token = self.advance();

            parameters.push(FunctionParameter::new(
              param,
              DataType::from_token_type(token.kind),
              is_mut,
            ));

            if !self.match_token(&[TokenType::Comma]) {
              break;
            }
          }
        }

        self.consume(TokenType::RightParen)?;

        self.consume(TokenType::Colon)?;

        let mut return_type: Option<DataType> = None;
        if self.match_token(&[
          TokenType::Void,
          TokenType::IntType,
          TokenType::DoubleType,
          TokenType::StringType,
          TokenType::BooleanType,
          TokenType::CharType,
        ]) {
          return_type = Some(DataType::from_token_type(self.previous().kind));
        } else {
          let token = &self.peek();

          return Err(ParserDiagnosticError::ExpectedReturnTypeAfterFunction(
            token.clone(),
          ));
        }

        self.consume(TokenType::LeftBrace)?;

        let mut body: Vec<Statement> = Vec::new();

        body.push(self.block()?);

        Ok(Statement::FunctionStatement(FunctionStatement::new(
          name,
          parameters,
          body,
          return_type,
        )))
      }
      FunctionKind::Method => todo!(),
      FunctionKind::Initializer => todo!(),
      FunctionKind::Lambda => todo!(),
    }
  }

  fn block(&mut self) -> ParserResult<Statement> {
    let mut statements: Vec<Statement> = Vec::new();

    while !self.check(TokenType::RightBrace) && !self.is_at_end() {
      statements.push(self.declaration()?);
    }

    self.consume(TokenType::RightBrace)?;

    Ok(Statement::Block(Block::new(statements)))
  }

  fn variable_declaration(&mut self) -> ParserResult<Statement> {
    let mutable: bool = if self.peek().kind == TokenType::Mut {
      self.advance();
      true
    } else {
      false
    };

    let name: Token = self.consume(TokenType::Identifier)?;

    let mut initializer: Option<Expression> = None;

    self.consume(TokenType::Colon)?;

    let token = self.peek();

    let type_annotation = DataType::from_token_type(token.kind.clone());

    if type_annotation == DataType::None {
      return Err(ParserDiagnosticError::ExpectedTypeAfterVariable(token));
    }

    self.advance();

    if self.match_token(&[TokenType::Equal]) {
      initializer = Some(self.expression()?);
    }

    self.consume(TokenType::SemiColon)?;

    if let Some(ini) = initializer {
      Ok(Statement::Variable(Variable::new(
        Box::new(name),
        Some(Box::new(ini)),
        type_annotation,
        VariableMetadata::new(mutable, false, false, false, false),
      )))
    } else {
      let token = self.peek();
      Err(ParserDiagnosticError::ExpectedExpression(token.clone()))
    }
  }

  // statement -> expressionStatement | ifStatement;
  fn statement(&mut self) -> ParserResult<Statement> {
    if self.match_token(&[TokenType::LeftBrace]) {
      return self.block();
    }

    if self.match_token(&[TokenType::If]) {
      return self.if_statement();
    }

    self.expression_statement()
  }

  // expressionStatement -> expression ";";
  fn expression_statement(&mut self) -> ParserResult<Statement> {
    let expression = self.expression()?;

    self.consume(TokenType::SemiColon)?;

    Ok(Statement::Expression(ExpressionStatement::new(Box::new(
      expression,
    ))))
  }

  fn assignment(&mut self) -> ParserResult<Expression> {
    let mut expression: Expression = self.ternary()?;

    if self.match_token(&[TokenType::Equal]) {
      let equals: Token = self.previous();
      let value: Expression = self.assignment()?;

      if let Expression::Variable(variable) = expression {
        expression = Expression::Assign(Assign::new(
          variable.name,
          Box::new(value),
          variable.data_type,
        ));
      } else {
        return Err(ParserDiagnosticError::InvalidAssignmentTarget(
          equals.span.clone(),
        ));
      }
    }

    return Ok(expression);
  }

  fn ternary(&mut self) -> ParserResult<Expression> {
    let mut children: Vec<Expression> = Vec::new();

    children.push(self.or_expression()?);

    while self.match_token(&[TokenType::QuestionMark]) {
      children.push(self.expression()?);

      self.consume(TokenType::Colon)?;

      children.push(self.expression()?);
    }

    if children.len() == 1 {
      return Ok(children.pop().unwrap());
    }

    let else_branch = children.pop().unwrap();
    let then_branch = children.pop().unwrap();
    let condition = children.pop().unwrap();

    let mut expression: Expression = Expression::Ternary(ternary::Ternary::new(
      Box::new(condition),
      Box::new(then_branch),
      Box::new(else_branch),
      DataType::Pending,
    ));

    while !children.is_empty() {
      expression = Expression::Ternary(ternary::Ternary::new(
        Box::new(children.pop().unwrap()),
        Box::new(expression),
        Box::new(children.pop().unwrap()),
        DataType::Pending,
      ));
    }

    Ok(expression)
  }

  fn or_expression(&mut self) -> ParserResult<Expression> {
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

  fn and_expression(&mut self) -> ParserResult<Expression> {
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

  fn while_statement(&mut self) -> ParserResult<Statement> {
    self.consume(TokenType::LeftParen)?;

    let condition: Expression = self.expression()?;

    self.consume(TokenType::RightParen)?;

    let body: Statement = self.statement()?;

    Ok(Statement::WhileStatement(WhileStatement::new(
      Box::new(condition),
      Box::new(body),
    )))
  }

  fn if_statement(&mut self) -> ParserResult<Statement> {
    self.consume(TokenType::LeftParen)?;

    let condition: Expression = self.expression()?;

    self.consume(TokenType::RightParen)?;

    let then_branch: Statement = self.statement()?;

    let else_branch: Option<Statement> = if self.match_token(&[TokenType::Else]) {
      Some(self.statement()?)
    } else {
      None
    };

    Ok(Statement::IfStatement(IfStatement::new(
      Box::new(condition),
      Box::new(then_branch),
      else_branch.map(|s| Box::new(s)),
    )))
  }

  fn consume(&mut self, kind: TokenType) -> ParserResult<Token> {
    let token: Token = self.peek();
    if token.kind == kind {
      return Ok(self.advance());
    }

    let error = match kind {
      TokenType::SemiColon => {
        ParserDiagnosticError::UnexpectedToken(TokenType::SemiColon, token.clone())
      }
      TokenType::Colon => ParserDiagnosticError::UnexpectedToken(TokenType::Colon, token.clone()),
      TokenType::Identifier => ParserDiagnosticError::ExpectedVariableName(token.clone()),
      TokenType::QuestionMark => {
        ParserDiagnosticError::ExpectedToken(TokenType::QuestionMark, token.clone())
      }
      TokenType::LeftParen | TokenType::RightParen => {
        let expression = self.previous();

        ParserDiagnosticError::ExpectedAfterExpression(
          kind.clone(),
          expression.clone(),
          token.clone(),
        )
      }
      _ => ParserDiagnosticError::ExpectedToken(kind.clone(), token.clone()),
    };

    return Err(error);
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

  fn class_declaration(&mut self) -> Result<Statement, ParserDiagnosticError> {
    let name: Token = self.consume(TokenType::Identifier)?;

    let mut methods: Vec<FunctionStatement> = Vec::new();

    self.consume(TokenType::LeftBrace)?;

    while !self.check(TokenType::RightBrace) && !self.is_at_end() {
      let method = match self.function(FunctionKind::Method)? {
        Statement::FunctionStatement(function) => methods.push(function),
        _ => (),
      };
    }

    self.consume(TokenType::RightBrace)?;

    Ok(Statement::Class(Class::new(name, methods)))
  }
}
