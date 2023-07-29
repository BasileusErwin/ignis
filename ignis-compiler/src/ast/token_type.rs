
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
  Colon, // :

  // One or two character tokens
  Equal, // =
  EqualEqual, // ==
  Bang,          // !
  BangEqual,    // !=
  Greater,       // >
  GreaterEqual, // >=
  Less,          // <
  LessEqual,    // <=
  Or,            // ||
  And,           // &&

  // Separator
  Comma, // ,
  SemiColon, // ;
  Dot, // .

  // Literals
  Number(f32),
  Int(i32),    // 1-10
  Double(f32), // 1.3
  Char(char),  // c

  // Keywords
  Class,
  Super,
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
  While,

  Bad,
  Identifier,
  Eof,
}
