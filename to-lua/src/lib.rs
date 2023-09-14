use std::ops::IndexMut;

use analyzer::{
  ir::{
    instruction::{IRInstruction, function::IRFunction, call::IRCall},
    instruction_type::IRInstructionType,
  },
  analyzer_value::AnalyzerValue,
};

fn transpile_opeartor_to_lua(operator: &IRInstructionType) -> String {
  match operator {
    IRInstructionType::Add => "+",
    IRInstructionType::Sub => "-",
    IRInstructionType::Mul => "*",
    IRInstructionType::Div => "/",
    IRInstructionType::GreaterEqual => ">=",
    IRInstructionType::Greater => ">",
    IRInstructionType::LessEqual => "<=",
    IRInstructionType::Less => "<",
    IRInstructionType::Equal => "==",
    IRInstructionType::NotEqual => "~=",
    IRInstructionType::And => "and",
    IRInstructionType::Or => "or",
    IRInstructionType::Not => "not",
    IRInstructionType::Assign => "=",
    IRInstructionType::AssignAdd => "+=",
    IRInstructionType::AssignSub => "-=",
    IRInstructionType::Mod => "%",
  }
  .to_string()
}

fn transpile_function_to_lua(func: &IRFunction, indent_level: usize) -> String {
  let mut code = String::new();

  if let Some(body) = &func.body {
    let parameters = func
      .parameters
      .iter()
      .map(|x| x.name.clone())
      .collect::<Vec<String>>()
      .join(", ");

    code.push_str(&format!(
      "{}local {} = function({})\n",
      " ".repeat(indent_level),
      func.name,
      parameters
    ));

    for instr in &body.instructions {
      code.push_str(&transpile_ir_to_lua(instr, indent_level + 2));
    }

    code.push_str(format!("{}end\n", " ".repeat(indent_level)).as_str());
  }

  code
}

fn transpile_call_to_lua(call: &IRCall, indent_level: usize) -> String {
  let mut code: String = String::new();

  let name = match call.name.as_str() {
    "println" => "print".to_string(),
    _ => call.name.clone(),
  };

  code.push_str(&format!(
    "{}{}({})",
    " ".repeat(indent_level),
    name,
    call
      .arguments
      .iter()
      .map(|f| transpile_ir_to_lua(&f, indent_level))
      .collect::<Vec<String>>()
      .join(", ")
  ));

  code.push_str("\n");

  code
}

pub fn transpile_ir_to_lua(instruction: &IRInstruction, indent_level: usize) -> String {
  match instruction {
    IRInstruction::Literal(literal) => match &literal.value {
      AnalyzerValue::Int(num) => num.to_string(),
      AnalyzerValue::String(s) => format!("\"{}\"", s),
      AnalyzerValue::Double(num) => num.to_string(),
      AnalyzerValue::Boolean(boolean) => boolean.to_string(),
      AnalyzerValue::Return(r) => r.to_string(),
      AnalyzerValue::Function(f) => f.name.span.literal.clone(),
      AnalyzerValue::Null | AnalyzerValue::None => "nil".to_string(),
    },
    IRInstruction::Binary(binary) => {
      let left = transpile_ir_to_lua(&binary.left, indent_level);
      let right = transpile_ir_to_lua(&binary.right, indent_level);
      let op = transpile_opeartor_to_lua(&binary.instruction_type);

      format!("{} {} {}", left, op, right)
    }
    IRInstruction::Block(block) => {
      let mut code = String::new();
      for var in &block.scopes_variables {
        let var_value = if let Some(value) = &var.value {
          transpile_ir_to_lua(value, indent_level)
        } else {
          "".to_string()
        };

        code.push_str(&format!(
          "{}local {} = {}\n",
          " ".repeat(indent_level),
          var.name,
          var_value
        ));
      }

      for instr in &block.instructions {
        match instr {
          IRInstruction::Variable(_) => continue,
          _ => (),
        };

        code.push_str(&transpile_ir_to_lua(instr, indent_level));
      }
      code
    }
    IRInstruction::Function(func) => transpile_function_to_lua(func, indent_level),
    IRInstruction::Unary(unary) => {
      let value = transpile_ir_to_lua(&unary.right, indent_level);
      let op = transpile_opeartor_to_lua(&unary.instruction_type);

      format!("{} {}", op, value)
    }
    IRInstruction::Variable(var) => {
      let var_value = if let Some(value) = &var.value {
        transpile_ir_to_lua(value, 0)
      } else {
        "".to_string()
      };

      if var.metadata.is_declaration {
        format!(
          "{}local {} = {}\n",
          " ".repeat(indent_level),
          var.name,
          var_value
        )
      } else {
        format!("{}", var.name)
      }
    }
    IRInstruction::Logical(logical) => {
      let left = transpile_ir_to_lua(&logical.left, indent_level);
      let right = transpile_ir_to_lua(&logical.right, indent_level);
      let op = transpile_opeartor_to_lua(&logical.instruction_type);

      format!("{} {} {}", left, op, right)
    }
    IRInstruction::If(if_instruction) => {
      let mut code = String::new();

      code.push_str(&format!(
        "{}if {} then\n",
        " ".repeat(indent_level),
        transpile_ir_to_lua(&if_instruction.condition, indent_level)
      ));

      code.push_str(&transpile_ir_to_lua(
        &if_instruction.then_branch,
        indent_level,
      ));

      if let Some(else_branch) = &if_instruction.else_branch {
        code.push_str(format!("{}else\n", " ".repeat(indent_level)).as_str());
        code.push_str(&transpile_ir_to_lua(else_branch, indent_level));
      }

      code.push_str(format!("{} end\n", " ".repeat(indent_level)).as_str());

      code
    }
    IRInstruction::While(ir_while) => {
      let mut code = String::new();
      let condition = transpile_ir_to_lua(&ir_while.condition, indent_level);

      code.push_str(&format!(
        "{}while {} do\n",
        " ".repeat(indent_level),
        condition
      ));

      code.push_str(&transpile_ir_to_lua(&ir_while.body, indent_level + 2));

      code.push_str(format!("{}end\n", " ".repeat(indent_level)).as_str());

      code
    }
    IRInstruction::Call(call) => transpile_call_to_lua(call, indent_level),
    IRInstruction::Return(r) => {
      let value = transpile_ir_to_lua(&r.value, indent_level);
      format!("{}return {}\n", " ".repeat(indent_level), value)
    }
    IRInstruction::Assign(assign) => {
      let value = transpile_ir_to_lua(&assign.value, 0);
      format!("{}{} = {}\n", " ".repeat(indent_level), assign.name, value)
    }
    IRInstruction::Class(_) => todo!(),
    IRInstruction::Ternary(ternary) => {
      let condition = transpile_ir_to_lua(&ternary.condition, indent_level);

      let then_branch = transpile_ir_to_lua(&ternary.then_branch, indent_level);
      let else_branch = transpile_ir_to_lua(&ternary.else_branch, indent_level);

      format!("{} and {} or {}", condition, then_branch, else_branch)
    }
  }
}
