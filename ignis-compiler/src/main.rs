mod ast;
mod diagnostic;

use std::{
  io::{self, Write, BufRead},
  env,
  process::exit,
  fs,
};

use ast::{
  parser::{Parser, ParserResult},
  lexer::Lexer,
  evaluator::Evaluator,
  Ast,
};
use diagnostic::DiagnosticList;

fn display_diagnostic(diagnostics: &DiagnosticList, relp: bool) {
  for diagnostic in diagnostics.diagnostics.iter() {
    println!("{}: {}", diagnostic.code, diagnostic.hint.as_ref().unwrap());
    if relp {
      println!("{} | {}", diagnostic.span.line, diagnostic.span.literal);
    }
  }
}

fn run_file(path: &str) -> Result<(), ()> {
  let mut evaluator = Evaluator::new();

  match fs::read_to_string(path) {
    Ok(content) => run(content, &mut evaluator, true),
    Err(e) => {
      println!("{:?}", e);
      Err(())
    }
  }
}

fn run(source: String, evaluator: &mut Evaluator, relp: bool) -> Result<(), ()> {
  let mut lexer: Lexer<'_> = Lexer::new(&source);
  lexer.scan_tokens();

  // for token in &lexer.tokens {
  //   println!("{:?}", token);
  // }

  let mut parser = Parser::new(lexer.tokens);
  let expressions = parser.parse();
  let mut ast = Ast::new();
  
  for expression in expressions {
    match expression {
      ParserResult::Statement(expr) => ast.add(expr),
      _ => {
        display_diagnostic(&parser.diagnostics, relp);
        return Err(());
      }
    }
  }

  ast.visit(evaluator);
  
  let mut diagnostics = evaluator.diagnostics.borrow_mut();

  if diagnostics.diagnostics.len() > 0 {
    display_diagnostic(&diagnostics, relp);
  }
  
  diagnostics.clean_diagnostic();

  Ok(())
}

fn run_prompt() -> Result<(), String> {
  let mut evaluator = Evaluator::new();

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

    if buffer.trim() == String::from("exit") {
      println!("Bye!");
      exit(0);
    }
    
    if buffer.contains("load") == true {
      let path = buffer.split("load").collect::<Vec<&str>>()[1].trim().to_string();
      match run_file(&path) {
        Ok(_) => (),
        Err(_) => println!("Could not import file"),
      }
      continue;
    }

    match run(buffer, &mut evaluator, false) {
      Ok(_) => (),
      Err(()) => (),
    }
  }
}

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() == 2 {
    run_file(&args[1]);
  } else {
    match run_prompt() {
      Ok(_) => exit(0),
      Err(msg) => {
        println!("ERROR\n{}", msg);
        exit(1);
      }
    }
  }
}
