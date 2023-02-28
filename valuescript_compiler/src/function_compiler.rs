use queues::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use swc_common::Spanned;

use super::capture_finder::CaptureFinder;
use super::diagnostic::{Diagnostic, DiagnosticLevel};
use super::expression_compiler::CompiledExpression;
use super::expression_compiler::ExpressionCompiler;
use super::name_allocator::NameAllocator;
use super::scope::{init_std_scope, MappedName, Scope, ScopeTrait};

#[derive(Clone, Debug)]
pub enum Functionish {
  Fn(swc_ecma_ast::Function),
  Arrow(swc_ecma_ast::ArrowExpr),
  Constructor(Vec<String>, swc_ecma_ast::Constructor),
}

#[derive(Clone, Debug)]
pub struct QueuedFunction {
  pub definition_name: String,
  pub fn_name: Option<String>,
  pub capture_params: Vec<String>,
  pub functionish: Functionish,
}

pub struct LoopLabels {
  pub continue_: String,
  pub break_: String,
}

pub struct FunctionCompiler {
  pub definition: Vec<String>,
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

    return FunctionCompiler {
      definition: Vec::new(),
      definition_allocator: definition_allocator,
      reg_allocator: reg_allocator,
      label_allocator: NameAllocator::default(),
      queue: Queue::new(),
      loop_labels: vec![],
      diagnostics: vec![],
    };
  }

  pub fn todo(&mut self, span: swc_common::Span, message: &str) {
    self.diagnostics.push(Diagnostic {
      level: DiagnosticLevel::InternalError,
      message: format!("TODO: {}", message),
      span: span,
    });
  }

  pub fn compile(
    definition_name: String,
    fn_name: Option<String>,
    functionish: Functionish,
    definition_allocator: Rc<RefCell<NameAllocator>>,
    parent_scope: &Scope,
  ) -> (Vec<String>, Vec<Diagnostic>) {
    let mut self_ = FunctionCompiler::new(definition_allocator);

    self_
      .queue
      .add(QueuedFunction {
        definition_name: definition_name.clone(),
        fn_name: fn_name,
        capture_params: Vec::new(),
        functionish: functionish,
      })
      .expect("Failed to queue function");

    self_.process_queue(parent_scope);

    return (self_.definition, self_.diagnostics);
  }

  pub fn process_queue(&mut self, parent_scope: &Scope) {
    loop {
      match self.queue.remove() {
        Ok(qfn) => self.compile_functionish(
          qfn.definition_name,
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
    definition_name: String,
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
      Some(fn_name_) => scope.set(fn_name_, MappedName::Definition(definition_name.clone())),
      None => {}
    }

    let mut heading = "@".to_string();
    heading += &definition_name;
    heading += " = function(";

    let mut param_count = 0;

    for cap_param in &capture_params {
      if param_count != 0 {
        heading += ", ";
      }

      let reg = self.reg_allocator.allocate(cap_param);

      heading += "%";
      heading += &reg;

      scope.set(cap_param.clone(), MappedName::Register(reg));

      param_count += 1;
    }

    let param_registers = self.allocate_param_registers(functionish);

    for reg in &param_registers {
      if param_count != 0 {
        heading += ", ";
      }

      heading += "%";
      heading += reg;

      param_count += 1;
    }

    heading += ") {";

    self.definition.push(heading);

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

          expression_compiler.compile(expr, Some("return".to_string()));
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

    self.definition.push("}".to_string());
  }

  fn allocate_param_registers(&mut self, functionish: &Functionish) -> Vec<String> {
    let mut param_registers = Vec::<String>::new();

    match functionish {
      Functionish::Fn(fn_) => {
        for p in &fn_.params {
          param_registers.push(self.allocate_param_reg(&p.pat));
        }
      }
      Functionish::Arrow(arrow) => {
        for p in &arrow.params {
          param_registers.push(self.allocate_param_reg(p));
        }
      }
      Functionish::Constructor(_, constructor) => {
        for potspp in &constructor.params {
          match potspp {
            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(ts_param_prop) => {
              self.todo(
                ts_param_prop.span(),
                "TypeScript parameter properties (what are these?)",
              );

              param_registers.push(
                self
                  .reg_allocator
                  .allocate_numbered(&"_todo_ts_param_prop".to_string()),
              );
            }
            swc_ecma_ast::ParamOrTsParamProp::Param(p) => {
              param_registers.push(self.allocate_param_reg(&p.pat))
            }
          }
        }
      }
    };

    return param_registers;
  }

  fn allocate_param_reg(&mut self, param_pat: &swc_ecma_ast::Pat) -> String {
    use swc_ecma_ast::Pat;

    match param_pat {
      Pat::Ident(ident) => self.reg_allocator.allocate(&ident.id.sym.to_string()),
      Pat::Assign(assign) => self.allocate_param_reg(&assign.left),
      Pat::Array(_) => self
        .reg_allocator
        .allocate_numbered(&"_array_pat".to_string()),
      Pat::Object(_) => self
        .reg_allocator
        .allocate_numbered(&"_object_pat".to_string()),
      Pat::Invalid(_) => self
        .reg_allocator
        .allocate_numbered(&"_invalid_pat".to_string()),
      Pat::Rest(_) => self
        .reg_allocator
        .allocate_numbered(&"_rest_pat".to_string()),
      Pat::Expr(_) => self
        .reg_allocator
        .allocate_numbered(&"_expr_pat".to_string()),
    }
  }

  fn add_param_code(
    &mut self,
    functionish: &Functionish,
    param_registers: &Vec<String>,
    scope: &Scope,
  ) {
    match functionish {
      Functionish::Fn(fn_) => {
        for (i, p) in fn_.params.iter().enumerate() {
          self.decl_or_param_pat(&p.pat, &param_registers[i], scope);
        }
      }
      Functionish::Arrow(arrow) => {
        for (i, p) in arrow.params.iter().enumerate() {
          self.decl_or_param_pat(p, &param_registers[i], scope);
        }
      }
      Functionish::Constructor(_, constructor) => {
        for (i, potspp) in constructor.params.iter().enumerate() {
          match potspp {
            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(_) => {
              // TODO (Diagnostic emitted elsewhere)
            }
            swc_ecma_ast::ParamOrTsParamProp::Param(p) => {
              self.decl_or_param_pat(&p.pat, &param_registers[i], scope);
            }
          }
        }
      }
    };
  }

  fn decl_or_param_pat(&mut self, pat: &swc_ecma_ast::Pat, register: &String, scope: &Scope) {
    use swc_ecma_ast::Pat;

    match pat {
      Pat::Ident(ident) => {
        scope.set(
          ident.id.sym.to_string(),
          MappedName::Register(register.clone()),
        );
      }
      Pat::Assign(assign) => {
        self.default_expr(&assign.right, register, scope);
        self.decl_or_param_pat(&assign.left, register, scope);
      }
      Pat::Array(array) => {
        for (i, elem_opt) in array.elems.iter().enumerate() {
          let elem = match elem_opt {
            Some(elem) => elem,
            None => continue,
          };

          let elem_reg = self.allocate_param_reg(elem);

          self
            .definition
            .push(format!("  sub %{} {} %{}", register, i, elem_reg));

          self.decl_or_param_pat(elem, &elem_reg, scope);
        }

        self.reg_allocator.release(register);
      }
      Pat::Object(object) => {
        for prop in &object.props {
          use swc_ecma_ast::ObjectPatProp;

          match prop {
            ObjectPatProp::KeyValue(kv) => {
              let mut ec = ExpressionCompiler {
                fnc: self,
                scope: scope,
              };

              let param_reg = ec.fnc.allocate_param_reg(&kv.value);
              let compiled_key = ec.prop_name(&kv.key);

              let sub_instr = format!(
                "  sub %{} {} %{}",
                register,
                ec.fnc.use_(compiled_key),
                param_reg
              );

              ec.fnc.definition.push(sub_instr);

              ec.fnc.decl_or_param_pat(&kv.value, &param_reg, scope);
            }
            ObjectPatProp::Assign(assign) => {
              let key = assign.key.sym.to_string();
              let reg = self.reg_allocator.allocate(&key);

              self
                .definition
                .push(format!("  sub %{} \"{}\" %{}", register, key, reg));

              if let Some(value) = &assign.value {
                self.default_expr(value, &reg, scope);
              }

              scope.set(key, MappedName::Register(reg));
            }
            ObjectPatProp::Rest(rest) => {
              self.todo(rest.span, "Rest pattern in object destructuring");
            }
          }
        }

        self.reg_allocator.release(register);
      }
      Pat::Invalid(_) => {
        // Diagnostic emitted elsewhere
      }
      Pat::Rest(_) => {
        // TODO (Diagnostic emitted elsewhere)
      }
      Pat::Expr(_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "Unexpected Pat::Expr in param/decl context".to_string(),
          span: pat.span(),
        });
      }
    }
  }

  fn default_expr(&mut self, expr: &swc_ecma_ast::Expr, register: &String, scope: &Scope) {
    let provided_reg = self.reg_allocator.allocate_numbered(&"_tmp".to_string());

    let initialized_label = self
      .label_allocator
      .allocate(&format!("{}_initialized", register));

    self
      .definition
      .push(format!("  op!== %{} undefined %{}", register, provided_reg));

    self
      .definition
      .push(format!("  jmpif %{} :{}", provided_reg, initialized_label));

    self.reg_allocator.release(&provided_reg);

    let mut expression_compiler = ExpressionCompiler {
      fnc: self,
      scope: scope,
    };

    let compiled = expression_compiler.compile(expr, Some(register.clone()));

    if self.use_(compiled) != format!("%{}", register) {
      self.diagnostics.push(Diagnostic {
        level: DiagnosticLevel::InternalError,
        message: "Default expression not compiled into target register (not sure whether this is possible in this case)".to_string(),
        span: expr.span(),
      });
    }

    self.definition.push(format!("{}:", initialized_label));
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
      match &decl.name {
        swc_ecma_ast::Pat::Ident(ident) => {
          let name = ident.id.sym.to_string();

          scope.set(
            name.clone(),
            MappedName::Register(self.reg_allocator.allocate(&name)),
          );
        }
        _ => self.todo(var_decl.span(), "destructuring"),
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
      synth_scope.set(
        fn_.ident.sym.to_string(),
        MappedName::Register("".to_string()),
      );
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

      let definition_name = self.definition_allocator.borrow_mut().allocate(&fn_name);

      let qf = QueuedFunction {
        definition_name: definition_name,
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
      match &decl.name {
        swc_ecma_ast::Pat::Ident(ident) => {
          let name = ident.id.sym.to_string();

          scope.set(
            name.clone(),
            MappedName::Register(self.reg_allocator.allocate(&name)),
          );
        }
        _ => self.todo(decl.span(), "destructuring"),
      }
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

        for mapping in block_scope.borrow().name_map.values() {
          match mapping {
            MappedName::Register(reg) => {
              self.reg_allocator.release(reg);
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
          self.definition.push("  end".to_string());
        }
        Some(expr) => {
          let mut expression_compiler = ExpressionCompiler {
            fnc: self,
            scope: scope,
          };

          expression_compiler.compile(expr, Some("return".to_string()));

          if !fn_last {
            self.definition.push("  end".to_string());
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
            self
              .definition
              .push(format!("  jmp :{}", loop_labels.break_));
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
            self
              .definition
              .push(format!("  jmp :{}", loop_labels.continue_));
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

        let cond_reg = self.reg_allocator.allocate_numbered(&"_cond".to_string());

        // TODO: Add negated jmpif instruction to avoid this
        self
          .definition
          .push(std::format!("  op! {} %{}", condition_asm, cond_reg));

        let else_label = self.label_allocator.allocate_numbered(&"else".to_string());

        let mut jmpif_instr = "  jmpif %".to_string();
        jmpif_instr += &cond_reg;
        jmpif_instr += " :";
        jmpif_instr += &else_label;
        self.definition.push(jmpif_instr);

        self.reg_allocator.release(&cond_reg);

        self.statement(&*if_.cons, false, scope);

        match &if_.alt {
          None => {
            self.definition.push(std::format!("{}:", else_label));
          }
          Some(alt) => {
            let after_else_label = self
              .label_allocator
              .allocate_numbered(&"after_else".to_string());
            self
              .definition
              .push(std::format!("  jmp :{}", after_else_label));
            self.definition.push(std::format!("{}:", else_label));
            self.statement(&*alt, false, scope);
            self.definition.push(std::format!("{}:", after_else_label));
          }
        }
      }
      Switch(switch) => self.todo(switch.span, "Switch statement"),
      Throw(throw) => self.todo(throw.span, "Throw statement"),
      Try(try_) => self.todo(try_.span, "Try statement"),
      While(while_) => {
        let start_label = self.label_allocator.allocate_numbered(&"while".to_string());

        let end_label = self
          .label_allocator
          .allocate_numbered(&"while_end".to_string());

        self.loop_labels.push(LoopLabels {
          continue_: start_label.clone(),
          break_: end_label.clone(),
        });

        self.definition.push(std::format!("{}:", start_label));

        let mut expression_compiler = ExpressionCompiler {
          fnc: self,
          scope: scope,
        };

        let condition = expression_compiler.compile(&*while_.test, None);

        // Usually we wouldn't capture the value_assembly before release, but
        // it's safe to do so here, and allows cond_reg to re-use a register
        // from the condition
        let condition_asm = self.use_(condition);

        let cond_reg = self.reg_allocator.allocate_numbered(&"_cond".to_string());

        // TODO: Add negated jmpif instruction to avoid this
        self
          .definition
          .push(std::format!("  op! {} %{}", condition_asm, cond_reg));

        let mut jmpif_instr = "  jmpif %".to_string();
        jmpif_instr += &cond_reg;
        jmpif_instr += " :";
        jmpif_instr += &end_label;
        self.definition.push(jmpif_instr);

        self.reg_allocator.release(&cond_reg);

        self.statement(&*while_.body, false, scope);
        self.definition.push(std::format!("  jmp :{}", start_label));

        self.definition.push(std::format!("{}:", end_label));

        self.loop_labels.pop();
      }
      DoWhile(do_while) => {
        let start_label = self
          .label_allocator
          .allocate_numbered(&"do_while".to_string());

        let continue_label = self
          .label_allocator
          .allocate_numbered(&"do_while_continue".to_string());

        let end_label = self
          .label_allocator
          .allocate_numbered(&"do_while_end".to_string());

        self.loop_labels.push(LoopLabels {
          continue_: continue_label.clone(),
          break_: end_label.clone(),
        });

        self.definition.push(std::format!("{}:", start_label));

        self.statement(&*do_while.body, false, scope);

        let mut expression_compiler = ExpressionCompiler {
          fnc: self,
          scope: scope,
        };

        let condition = expression_compiler.compile(&*do_while.test, None);

        self.definition.push(format!("{}:", continue_label));

        let mut jmpif_instr = "  jmpif ".to_string();
        jmpif_instr += &self.use_(condition);
        jmpif_instr += " :";
        jmpif_instr += &start_label;
        self.definition.push(jmpif_instr);

        self.definition.push(format!("{}:", end_label));

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

        let for_test_label = self
          .label_allocator
          .allocate_numbered(&"for_test".to_string());

        let for_continue_label = self
          .label_allocator
          .allocate_numbered(&"for_continue".to_string());

        let for_end_label = self
          .label_allocator
          .allocate_numbered(&"for_end".to_string());

        self.definition.push(format!("{}:", &for_test_label));

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

            let cond_reg = self.reg_allocator.allocate_numbered(&"_cond".to_string());

            // TODO: Add negated jmpif instruction to avoid this
            self
              .definition
              .push(std::format!("  op! {} %{}", condition_asm, cond_reg));

            let mut jmpif_instr = "  jmpif %".to_string();
            jmpif_instr += &cond_reg;
            jmpif_instr += " :";
            jmpif_instr += &for_end_label;
            self.definition.push(jmpif_instr);

            self.reg_allocator.release(&cond_reg);
          }
          None => {}
        }

        self.statement(&for_.body, false, &for_scope);

        self.definition.push(format!("{}:", for_continue_label));

        match &for_.update {
          Some(update) => self.expression(update, &for_scope),
          None => {}
        }

        self.definition.push(format!("  jmp :{}", for_test_label));

        self.definition.push(format!("{}:", for_end_label));

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
          let mut expr_compiler = ExpressionCompiler { fnc: self, scope };

          let name = match &decl.name {
            swc_ecma_ast::Pat::Ident(ident) => ident.id.sym.to_string(),
            _ => {
              self.todo(decl.span(), "destructuring");

              return;
            }
          };

          let target_register = match scope.get(&name) {
            Some(MappedName::Register(reg_name)) => reg_name,
            _ => {
              self.todo(
                decl.span(),
                "var decl should always get mapped to a register during scan",
              );

              return;
            }
          };

          expr_compiler.compile(expr, Some(target_register));
        }
        None => {}
      }
    }
  }

  fn expression(&mut self, expr: &swc_ecma_ast::Expr, scope: &Scope) {
    let mut expression_compiler = ExpressionCompiler {
      fnc: self,
      scope: scope,
    };

    let compiled = expression_compiler.compile(
      expr, // FIXME: Specify the ignore register instead
      None,
    );

    self.use_(compiled);
  }

  pub fn use_(&mut self, mut compiled_expr: CompiledExpression) -> String {
    let asm = compiled_expr.value_assembly;

    for reg in &compiled_expr.nested_registers {
      self.reg_allocator.release(reg);
    }

    compiled_expr.release_checker.has_unreleased_registers = false;

    return asm;
  }

  pub fn use_ref(&mut self, compiled_expr: &mut CompiledExpression) -> String {
    let asm = compiled_expr.value_assembly.clone();

    for reg in &compiled_expr.nested_registers {
      self.reg_allocator.release(reg);
    }

    compiled_expr.release_checker.has_unreleased_registers = false;

    return asm;
  }
}
