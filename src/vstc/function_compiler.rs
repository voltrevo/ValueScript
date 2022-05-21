use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use queues::*;

use super::name_allocator::NameAllocator;
use super::expression_compiler::ExpressionCompiler;
use super::scope::{Scope, MappedName, ScopeTrait, init_scope};
use super::capture_finder::CaptureFinder;

#[derive(Clone, Debug)]
pub enum FnOrArrow {
  Fn(swc_ecma_ast::Function),
  Arrow(swc_ecma_ast::ArrowExpr),
}

#[derive(Clone, Debug)]
pub struct QueuedFunction {
  pub definition_name: String,
  pub fn_name: Option<String>,
  pub capture_params: Vec<String>,
  pub fn_or_arrow: FnOrArrow,
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
}

impl FunctionCompiler {
  fn new(definition_allocator: Rc<RefCell<NameAllocator>>) -> FunctionCompiler {
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
    };
  }

  pub fn compile(
    definition_name: String,
    fn_name: Option<String>,
    fn_: &swc_ecma_ast::Function,
    definition_allocator: Rc<RefCell<NameAllocator>>,
    parent_scope: &Scope,
  ) -> Vec<String> {
    let mut self_ = FunctionCompiler::new(definition_allocator);

    self_.queue.add(QueuedFunction {
      definition_name: definition_name.clone(),
      fn_name: fn_name,
      capture_params: Vec::new(),
      fn_or_arrow: FnOrArrow::Fn(fn_.clone()),
    }).expect("Failed to queue function");

    loop {
      match self_.queue.remove() {
        Ok(qfn) => self_.compile_fn_or_arrow(
          qfn.definition_name,
          qfn.fn_name,
          qfn.capture_params,
          &qfn.fn_or_arrow,
          parent_scope,
        ),
        Err(_) => { break; },
      }
    }

    return self_.definition;
  }

  fn compile_fn_or_arrow(
    &mut self,
    definition_name: String,
    fn_name: Option<String>,
    mut capture_params: Vec<String>,
    fn_or_arrow: &FnOrArrow,
    parent_scope: &Scope,
  ) {
    let scope = parent_scope.nest();

    // TODO: Use a new FunctionCompiler per function instead of this hack
    self.reg_allocator = NameAllocator::default();

    match fn_name {
      // TODO: Capture propagation when using this name recursively
      Some(fn_name_) => scope.set(
        fn_name_,
        MappedName::Definition(definition_name.clone()),
      ),
      None => {},
    }

    let mut heading = "@".to_string();
    heading += &definition_name;
    heading += " = function(";

    let mut params = Vec::<String>::new();
    params.append(&mut capture_params);

    let mut handle_param_pat = |pat: &swc_ecma_ast::Pat| match pat {
      swc_ecma_ast::Pat::Ident(binding_ident) => {
        let param_name = binding_ident.id.sym.to_string();
        params.push(param_name);
      },
      _ => std::panic!("Not implemented: parameter destructuring"),
    };

    match fn_or_arrow {
      FnOrArrow::Fn(fn_) => {
        for p in &fn_.params {
          handle_param_pat(&p.pat);
        }
      },
      FnOrArrow::Arrow(arrow) => {
        for p in &arrow.params {
          handle_param_pat(p);
        }
      },
    };

    for i in 0..params.len() {
      let reg = self.reg_allocator.allocate(&params[i]);

      heading += "%";
      heading += &reg;

      scope.set(
        params[i].clone(),
        MappedName::Register(reg),
      );

      if i != params.len() - 1 {
        heading += ", ";
      }
    }

    heading += ") {";

    self.definition.push(heading);

    let mut handle_block_body = |block: &swc_ecma_ast::BlockStmt| {
      self.populate_fn_scope(block, &scope);
      self.populate_block_scope(block, &scope);
  
      for i in 0..block.stmts.len() {
        self.statement(
          &block.stmts[i],
          i == block.stmts.len() - 1,
          &scope,
        );
      }
    };

    match fn_or_arrow {
      FnOrArrow::Fn(fn_) => {
        let block = fn_.body.as_ref()
          .expect("Not implemented: function without body");

        handle_block_body(block);
      },
      FnOrArrow::Arrow(arrow) => match &arrow.body {
        swc_ecma_ast::BlockStmtOrExpr::BlockStmt(block) => {
          handle_block_body(block);
        },
        swc_ecma_ast::BlockStmtOrExpr::Expr(expr) => {
          let mut expression_compiler = ExpressionCompiler {
            fnc: self,
            scope: &scope,
          };

          expression_compiler.compile(expr, Some("return".to_string()));
        }
      },
    }

    self.definition.push("}".to_string());
  }

  fn populate_fn_scope(
    &mut self,
    block: &swc_ecma_ast::BlockStmt,
    scope: &Scope,
  ) {
    for statement in &block.stmts {
      self.populate_fn_scope_statement(statement, scope);
    }
  }

  fn populate_fn_scope_statement(
    &mut self,
    statement: &swc_ecma_ast::Stmt,
    scope: &Scope,
  ) {
    use swc_ecma_ast::Stmt::*;

    match statement {
      Block(nested_block) => {
        self.populate_fn_scope(nested_block, scope);
      },
      Empty(_) => {},
      Debugger(_) => {},
      With(_) => std::panic!("Not supported: With statement"),
      Return(_) => {},
      Labeled(_) => std::panic!("Not implemented: Labeled statement"),
      Break(_) => {},
      Continue(_) => {},
      If(if_) => {
        self.populate_fn_scope_statement(&if_.cons, scope);

        for stmt in &if_.alt {
          self.populate_fn_scope_statement(stmt, scope);
        }
      },
      Switch(_) => std::panic!("Not implemented: Switch statement"),
      Throw(_) => {},
      Try(_) => std::panic!("Not implemented: Try statement"),
      While(while_) => {
        self.populate_fn_scope_statement(&while_.body, scope);
      },
      DoWhile(do_while) => {
        self.populate_fn_scope_statement(&do_while.body, scope);
      },
      For(for_) => {
        match &for_.init {
          Some(swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl)) => {
            self.populate_fn_scope_var_decl(var_decl, scope);
          },
          _ => {},
        };

        self.populate_fn_scope_statement(&for_.body, scope);
      },
      ForIn(_) => std::panic!("Not implemented: ForIn statement"),
      ForOf(_) => std::panic!("Not implemented: ForOf statement"),
      Decl(decl) => {
        use swc_ecma_ast::Decl::*;

        match decl {
          Class(_) => std::panic!("Not implemented: Class declaration"),
          Fn(_) => {},
          Var(var_decl) => self.populate_fn_scope_var_decl(var_decl, scope),
          TsInterface(_) => {},
          TsTypeAlias(_) => {},
          TsEnum(_) => std::panic!("Not implemented: TsEnum declaration"),
          TsModule(_) => std::panic!("Not implemented: TsModule declaration"),
        }
      },
      Expr(_) => {},
    };
  }

  fn populate_fn_scope_var_decl(
    &mut self,
    var_decl: &swc_ecma_ast::VarDecl,
    scope: &Scope,
  ) {
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
        },
        _ => std::panic!("Not implemented: destructuring"),
      }
    }
  }

  fn populate_block_scope(
    &mut self,
    block: &swc_ecma_ast::BlockStmt,
    scope: &Scope,
  ) {
    let mut function_decls = Vec::<swc_ecma_ast::FnDecl>::new();

    for statement in &block.stmts {
      use swc_ecma_ast::Stmt::*;

      match statement {
        Block(_) => {},
        Empty(_) => {},
        Debugger(_) => {},
        With(_) => std::panic!("Not supported: With statement"),
        Return(_) => {},
        Labeled(_) => std::panic!("Not implemented: Labeled statement"),
        Break(_) => {},
        Continue(_) => {},
        If(_) => {},
        Switch(_) => {},
        Throw(_) => {},
        Try(_) => {},
        While(_) => {},
        DoWhile(_) => {},
        For(_) => {},
        ForIn(_) => {},
        ForOf(_) => {},
        Decl(decl) => {
          use swc_ecma_ast::Decl::*;
  
          match decl {
            Class(_) => std::panic!("Not implemented: Class declaration"),
            Fn(fn_) => function_decls.push(fn_.clone()),
            Var(var_decl) => self.populate_block_scope_var_decl(var_decl, scope),
            TsInterface(_) => {},
            TsTypeAlias(_) => {},
            TsEnum(_) => std::panic!("Not implemented: TsEnum declaration"),
            TsModule(_) => {},
          }
        },
        Expr(_) => {},
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
      cf.fn_decl(&init_scope(), fn_);

      direct_captures_map.insert(
        fn_.ident.sym.to_string(),
        cf.ordered_names,
      );
    }

    for fn_ in &function_decls {
      let mut full_captures = Vec::<String>::new();
      let mut full_captures_set = HashSet::<String>::new();

      let mut cap_queue = Queue::<String>::new();

      for dc in direct_captures_map.get(&fn_.ident.sym.to_string())
        .expect("direct captures not found")
      {
        cap_queue.add(dc.clone())
          .expect("Failed to add to queue");
      }

      loop {
        let cap = match cap_queue.remove() {
          Ok(c) => c,
          Err(_) => { break; },
        };

        let is_new = full_captures_set.insert(cap.clone());

        if !is_new {
          continue;
        }

        full_captures.push(cap.clone());

        for nested_caps in direct_captures_map.get(&cap) {
          for nested_cap in nested_caps {
            cap_queue.add(nested_cap.clone())
              .expect("Failed to add to queue");
          }
        }
      }

      let fn_name = fn_.ident.sym.to_string();

      let definition_name = self
        .definition_allocator
        .borrow_mut()
        .allocate(&fn_name)
      ;

      let qf = QueuedFunction {
        definition_name: definition_name,
        fn_name: Some(fn_name.clone()),
        capture_params: full_captures,
        fn_or_arrow: FnOrArrow::Fn(fn_.function.clone()),
      };

      scope.set(
        fn_name.clone(),
        MappedName::QueuedFunction(qf.clone()),
      );

      self.queue.add(qf).expect("Failed to queue function");
    }
  }

  fn populate_block_scope_var_decl(
    &mut self,
    var_decl: &swc_ecma_ast::VarDecl,
    scope: &Scope,
  ) {
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
        },
        _ => std::panic!("Not implemented: destructuring"),
      }
    }
  }

  fn statement(
    &mut self,
    statement: &swc_ecma_ast::Stmt,
    fn_last: bool,
    scope: &Scope,
  ) {
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
            },
            MappedName::Definition(_) => {},
            MappedName::QueuedFunction(_) => {},
          }
        }
      },
      Empty(_) => {},
      Debugger(_) => std::panic!("Not implemented: Debugger statement"),
      With(_) => std::panic!("Not supported: With statement"),

      Return(ret_stmt) => match &ret_stmt.arg {
        None => {
          // TODO: Skip if fn_last
          self.definition.push("  end".to_string());
        },
        Some(expr) => {
          let mut expression_compiler = ExpressionCompiler {
            fnc: self,
            scope: scope,
          };

          expression_compiler.compile(expr, Some("return".to_string()));

          if !fn_last {
            self.definition.push("  end".to_string());
          }
        },
      },

      Labeled(_) => std::panic!("Not implemented: Labeled statement"),

      Break(break_) => {
        if break_.label.is_some() {
          std::panic!("Not implemented: labeled break statement");
        }

        let loop_labels = self.loop_labels.last()
          .expect("break statement outside loop")
        ;

        self.definition.push(
          format!("  jmp :{}", loop_labels.break_)
        );
      },
      Continue(continue_) => {
        if continue_.label.is_some() {
          std::panic!("Not implemented: labeled continue statement");
        }

        let loop_labels = self.loop_labels.last()
          .expect("continue statement outside loop")
        ;

        self.definition.push(
          format!("  jmp :{}", loop_labels.continue_)
        );
      },
      If(if_) => {
        let mut expression_compiler = ExpressionCompiler {
          fnc: self,
          scope: scope,
        };

        let condition = expression_compiler.compile(&*if_.test, None);

        for reg in condition.nested_registers {
          self.reg_allocator.release(&reg);
        }

        let cond_reg = self.reg_allocator.allocate_numbered(&"_cond".to_string());

        // TODO: Add negated jmpif instruction to avoid this
        self.definition.push(std::format!(
          "  op! {} %{}",
          condition.value_assembly,
          cond_reg,
        ));

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
          },
          Some(alt) => {
            let after_else_label = self.label_allocator.allocate_numbered(&"after_else".to_string());
            self.definition.push(std::format!("  jmp :{}", after_else_label));
            self.definition.push(std::format!("{}:", else_label));
            self.statement(&*alt, false, scope);
            self.definition.push(std::format!("{}:", after_else_label));
          }
        }
      },
      Switch(_) => std::panic!("Not implemented: Switch statement"),
      Throw(_) => std::panic!("Not implemented: Throw statement"),
      Try(_) => std::panic!("Not implemented: Try statement"),
      While(while_) => {
        let start_label = self.label_allocator.allocate_numbered(
          &"while".to_string()
        );

        let end_label = self.label_allocator.allocate_numbered(
          &"while_end".to_string()
        );

        self.loop_labels.push(LoopLabels {
          continue_: start_label.clone(),
          break_: end_label.clone(),
        });

        self.definition.push(
          std::format!("{}:", start_label)
        );

        let mut expression_compiler = ExpressionCompiler {
          fnc: self,
          scope: scope,
        };

        let condition = expression_compiler.compile(&*while_.test, None);

        for reg in condition.nested_registers {
          self.reg_allocator.release(&reg);
        }

        let cond_reg = self.reg_allocator.allocate_numbered(&"_cond".to_string());

        // TODO: Add negated jmpif instruction to avoid this
        self.definition.push(std::format!(
          "  op! {} %{}",
          condition.value_assembly,
          cond_reg,
        ));

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
      },
      DoWhile(do_while) => {
        let start_label = self.label_allocator.allocate_numbered(
          &"do_while".to_string()
        );

        let continue_label = self.label_allocator.allocate_numbered(
          &"do_while_continue".to_string()
        );

        let end_label = self.label_allocator.allocate_numbered(
          &"do_while_end".to_string()
        );

        self.loop_labels.push(LoopLabels {
          continue_: continue_label.clone(),
          break_: end_label.clone(),
        });

        self.definition.push(
          std::format!("{}:", start_label)
        );

        self.statement(&*do_while.body, false, scope);
        
        let mut expression_compiler = ExpressionCompiler {
          fnc: self,
          scope: scope,
        };

        let condition = expression_compiler.compile(&*do_while.test, None);

        for reg in condition.nested_registers {
          self.reg_allocator.release(&reg);
        }

        self.definition.push(
          format!("{}:", continue_label)
        );

        let mut jmpif_instr = "  jmpif ".to_string();
        jmpif_instr += &condition.value_assembly;
        jmpif_instr += " :";
        jmpif_instr += &start_label;
        self.definition.push(jmpif_instr);

        self.definition.push(
          format!("{}:", end_label)
        );

        self.loop_labels.pop();
      },
      For(for_) => {
        let for_scope = scope.nest();

        match &for_.init {
          Some(swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl)) => {
            self.populate_block_scope_var_decl(var_decl, &for_scope);
          },
          _ => {},
        }

        match &for_.init {
          Some(var_decl_or_expr) => match var_decl_or_expr {
            swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl) => {
              self.var_declaration(var_decl, &for_scope);
            },
            swc_ecma_ast::VarDeclOrExpr::Expr(expr) => {
              self.expression(expr, &for_scope);
            },
          },
          None => {},
        }

        let for_test_label = self.label_allocator.allocate_numbered(
          &"for_test".to_string()
        );

        let for_continue_label = self.label_allocator.allocate_numbered(
          &"for_continue".to_string()
        );

        let for_end_label = self.label_allocator.allocate_numbered(
          &"for_end".to_string()
        );

        self.definition.push(
          format!("{}:", &for_test_label)
        );

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
    
            for reg in condition.nested_registers {
              self.reg_allocator.release(&reg);
            }

            let cond_reg = self.reg_allocator.allocate_numbered(&"_cond".to_string());

            // TODO: Add negated jmpif instruction to avoid this
            self.definition.push(std::format!(
              "  op! {} %{}",
              condition.value_assembly,
              cond_reg,
            ));

            let mut jmpif_instr = "  jmpif %".to_string();
            jmpif_instr += &cond_reg;
            jmpif_instr += " :";
            jmpif_instr += &for_end_label;
            self.definition.push(jmpif_instr);

            self.reg_allocator.release(&cond_reg);
          },
          None => {},
        }

        self.statement(&for_.body, false, &for_scope);

        self.definition.push(
          format!("{}:", for_continue_label)
        );

        match &for_.update {
          Some(update) => self.expression(update, &for_scope),
          None => {},
        }

        self.definition.push(
          format!("  jmp :{}", for_test_label)
        );

        self.definition.push(
          format!("{}:", for_end_label)
        );

        self.loop_labels.pop();
      },
      ForIn(_) => std::panic!("Not implemented: ForIn statement"),
      ForOf(_) => std::panic!("Not implemented: ForOf statement"),
      Decl(decl) => {
        self.declaration(decl, scope);
      },
      Expr(expr) => {
        self.expression(&expr.expr, scope);
      },
    }
  }

  fn declaration(
    &mut self,
    decl: &swc_ecma_ast::Decl,
    scope: &Scope,
  ) {
    use swc_ecma_ast::Decl::*;

    match decl {
      Class(_) => std::panic!("Not implemented: Class declaration"),
      Fn(_) => {},
      Var(var_decl) => self.var_declaration(var_decl, scope),
      TsInterface(_) => std::panic!("Not implemented: TsInterface declaration"),
      TsTypeAlias(_) => {},
      TsEnum(_) => std::panic!("Not implemented: TsEnum declaration"),
      TsModule(_) => std::panic!("Not implemented: TsModule declaration"),
    };
  }

  fn var_declaration(
    &mut self,
    var_decl: &swc_ecma_ast::VarDecl,
    scope: &Scope,
  ) {
    for decl in &var_decl.decls {
      match &decl.init {
        Some(expr) => {
          let mut expr_compiler = ExpressionCompiler {
            fnc: self,
            scope: scope,
          };

          let name = match &decl.name {
            swc_ecma_ast::Pat::Ident(ident) => ident.id.sym.to_string(),
            _ => std::panic!("Not implemented: destructuring"),
          };

          let target_register = match scope.get(&name) {
            Some(MappedName::Register(reg_name)) => reg_name,
            _ => std::panic!("var decl should always get mapped to a register during scan"),
          };

          expr_compiler.compile(expr, Some(target_register));
        },
        None => {},
      }
    }
  }

  fn expression(
    &mut self,
    expr: &swc_ecma_ast::Expr,
    scope: &Scope,
  ) {
    let mut expression_compiler = ExpressionCompiler {
      fnc: self,
      scope: scope,
    };

    let compiled = expression_compiler.compile(
      expr,

      // FIXME: Specify the ignore register instead
      None,
    );

    for reg in compiled.nested_registers {
      self.reg_allocator.release(&reg);
    }
  }
}
