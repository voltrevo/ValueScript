use std::process::exit;
use std::{path::Path, sync::Arc};
use std::collections::HashSet;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use swc_ecma_ast::{EsVersion};
use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
};
use swc_ecma_parser::{TsConfig, Syntax};

pub fn command(args: &Vec<String>) {
  if args.len() != 3 {
    println!("ERROR: Unrecognized command\n");
    show_help();
    exit(1);
  }

  let program = parse(&args[2]);
  let assembly = compile(&program);

  let mut file = File::create("out.vsm").expect("Couldn't create out.vsm");

  for line in assembly {
    file.write_all(line.as_bytes()).expect("Failed to write line");
    file.write_all(b"\n").expect("Failed to write line");
  }
}

fn show_help() {
  println!("vstc compile");
  println!("");
  println!("Compile ValueScript");
  println!("");
  println!("USAGE:");
  println!("    vstc compile <entry point>");
}

pub fn parse(file_path: &String) -> swc_ecma_ast::Program {
  let source_map = Arc::<SourceMap>::default();

  let handler = Handler::with_tty_emitter(
      ColorConfig::Auto,
      true,
      false,
      Some(source_map.clone()),
  );

  let swc_compiler = swc::Compiler::new(source_map.clone());

  let file = source_map
      .load_file(Path::new(&file_path))
      .expect("failed to load file");

  let result = swc_compiler.parse_js(
      file,
      &handler,
      EsVersion::Es2022,
      Syntax::Typescript(TsConfig::default()),
      swc::config::IsModule::Bool(true),
      None,
  );

  return result.expect("Parse failed");
}

pub fn compile(program: &swc_ecma_ast::Program) -> Vec<String> {
  let mut compiler = Compiler::default();
  compiler.compile_program(&program);
  
  let mut lines = Vec::<String>::new();

  for def in compiler.definitions {
    for line in def {
      lines.push(line);
    }
  }

  return lines;
}

#[derive(Default)]
struct Compiler {
  definition_allocator: NameAllocator,
  definitions: Vec<Vec<String>>,
}

impl Compiler {
  fn compile_program(&mut self, program: &swc_ecma_ast::Program) {
    use swc_ecma_ast::Program::*;

    match program {
      Module(module) => self.compile_module(module),
      Script(_) => std::panic!("Not supported: script"),
    }
  }

  fn compile_module(&mut self, module: &swc_ecma_ast::Module) {
    if module.body.len() != 1 {
      std::panic!("Not implemented: modules that aren't just export default");
    }

    self.compile_module_item(&module.body[0]);
  }

  fn compile_module_item(&mut self, module_item: &swc_ecma_ast::ModuleItem) {
    use swc_ecma_ast::ModuleItem::*;

    match module_item {
      ModuleDecl(module_decl) => self.compile_module_decl(module_decl),
      Stmt(_) => std::panic!("Not supported: module statement"),
    }
  }

  fn compile_module_decl(&mut self, module_decl: &swc_ecma_ast::ModuleDecl) {
    use swc_ecma_ast::ModuleDecl::*;

    match module_decl {
      ExportDefaultDecl(edd) => self.compile_export_default_decl(edd),
      _ => std::panic!("Not implemented: non-default module declaration"),
    }
  }

  fn compile_export_default_decl(&mut self, edd: &swc_ecma_ast::ExportDefaultDecl) {
    use swc_ecma_ast::DefaultDecl::*;

    match &edd.decl {
      Fn(fn_) => self.compile_main_fn(fn_),
      _ => std::panic!("Not implemented: Non-function default export"),
    }
  }

  fn compile_main_fn(&mut self, main_fn: &swc_ecma_ast::FnExpr) {
    let mut definition: Vec<String> = Vec::new();
    
    let fn_defn_name = self.definition_allocator.allocate(&match &main_fn.ident {
      Some(ident) => ident.sym.to_string(),
      None => "main".to_string(),
    });

    let mut name_reg_map = HashMap::<String, String>::new();
    let mut reg_allocator = NameAllocator::default();
    reg_allocator.allocate(&"return".to_string());
    reg_allocator.allocate(&"this".to_string());
    let mut param_registers = Vec::<String>::new();

    for p in &main_fn.function.params {
      match &p.pat {
        swc_ecma_ast::Pat::Ident(binding_ident) => {
          let param_name = binding_ident.id.sym.to_string();
          let reg = reg_allocator.allocate(&param_name);
          param_registers.push(reg.clone());
          name_reg_map.insert(param_name, reg);
        },
        _ => std::panic!("Not implemented: parameter destructuring"),
      }
    }

    let mut heading = "@".to_string();
    heading += &fn_defn_name;
    heading += " = function(";

    for i in 0..param_registers.len() {
      heading += "%";
      heading += &param_registers[i];

      if i != param_registers.len() - 1 {
        heading += ", ";
      }
    }

    heading += ") {";

    definition.push(heading);

    let statements = match &main_fn.function.body {
      Some(body) => &body.stmts,
      None => std::panic!(""),
    };

    for i in 0..statements.len() {
      let statement = &statements[i];
      let last = i == statements.len() - 1;

      use swc_ecma_ast::Stmt::*;

      match statement {
        Return(ret_stmt) => match &ret_stmt.arg {
          None => {
            definition.push("  end".to_string());
          },
          Some(expr) => {
            compile_expression(
              &mut definition,
              &mut name_reg_map,
              &mut reg_allocator,
              expr,
              None,
              Some("return".to_string()),
            );

            if !last {
              definition.push("  end".to_string());
            }
          },
        },
        _ => std::panic!("Not implemented"),
      }
    }

    definition.push("}".to_string());

    self.definitions.push(definition);
  }
}

#[derive(Default)]
struct NameAllocator {
  used_names: HashSet<String>,
}

impl NameAllocator {
  fn allocate(&mut self, based_on_name: &String) -> String {
    if !self.used_names.contains(based_on_name) {
      self.used_names.insert(based_on_name.clone());
      return based_on_name.clone();
    }

    return self.allocate_numbered(&(based_on_name.clone() + "_"));
  }

  fn allocate_numbered(&mut self, prefix: &String) -> String {
    let mut i = 0_u64;

    loop {
      let candidate = prefix.clone() + &i.to_string();

      if !self.used_names.contains(&candidate) {
        self.used_names.insert(candidate.clone());
        return candidate;
      }

      i += 1;
    }
  }

  fn release(&mut self, name: &String) {
    self.used_names.remove(name);
  }
}

struct CompiledExpression {
  value_assembly: String,
  used_available_register: bool,
  nested_registers: Vec<String>,
}

fn compile_expression(
  definition: &mut Vec<String>,
  name_reg_map: &mut HashMap<String, String>,
  reg_allocator: &mut NameAllocator,
  expr: &swc_ecma_ast::Expr,
  mut available_register: Option<String>,
  target_register: Option<String>,
) -> CompiledExpression {
  use swc_ecma_ast::Expr::*;

  available_register = available_register.or(target_register.clone());

  let mut nested_registers = Vec::<String>::new();

  match expr {
    This(_) => std::panic!("Not implemented: This expression"),
    Array(_) => std::panic!("Not implemented: Array expression"),
    Object(_) => std::panic!("Not implemented: Object expression"),
    Fn(_) => std::panic!("Not implemented: Fn expression"),
    Unary(_) => std::panic!("Not implemented: Unary expression"),
    Update(_) => std::panic!("Not implemented: Update expression"),
    Bin(bin) => {
      // If the available register is used in subexpressions it'll still be
      // available afterwards. We do need to make sure the same register isn't
      // used for both subexpressions though.
      let mut sub_available_register = available_register.clone();

      let left = compile_expression(
        definition,
        name_reg_map,
        reg_allocator,
        &bin.left,
        sub_available_register.clone(),
        None
      );

      if left.used_available_register {
        sub_available_register = None;
      }

      let right = compile_expression(
        definition,
        name_reg_map,
        reg_allocator,
        &bin.right,
        sub_available_register.clone(),
        None,
      );

      let mut instr = "  ".to_string();
      instr += get_binary_op_str(bin.op);
      instr += " ";
      instr += &left.value_assembly;
      instr += " ";
      instr += &right.value_assembly;

      for used_reg in left.nested_registers {
        reg_allocator.release(&used_reg);
      }

      for used_reg in right.nested_registers {
        reg_allocator.release(&used_reg);
      }

      let target: String = match &target_register {
        None => match available_register {
          None => {
            let res = reg_allocator.allocate_numbered(&"_tmp".to_string());
            nested_registers.push(res.clone());
            res
          },
          Some(a) => {
            let res = a.clone();
            available_register = None;
            res
          },
        },
        Some(t) => t.clone(),
      };

      instr += " %";
      instr += &target;

      definition.push(instr);

      return CompiledExpression {
        value_assembly: std::format!("%{}", target),

        // Note: Possibly misleading here - we'll return true here even if we
        // weren't given an available register (FIXME?)
        used_available_register: available_register.is_none(),

        nested_registers: nested_registers,
      };
    },
    Assign(_) => std::panic!("Not implemented: Assign expression"),
    Member(_) => std::panic!("Not implemented: Member expression"),
    SuperProp(_) => std::panic!("Not implemented: SuperProp expression"),
    Cond(_) => std::panic!("Not implemented: Cond expression"),
    Call(_) => std::panic!("Not implemented: Call expression"),
    New(_) => std::panic!("Not implemented: New expression"),
    Seq(_) => std::panic!("Not implemented: Seq expression"),
    Ident(_) => std::panic!("Not implemented: Ident expression"),
    Lit(lit) => match target_register {
      None => {
        return CompiledExpression {
          value_assembly: compile_literal(lit),
          used_available_register: false,
          nested_registers: nested_registers,
        };
      },
      Some(t) => {
        let mut instr = "  mov ".to_string();
        instr += &compile_literal(lit);
        instr += " %";
        instr += &t;
        definition.push(instr);

        return CompiledExpression {
          value_assembly: std::format!("%{}", t),
          used_available_register: false,
          nested_registers: nested_registers,
        };
      },
    },
    Tpl(_) => std::panic!("Not implemented: Tpl expression"),
    TaggedTpl(_) => std::panic!("Not implemented: TaggedTpl expression"),
    Arrow(_) => std::panic!("Not implemented: Arrow expression"),
    Class(_) => std::panic!("Not implemented: Class expression"),
    Yield(_) => std::panic!("Not implemented: Yield expression"),
    MetaProp(_) => std::panic!("Not implemented: MetaProp expression"),
    Await(_) => std::panic!("Not implemented: Await expression"),
    Paren(_) => std::panic!("Not implemented: Paren expression"),
    JSXMember(_) => std::panic!("Not implemented: JSXMember expression"),
    JSXNamespacedName(_) => std::panic!("Not implemented: JSXNamespacedName expression"),
    JSXEmpty(_) => std::panic!("Not implemented: JSXEmpty expression"),
    JSXElement(_) => std::panic!("Not implemented: JSXElement expression"),
    JSXFragment(_) => std::panic!("Not implemented: JSXFragment expression"),
    TsTypeAssertion(_) => std::panic!("Not implemented: TsTypeAssertion expression"),
    TsConstAssertion(_) => std::panic!("Not implemented: TsConstAssertion expression"),
    TsNonNull(_) => std::panic!("Not implemented: TsNonNull expression"),
    TsAs(_) => std::panic!("Not implemented: TsAs expression"),
    TsInstantiation(_) => std::panic!("Not implemented: TsInstantiation expression"),
    PrivateName(_) => std::panic!("Not implemented: PrivateName expression"),
    OptChain(_) => std::panic!("Not implemented: OptChain expression"),
    Invalid(_) => std::panic!("Not implemented: Invalid expression"),
  };
}

fn compile_literal(lit: &swc_ecma_ast::Lit) -> String {
  use swc_ecma_ast::Lit::*;

  return match lit {
    Str(str_) => std::format!("\"{}\"", str_.value), // TODO: Escaping
    Bool(bool_) => bool_.value.to_string(),
    Null(_) => "null".to_string(),
    Num(num) => num.value.to_string(),
    BigInt(_) => std::panic!("Not implemented: BigInt expression"),
    Regex(_) => std::panic!("Not implemented: Regex expression"),
    JSXText(_) => std::panic!("Not implemented: JSXText expression"),
  };
}

fn get_binary_op_str(op: swc_ecma_ast::BinaryOp) -> &'static str {
  use swc_ecma_ast::BinaryOp::*;

  return match op {
    EqEq => "op==",
    NotEq => "op!=",
    EqEqEq => "op===",
    NotEqEq => "op!==",
    Lt => "op<",
    LtEq => "op<=",
    Gt => "op>",
    GtEq => "op>=",
    LShift => "op<<",
    RShift => "op>>",
    ZeroFillRShift => "op>>>",
    Add => "op+",
    Sub => "op-",
    Mul => "op*",
    Div => "op/",
    Mod => "op%",
    BitOr => "op|",
    BitXor => "op^",
    BitAnd => "op&",
    LogicalOr => "op||",
    LogicalAnd => "op&&",
    In => "in",
    InstanceOf => "instanceof",
    Exp => "op**",
    NullishCoalescing => "op??",
  };
}
