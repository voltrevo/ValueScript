use queues::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use swc_common::Spanned;

use crate::asm::{Instruction, Label, Pointer, Register, Value};
use crate::scope::scope_reg;

use super::capture_finder::CaptureFinder;
use super::diagnostic::{Diagnostic, DiagnosticLevel};
use super::expression_compiler::CompiledExpression;
use super::expression_compiler::ExpressionCompiler;
use super::name_allocator::NameAllocator;
use super::scope::{init_std_scope, MappedName, Scope};

#[derive(Clone, Debug)]
pub enum Functionish {
  Fn(swc_ecma_ast::Function),
  Arrow(swc_ecma_ast::ArrowExpr),
  Constructor(Vec<String>, swc_ecma_ast::Constructor),
}

#[derive(Clone, Debug)]
pub struct QueuedFunction {
  pub definition_pointer: Pointer,
  pub fn_name: Option<String>,
  pub capture_params: Vec<String>,
  pub functionish: Functionish,
}

pub struct LoopLabels {
  pub continue_: Label,
  pub break_: Label,
}

pub struct FunctionCompiler {
  pub lines: Vec<String>,
  pub definition_allocator: Rc<RefCell<NameAllocator>>,
  pub reg_allocator: NameAllocator,
  pub label_allocator: NameAllocator,
  pub queue: Queue<QueuedFunction>,
  pub loop_labels: Vec<LoopLabels>,
  pub diagnostics: Vec<Diagnostic>,
}

impl FunctionCompiler {
  pub fn new(definition_allocator: Rc<RefCell<NameAllocator>>) -> FunctionCompiler {
    let mut reg_allocator = NameAllocator::default();
    reg_allocator.allocate(&"return".to_string());
    reg_allocator.allocate(&"this".to_string());
    reg_allocator.allocate(&"ignore".to_string());

    return FunctionCompiler {
      lines: vec![],
      definition_allocator,
      reg_allocator,
      label_allocator: NameAllocator::default(),
      queue: Queue::new(),
      loop_labels: vec![],
      diagnostics: vec![],
    };
  }

  pub fn push(&mut self, instruction: Instruction) {
    self.lines.push(format!("  {}", instruction));
  }

  pub fn label(&mut self, label: Label) {
    self.lines.push(format!("{}", label));
  }

  pub fn todo(&mut self, span: swc_common::Span, message: &str) {
    self.diagnostics.push(Diagnostic {
      level: DiagnosticLevel::InternalError,
      message: format!("TODO: {}", message),
      span: span,
    });
  }

  pub fn allocate_defn(&mut self, name: &str) -> Pointer {
    let allocated_name = self
      .definition_allocator
      .borrow_mut()
      .allocate(&name.to_string());

    Pointer {
      name: allocated_name,
    }
  }

  pub fn allocate_defn_numbered(&mut self, name: &str) -> Pointer {
    let allocated_name = self
      .definition_allocator
      .borrow_mut()
      .allocate_numbered(&name.to_string());

    Pointer {
      name: allocated_name,
    }
  }

  pub fn allocate_tmp(&mut self) -> Register {
    return Register::Named(self.reg_allocator.allocate(&"_tmp".to_string()));
  }

  pub fn allocate_reg(&mut self, based_on: &String) -> Register {
    return Register::Named(self.reg_allocator.allocate(based_on));
  }

  pub fn allocate_numbered_reg(&mut self, prefix: &str) -> Register {
    return Register::Named(self.reg_allocator.allocate_numbered(&prefix.to_string()));
  }

  pub fn release_reg(&mut self, reg: &Register) {
    match reg {
      Register::Named(name) => {
        self.reg_allocator.release(name);
      }
      _ => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: format!("Tried to release non-named register {:?}", reg),
          span: swc_common::DUMMY_SP,
        });
      }
    }
  }

  pub fn compile(
    definition_pointer: Pointer,
    fn_name: Option<String>,
    functionish: Functionish,
    definition_allocator: Rc<RefCell<NameAllocator>>,
    parent_scope: &Scope,
  ) -> (Vec<String>, Vec<Diagnostic>) {
    let mut self_ = FunctionCompiler::new(definition_allocator);

    self_
      .queue
      .add(QueuedFunction {
        definition_pointer: definition_pointer.clone(),
        fn_name,
        capture_params: Vec::new(),
        functionish,
      })
      .expect("Failed to queue function");

    self_.process_queue(parent_scope);

    return (self_.lines, self_.diagnostics);
  }

  pub fn process_queue(&mut self, parent_scope: &Scope) {
    loop {
      match self.queue.remove() {
        Ok(qfn) => self.compile_functionish(
          qfn.definition_pointer,
          qfn.fn_name,
          qfn.capture_params,
          &qfn.functionish,
          parent_scope,
        ),
        Err(_) => {
          break;
        }
      }
    }
  }

  fn compile_functionish(
    &mut self,
    definition_pointer: Pointer,
    fn_name: Option<String>,
    capture_params: Vec<String>,
    functionish: &Functionish,
    parent_scope: &Scope,
  ) {
    let scope = parent_scope.nest();

    // TODO: Use a new FunctionCompiler per function instead of this hack
    self.reg_allocator = NameAllocator::default();

    match fn_name {
      // TODO: Capture propagation when using this name recursively
      Some(fn_name_) => scope.set(fn_name_, MappedName::Definition(definition_pointer.clone())),
      None => {}
    }

    let mut heading = format!("{} = function(", definition_pointer);

    let mut param_count = 0;

    for cap_param in &capture_params {
      if param_count != 0 {
        heading += ", ";
      }

      let reg = self.allocate_reg(cap_param);

      heading += &reg.to_string();

      scope.set(cap_param.clone(), MappedName::Register(reg));

      param_count += 1;
    }

    self.populate_fn_scope_params(functionish, &scope);

    let param_registers = self.allocate_param_registers(functionish, &scope);

    for reg in &param_registers {
      if param_count != 0 {
        heading += ", ";
      }

      heading += &reg.to_string();
      param_count += 1;
    }

    heading += ") {";

    self.lines.push(heading);

    self.add_param_code(functionish, &param_registers, &scope);

    let mut handle_block_body = |block: &swc_ecma_ast::BlockStmt| {
      self.populate_fn_scope(block, &scope);
      self.populate_block_scope(block, &scope);

      for i in 0..block.stmts.len() {
        self.statement(&block.stmts[i], i == block.stmts.len() - 1, &scope);
      }
    };

    match functionish {
      Functionish::Fn(fn_) => {
        match &fn_.body {
          Some(block) => {
            handle_block_body(block);
          }
          None => self.todo(
            fn_.span(),
            "function without body (abstract/interface method?)",
          ),
        };
      }
      Functionish::Arrow(arrow) => match &arrow.body {
        swc_ecma_ast::BlockStmtOrExpr::BlockStmt(block) => {
          handle_block_body(block);
        }
        swc_ecma_ast::BlockStmtOrExpr::Expr(expr) => {
          let mut expression_compiler = ExpressionCompiler {
            fnc: self,
            scope: &scope,
          };

          expression_compiler.compile(expr, Some(Register::Return));
        }
      },
      Functionish::Constructor(_, constructor) => {
        match &constructor.body {
          Some(block) => {
            handle_block_body(block);
          }
          None => self.todo(constructor.span(), "constructor without body"),
        };
      }
    }

    self.lines.push("}".to_string());
  }

  fn populate_fn_scope_params(&mut self, functionish: &Functionish, scope: &Scope) {
    match functionish {
      Functionish::Fn(fn_) => {
        for p in &fn_.params {
          self.populate_scope_pat(&p.pat, scope);
        }
      }
      Functionish::Arrow(arrow) => {
        for p in &arrow.params {
          self.populate_scope_pat(p, scope);
        }
      }
      Functionish::Constructor(_, constructor) => {
        for potspp in &constructor.params {
          match potspp {
            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(ts_param_prop) => {
              use swc_ecma_ast::TsParamPropParam::*;

              match &ts_param_prop.param {
                Ident(ident) => {
                  self.populate_scope_ident(&ident.id, scope);
                }
                Assign(assign) => {
                  self.populate_scope_pat(&assign.left, scope);
                }
              }
            }
            swc_ecma_ast::ParamOrTsParamProp::Param(param) => {
              self.populate_scope_pat(&param.pat, scope);
            }
          }
        }
      }
    }
  }

  fn populate_scope_ident(&mut self, ident: &swc_ecma_ast::Ident, scope: &Scope) {
    scope.set(
      ident.sym.to_string(),
      MappedName::Register(self.allocate_reg(&ident.sym.to_string())),
    );
  }

  fn allocate_param_registers(
    &mut self,
    functionish: &Functionish,
    scope: &Scope,
  ) -> Vec<Register> {
    let mut param_registers = Vec::<Register>::new();

    match functionish {
      Functionish::Fn(fn_) => {
        for p in &fn_.params {
          param_registers.push(self.get_pattern_register(&p.pat, scope));
        }
      }
      Functionish::Arrow(arrow) => {
        for p in &arrow.params {
          param_registers.push(self.get_pattern_register(p, scope));
        }
      }
      Functionish::Constructor(_, constructor) => {
        for potspp in &constructor.params {
          match potspp {
            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(ts_param_prop) => {
              self.todo(ts_param_prop.span(), "TypeScript parameter properties");
              param_registers.push(self.allocate_numbered_reg(&"_todo_ts_param_prop".to_string()));
            }
            swc_ecma_ast::ParamOrTsParamProp::Param(p) => {
              param_registers.push(self.get_pattern_register(&p.pat, scope))
            }
          }
        }
      }
    };

    return param_registers;
  }

  pub fn get_pattern_register(&mut self, param_pat: &swc_ecma_ast::Pat, scope: &Scope) -> Register {
    use swc_ecma_ast::Pat;

    match param_pat {
      Pat::Ident(ident) => self.get_variable_register(&ident.id, scope),
      Pat::Assign(assign) => self.get_pattern_register(&assign.left, scope),
      Pat::Array(_) => self.allocate_numbered_reg(&"_array_pat".to_string()),
      Pat::Object(_) => self.allocate_numbered_reg(&"_object_pat".to_string()),
      Pat::Invalid(_) => self.allocate_numbered_reg(&"_invalid_pat".to_string()),
      Pat::Rest(_) => self.allocate_numbered_reg(&"_rest_pat".to_string()),
      Pat::Expr(_) => self.allocate_numbered_reg(&"_expr_pat".to_string()),
    }
  }

  pub fn get_variable_register(&mut self, ident: &swc_ecma_ast::Ident, scope: &Scope) -> Register {
    match scope.get(&ident.sym.to_string()) {
      Some(MappedName::Register(reg)) => reg,
      _ => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: format!(
            "Register should have been allocated for variable {}",
            ident.sym.to_string()
          ),
          span: ident.span(),
        });

        self.allocate_numbered_reg(&"_error_variable_without_register".to_string())
      }
    }
  }

  fn add_param_code(
    &mut self,
    functionish: &Functionish,
    param_registers: &Vec<Register>,
    scope: &Scope,
  ) {
    match functionish {
      Functionish::Fn(fn_) => {
        for (i, p) in fn_.params.iter().enumerate() {
          let mut ec = ExpressionCompiler { fnc: self, scope };
          ec.pat(&p.pat, &param_registers[i], false, scope);
        }
      }
      Functionish::Arrow(arrow) => {
        for (i, p) in arrow.params.iter().enumerate() {
          let mut ec = ExpressionCompiler { fnc: self, scope };
          ec.pat(p, &param_registers[i], false, scope);
        }
      }
      Functionish::Constructor(_, constructor) => {
        for (i, potspp) in constructor.params.iter().enumerate() {
          match potspp {
            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(_) => {
              // TODO (Diagnostic emitted elsewhere)
            }
            swc_ecma_ast::ParamOrTsParamProp::Param(p) => {
              let mut ec = ExpressionCompiler { fnc: self, scope };
              ec.pat(&p.pat, &param_registers[i], false, scope);
            }
          }
        }
      }
    };
  }

  fn populate_fn_scope(&mut self, block: &swc_ecma_ast::BlockStmt, scope: &Scope) {
    for statement in &block.stmts {
      self.populate_fn_scope_statement(statement, scope);
    }
  }

  fn populate_fn_scope_statement(&mut self, statement: &swc_ecma_ast::Stmt, scope: &Scope) {
    use swc_ecma_ast::Stmt::*;

    match statement {
      Block(nested_block) => {
        self.populate_fn_scope(nested_block, scope);
      }
      Empty(_) => {}
      Debugger(_) => {}
      With(with) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Not supported: With statement".to_string(),
          span: with.span(),
        });
      }
      Return(_) => {}
      Labeled(labeled) => self.todo(labeled.span, "Labeled statement"),
      Break(_) => {}
      Continue(_) => {}
      If(if_) => {
        self.populate_fn_scope_statement(&if_.cons, scope);

        for stmt in &if_.alt {
          self.populate_fn_scope_statement(stmt, scope);
        }
      }
      Switch(switch) => self.todo(switch.span, "Switch statement"),
      Throw(_) => {}
      Try(try_) => self.todo(try_.span, "Try statement"),
      While(while_) => {
        self.populate_fn_scope_statement(&while_.body, scope);
      }
      DoWhile(do_while) => {
        self.populate_fn_scope_statement(&do_while.body, scope);
      }
      For(for_) => {
        match &for_.init {
          Some(swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl)) => {
            self.populate_fn_scope_var_decl(var_decl, scope);
          }
          _ => {}
        };

        self.populate_fn_scope_statement(&for_.body, scope);
      }
      ForIn(for_in) => self.todo(for_in.span, "ForIn statement"),
      ForOf(for_of) => self.todo(for_of.span, "ForOf statement"),
      Decl(decl) => {
        use swc_ecma_ast::Decl::*;

        match decl {
          Class(class) => self.todo(class.span(), "Class declaration"),
          Fn(_) => {}
          Var(var_decl) => self.populate_fn_scope_var_decl(var_decl, scope),
          TsInterface(_) => {}
          TsTypeAlias(_) => {}
          TsEnum(ts_enum) => self.todo(ts_enum.span, "TsEnum declaration"),
          TsModule(ts_module) => self.todo(ts_module.span, "TsModule declaration"),
        }
      }
      Expr(_) => {}
    };
  }

  fn populate_fn_scope_var_decl(&mut self, var_decl: &swc_ecma_ast::VarDecl, scope: &Scope) {
    if var_decl.kind != swc_ecma_ast::VarDeclKind::Var {
      return;
    }

    for decl in &var_decl.decls {
      self.populate_scope_pat(&decl.name, scope);
    }
  }

  fn populate_scope_pat(&mut self, pat: &swc_ecma_ast::Pat, scope: &Scope) {
    use swc_ecma_ast::Pat::*;

    match pat {
      Ident(ident) => {
        let name = ident.id.sym.to_string();
        scope.set(name.clone(), MappedName::Register(self.allocate_reg(&name)));
      }
      Array(array) => {
        for element in &array.elems {
          if let Some(element) = element {
            self.populate_scope_pat(element, scope);
          }
        }
      }
      Object(object) => {
        for prop in &object.props {
          use swc_ecma_ast::ObjectPatProp::*;

          match prop {
            KeyValue(key_value) => {
              self.populate_scope_pat(&key_value.value, scope);
            }
            Assign(assign) => {
              self.populate_scope_ident(&assign.key, scope);
            }
            Rest(rest) => {
              self.populate_scope_pat(&rest.arg, scope);
            }
          }
        }
      }
      Rest(rest) => {
        self.populate_scope_pat(&rest.arg, scope);
      }
      Assign(assign) => {
        self.populate_scope_pat(&assign.left, scope);
      }
      Invalid(invalid) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Invalid pattern".to_string(),
          span: invalid.span(),
        });
      }
      Expr(expr) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "Unexpected Pat::Expr in param/decl context".to_string(),
          span: expr.span(),
        });
      }
    }
  }

  fn populate_block_scope(&mut self, block: &swc_ecma_ast::BlockStmt, scope: &Scope) {
    let mut function_decls = Vec::<swc_ecma_ast::FnDecl>::new();

    for statement in &block.stmts {
      use swc_ecma_ast::Stmt::*;

      match statement {
        Block(_) => {}
        Empty(_) => {}
        Debugger(_) => {}
        With(_) => {
          self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            message: "Not supported: With statement".to_string(),
            span: statement.span(),
          });
        }
        Return(_) => {}
        Labeled(labeled) => self.todo(labeled.span, "Labeled statement"),
        Break(_) => {}
        Continue(_) => {}
        If(_) => {}
        Switch(_) => {}
        Throw(_) => {}
        Try(_) => {}
        While(_) => {}
        DoWhile(_) => {}
        For(_) => {}
        ForIn(_) => {}
        ForOf(_) => {}
        Decl(decl) => {
          use swc_ecma_ast::Decl::*;

          match decl {
            Class(class) => self.todo(class.span(), "Class declaration"),
            Fn(fn_) => function_decls.push(fn_.clone()),
            Var(var_decl) => self.populate_block_scope_var_decl(var_decl, scope),
            TsInterface(_) => {}
            TsTypeAlias(_) => {}
            TsEnum(ts_enum) => self.todo(ts_enum.span, "TsEnum declaration"),
            TsModule(_) => {}
          }
        }
        Expr(_) => {}
      };
    }

    // Create a synth scope where the function decls that can co-mingle are
    // present but don't signal any nested captures. This allows us to first
    // construct all the direct captures and use that to find the complete
    // captures.
    let synth_scope = scope.nest();

    for fn_ in &function_decls {
      synth_scope.set(fn_.ident.sym.to_string(), scope_reg("".to_string()));
    }

    let mut direct_captures_map = HashMap::<String, Vec<String>>::new();

    for fn_ in &function_decls {
      let mut cf = CaptureFinder::new(synth_scope.clone());
      cf.fn_decl(&init_std_scope(), fn_);

      direct_captures_map.insert(fn_.ident.sym.to_string(), cf.ordered_names);
    }

    for fn_ in &function_decls {
      let mut full_captures = Vec::<String>::new();
      let mut full_captures_set = HashSet::<String>::new();

      let mut cap_queue = Queue::<String>::new();

      let direct_captures = direct_captures_map.get(&fn_.ident.sym.to_string());

      match direct_captures {
        Some(direct_captures) => {
          for dc in direct_captures {
            cap_queue.add(dc.clone()).expect("Failed to add to queue");
          }
        }
        None => self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "Direct captures not found".to_string(),
          span: fn_.ident.span,
        }),
      }

      loop {
        let cap = match cap_queue.remove() {
          Ok(c) => c,
          Err(_) => {
            break;
          }
        };

        let is_new = full_captures_set.insert(cap.clone());

        if !is_new {
          continue;
        }

        full_captures.push(cap.clone());

        if let Some(nested_caps) = direct_captures_map.get(&cap) {
          for nested_cap in nested_caps {
            cap_queue
              .add(nested_cap.clone())
              .expect("Failed to add to queue");
          }
        }
      }

      let fn_name = fn_.ident.sym.to_string();

      let definition_pointer = self.allocate_defn(&fn_name);

      let qf = QueuedFunction {
        definition_pointer,
        fn_name: Some(fn_name.clone()),
        capture_params: full_captures,
        functionish: Functionish::Fn(fn_.function.clone()),
      };

      scope.set(fn_name.clone(), MappedName::QueuedFunction(qf.clone()));

      self.queue.add(qf).expect("Failed to queue function");
    }
  }

  fn populate_block_scope_var_decl(&mut self, var_decl: &swc_ecma_ast::VarDecl, scope: &Scope) {
    if var_decl.kind == swc_ecma_ast::VarDeclKind::Var {
      return;
    }

    for decl in &var_decl.decls {
      self.populate_scope_pat(&decl.name, scope);
    }
  }

  fn statement(&mut self, statement: &swc_ecma_ast::Stmt, fn_last: bool, scope: &Scope) {
    use swc_ecma_ast::Stmt::*;

    match statement {
      Block(block) => {
        let block_scope = scope.nest();
        self.populate_block_scope(block, &block_scope);

        for stmt in &block.stmts {
          self.statement(stmt, false, &block_scope);
        }

        for mapping in block_scope.rc.borrow().name_map.values() {
          match mapping {
            MappedName::Register(reg) => {
              self.release_reg(reg);
            }
            MappedName::Definition(_) => {}
            MappedName::QueuedFunction(_) => {}
            MappedName::Builtin(_) => {}
          }
        }
      }
      Empty(_) => {}
      Debugger(debugger) => self.todo(debugger.span, "Debugger statement"),
      With(with) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Not supported: With statement".to_string(),
          span: with.span,
        });
      }

      Return(ret_stmt) => match &ret_stmt.arg {
        None => {
          // TODO: Skip if fn_last
          self.push(Instruction::End);
        }
        Some(expr) => {
          let mut expression_compiler = ExpressionCompiler { fnc: self, scope };

          expression_compiler.compile(expr, Some(Register::Return));

          if !fn_last {
            self.push(Instruction::End);
          }
        }
      },

      Labeled(labeled) => self.todo(labeled.span, "Labeled statement"),

      Break(break_) => {
        if break_.label.is_some() {
          self.todo(break_.span, "labeled break statement");

          return;
        }

        let loop_labels = self.loop_labels.last();

        match loop_labels {
          Some(loop_labels) => {
            self.push(Instruction::Jmp(loop_labels.break_.ref_()));
          }
          None => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::Error,
              message: "break statement outside loop".to_string(),
              span: break_.span,
            });
          }
        }
      }
      Continue(continue_) => {
        if continue_.label.is_some() {
          self.todo(continue_.span, "labeled continue statement");

          return;
        }

        match self.loop_labels.last() {
          Some(loop_labels) => {
            self.push(Instruction::Jmp(loop_labels.continue_.ref_()));
          }
          None => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::Error,
              message: "continue statement outside loop".to_string(),
              span: continue_.span,
            });
          }
        }
      }
      If(if_) => {
        let mut expression_compiler = ExpressionCompiler {
          fnc: self,
          scope: scope,
        };

        let condition = expression_compiler.compile(&*if_.test, None);

        // Usually we wouldn't capture the value_assembly before release, but
        // it's safe to do so here, and allows cond_reg to re-use a register
        // from the condition
        let condition_asm = self.use_(condition);

        let cond_reg = self.allocate_numbered_reg("_cond");

        // TODO: Add negated jmpif instruction to avoid this
        self.push(Instruction::OpNot(condition_asm, cond_reg.clone()));

        let else_label = Label {
          name: self.label_allocator.allocate_numbered(&"else".to_string()),
        };

        self.push(Instruction::JmpIf(
          Value::Register(cond_reg.clone()),
          else_label.ref_(),
        ));

        self.release_reg(&cond_reg);

        self.statement(&*if_.cons, false, scope);

        match &if_.alt {
          None => {
            self.label(else_label);
          }
          Some(alt) => {
            let after_else_label = Label {
              name: self
                .label_allocator
                .allocate_numbered(&"after_else".to_string()),
            };

            self.push(Instruction::Jmp(after_else_label.ref_()));

            self.label(else_label);
            self.statement(&*alt, false, scope);
            self.label(after_else_label);
          }
        }
      }
      Switch(switch) => self.todo(switch.span, "Switch statement"),
      Throw(throw) => self.todo(throw.span, "Throw statement"),
      Try(try_) => self.todo(try_.span, "Try statement"),
      While(while_) => {
        let start_label = Label {
          name: self.label_allocator.allocate_numbered(&"while".to_string()),
        };

        let end_label = Label {
          name: self
            .label_allocator
            .allocate_numbered(&"while_end".to_string()),
        };

        self.loop_labels.push(LoopLabels {
          continue_: start_label.clone(),
          break_: end_label.clone(),
        });

        self.label(start_label.clone());

        let mut expression_compiler = ExpressionCompiler {
          fnc: self,
          scope: scope,
        };

        let condition = expression_compiler.compile(&*while_.test, None);

        // Usually we wouldn't capture the value_assembly before release, but
        // it's safe to do so here, and allows cond_reg to re-use a register
        // from the condition
        let condition_asm = self.use_(condition);

        let cond_reg = self.allocate_numbered_reg(&"_cond".to_string());

        // TODO: Add negated jmpif instruction to avoid this
        self.push(Instruction::OpNot(condition_asm, cond_reg.clone()));

        self.push(Instruction::JmpIf(
          Value::Register(cond_reg.clone()),
          end_label.ref_(),
        ));

        self.release_reg(&cond_reg);
        self.statement(&*while_.body, false, scope);
        self.push(Instruction::Jmp(start_label.ref_()));
        self.label(end_label);

        self.loop_labels.pop();
      }
      DoWhile(do_while) => {
        let start_label = Label {
          name: self
            .label_allocator
            .allocate_numbered(&"do_while".to_string()),
        };

        let continue_label = Label {
          name: self
            .label_allocator
            .allocate_numbered(&"do_while_continue".to_string()),
        };

        let end_label = Label {
          name: self
            .label_allocator
            .allocate_numbered(&"do_while_end".to_string()),
        };

        self.loop_labels.push(LoopLabels {
          continue_: continue_label.clone(),
          break_: end_label.clone(),
        });

        self.label(start_label.clone());

        self.statement(&*do_while.body, false, scope);

        let mut expression_compiler = ExpressionCompiler {
          fnc: self,
          scope: scope,
        };

        let condition = expression_compiler.compile(&*do_while.test, None);

        self.label(continue_label);

        let jmpif = Instruction::JmpIf(self.use_(condition), start_label.ref_());
        self.push(jmpif);

        self.label(end_label);

        self.loop_labels.pop();
      }
      For(for_) => {
        let for_scope = scope.nest();

        match &for_.init {
          Some(swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl)) => {
            self.populate_block_scope_var_decl(var_decl, &for_scope);
          }
          _ => {}
        }

        match &for_.init {
          Some(var_decl_or_expr) => match var_decl_or_expr {
            swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl) => {
              self.var_declaration(var_decl, &for_scope);
            }
            swc_ecma_ast::VarDeclOrExpr::Expr(expr) => {
              self.expression(expr, &for_scope);
            }
          },
          None => {}
        }

        let for_test_label = Label {
          name: self
            .label_allocator
            .allocate_numbered(&"for_test".to_string()),
        };

        let for_continue_label = Label {
          name: self
            .label_allocator
            .allocate_numbered(&"for_continue".to_string()),
        };

        let for_end_label = Label {
          name: self
            .label_allocator
            .allocate_numbered(&"for_end".to_string()),
        };

        self.label(for_test_label.clone());

        self.loop_labels.push(LoopLabels {
          continue_: for_continue_label.clone(),
          break_: for_end_label.clone(),
        });

        match &for_.test {
          Some(cond) => {
            let mut ec = ExpressionCompiler {
              fnc: self,
              scope: &for_scope,
            };

            let condition = ec.compile(cond, None);

            // Usually we wouldn't capture the value_assembly before release, but
            // it's safe to do so here, and allows cond_reg to re-use a register
            // from the condition
            let condition_asm = self.use_(condition);

            let cond_reg = self.allocate_numbered_reg("_cond");

            // TODO: Add negated jmpif instruction to avoid this
            self.push(Instruction::OpNot(condition_asm, cond_reg.clone()));

            self.push(Instruction::JmpIf(
              Value::Register(cond_reg.clone()),
              for_end_label.ref_(),
            ));

            self.release_reg(&cond_reg);
          }
          None => {}
        }

        self.statement(&for_.body, false, &for_scope);

        self.label(for_continue_label);

        match &for_.update {
          Some(update) => self.expression(update, &for_scope),
          None => {}
        }

        self.push(Instruction::Jmp(for_test_label.ref_()));

        self.label(for_end_label);

        self.loop_labels.pop();
      }
      ForIn(for_in) => self.todo(for_in.span, "ForIn statement"),
      ForOf(for_of) => self.todo(for_of.span, "ForOf statement"),
      Decl(decl) => {
        self.declaration(decl, scope);
      }
      Expr(expr) => {
        self.expression(&expr.expr, scope);
      }
    }
  }

  fn declaration(&mut self, decl: &swc_ecma_ast::Decl, scope: &Scope) {
    use swc_ecma_ast::Decl::*;

    match decl {
      Class(class) => self.todo(class.span(), "Class declaration"),
      Fn(_) => {}
      Var(var_decl) => self.var_declaration(var_decl, scope),
      TsInterface(interface_decl) => self.todo(interface_decl.span, "TsInterface declaration"),
      TsTypeAlias(_) => {}
      TsEnum(ts_enum) => self.todo(ts_enum.span, "TsEnum declaration"),
      TsModule(ts_module) => self.todo(ts_module.span, "TsModule declaration"),
    };
  }

  fn var_declaration(&mut self, var_decl: &swc_ecma_ast::VarDecl, scope: &Scope) {
    for decl in &var_decl.decls {
      match &decl.init {
        Some(expr) => {
          let target_register = self.get_pattern_register(&decl.name, scope);

          let mut ec = ExpressionCompiler { fnc: self, scope };
          ec.compile(expr, Some(target_register.clone()));
          ec.pat(&decl.name, &target_register, false, scope);
        }
        None => match &decl.name {
          swc_ecma_ast::Pat::Ident(_) => {
            // Nothing to do - identifier without initializer should be
            // undefined
          }
          _ => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "Expected destructuring declaration without initializer \
                to be caught in the parser. Pattern has not been compiled."
                .to_string(),
              span: decl.span(),
            });
          }
        },
      }
    }
  }

  fn expression(&mut self, expr: &swc_ecma_ast::Expr, scope: &Scope) {
    let mut expression_compiler = ExpressionCompiler { fnc: self, scope };
    let compiled = expression_compiler.compile_top_level(expr, None);

    self.use_(compiled);
  }

  pub fn use_(&mut self, mut compiled_expr: CompiledExpression) -> Value {
    let asm = compiled_expr.value;

    for reg in &compiled_expr.nested_registers {
      self.release_reg(reg);
    }

    compiled_expr.release_checker.has_unreleased_registers = false;

    return asm;
  }

  pub fn use_ref(&mut self, compiled_expr: &mut CompiledExpression) -> Value {
    let asm = compiled_expr.value.clone();

    for reg in &compiled_expr.nested_registers {
      self.release_reg(reg);
    }

    compiled_expr.release_checker.has_unreleased_registers = false;

    return asm;
  }
}
