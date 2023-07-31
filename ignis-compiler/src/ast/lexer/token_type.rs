/**
  TokenType
*/
#[derive(Debug)]
pub enum TokenType {
  // Single-character tokens
  Plus,       // +
  Minus,      // -
  Asterisk,   // *
  Slash,      // /
  LeftParen,  // (
  RightParen, // )
  LeftBrace,  // {
  RightBrace, // }
  LeftBrack,  // [
  RightBrack, // ]
  Colon,      // :
  Pipe,       // |
  Ampersand,  // &

  // One or two character tokens
  Equal,        // =
  EqualEqual,   // ==
  Bang,         // !
  BangEqual,    // !=
  Greater,      // >
  GreaterEqual, // >=
  Less,         // <
  LessEqual,    // <=
  Or,           // ||
  And,          // &&
  Arrow,        // =>

  // Separator
  Comma,     // ,
  SemiColon, // ;
  Dot,       // .

  // Literals
  Number(f32),
  Int(i32),    // 1-10
  Double(f32), // 1.3
  Char(char),  // a-z A-Z 0-9
  String(String),

  // Keywords
  Class,
  Super,
  Static,
  Final,
  ReadOnly,
  Public,
  Private,
  Else,
  False,
  True,
  Function,
  For,
  If,
  Null,
  Return,
  This,
  Let,
  Const,
  Mut,
  As,
  While,
  Break,
  Enum,
  Export,
  Import,
  From,
  Extends,
  Implements,
  Interface,

  Bad,
  Identifier,
  Eof,
}
