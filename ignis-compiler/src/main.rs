mod ast;

use std::{
  io::{self, Write, BufRead},
  env,
  process::exit,
};

use ast::{parser::Parser, lexer::Lexer};

fn run(source: String) -> Result<(), String> {
  if source.trim() == String::from("exit") {
    println!("Bye!");
    exit(0);
  }

  let mut lexer: Lexer<'_> = Lexer::new(&source);
  lexer.scan_tokens();

  let mut parser = Parser::new(lexer.tokens);
  let expressions = parser.parse();
  
  match expressions {
      Ok(result) => println!("{:?}", result.to_string()),
      Err(error) => println!("{:?}", error)
  };

  Ok(())
}

fn run_prompt() -> Result<(), String> {
  loop {
    print!("(ignis) > ");

    match io::stdout().flush() {
      Ok(_) => (),
      Err(_) => return Err("Could not flush stdout".to_string()),
    }
    let mut buffer = String::new();
    let mut handler = io::stdin().lock();

    match handler.read_line(&mut buffer) {
      Ok(n) => {
        if n == 0 {
          println!("");
          return Ok(());
        } else if n == 1 {
          continue;
        }
      }
      Err(_) => return Err("Clound not read line".to_string()),
    }

    match run(buffer) {
      Ok(_) => (),
      Err(message) => println!("{}", message),
    }
  }
}

fn main() {
  match run_prompt() {
    Ok(_) => exit(0),
    Err(msg) => {
      println!("ERROR\n{}", msg);
      exit(1);
    }
  }
}
