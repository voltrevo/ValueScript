use std::rc::Rc;
use std::cell::RefCell;
use queues::*;

use super::name_allocator::NameAllocator;
use super::expression_compiler::ExpressionCompiler;
use super::scope::{Scope, MappedName, ScopeTrait};

#[derive(Clone)]
struct QueuedFunction {
  definition_name: String,
  fn_name: Option<String>,
  extra_params: Vec<String>,
  function: swc_ecma_ast::Function,
}

pub struct FunctionCompiler {
  definition: Vec<String>,
  definition_allocator: Rc<RefCell<NameAllocator>>,
  reg_allocator: NameAllocator,
  label_allocator: NameAllocator,
  queue: Queue<QueuedFunction>,
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
      extra_params: Vec::new(),
      function: fn_.clone(),
    }).expect("Failed to queue function");

    loop {
      match self_.queue.remove() {
        Ok(qfn) => self_.compile_fn(
          qfn.definition_name,
          qfn.fn_name,
          qfn.extra_params,
          &qfn.function,
          parent_scope,
        ),
        Err(_) => { break; },
      }
    }

    return self_.definition;
  }

  fn compile_fn(
    &mut self,
    definition_name: String,
    fn_name: Option<String>,
    mut extra_params: Vec<String>,
    fn_: &swc_ecma_ast::Function,
    parent_scope: &Scope,
  ) {
    let scope = parent_scope.nest();

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
    params.append(&mut extra_params);

    for p in &fn_.params {
      match &p.pat {
        swc_ecma_ast::Pat::Ident(binding_ident) => {
          let param_name = binding_ident.id.sym.to_string();
          params.push(param_name);
        },
        _ => std::panic!("Not implemented: parameter destructuring"),
      }
    }

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

    let body = fn_.body.as_ref()
      .expect("Not implemented: function without body");
    
    self.populate_fn_scope(body, &scope);
    self.populate_block_scope(body, &scope);

    for i in 0..body.stmts.len() {
      self.statement(
        &body.stmts[i],
        i == body.stmts.len() - 1,
        &scope,
      );
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
            Fn(fn_) => {
              let fn_name = fn_.ident.sym.to_string();

              let definition_name = self
                .definition_allocator
                .borrow_mut()
                .allocate(&fn_name)
              ;

              scope.set(
                fn_name.clone(),
                MappedName::Definition(definition_name.clone()),
              );

              self.queue.add(QueuedFunction {
                definition_name: definition_name,
                fn_name: Some(fn_name),
                extra_params: Vec::new(),
                function: fn_.function.clone(),
              }).expect("Failed to queue function");
            },
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
            definition: &mut self.definition,
            scope: scope,
            reg_allocator: &mut self.reg_allocator,
          };

          expression_compiler.compile(expr, Some("return".to_string()));

          if !fn_last {
            self.definition.push("  end".to_string());
          }
        },
      },

      Labeled(_) => std::panic!("Not implemented: Labeled statement"),
      Break(_) => std::panic!("Not implemented: Break statement"),
      Continue(_) => std::panic!("Not implemented: Continue statement"),
      If(if_) => {
        let mut expression_compiler = ExpressionCompiler {
          definition: &mut self.definition,
          scope: scope,
          reg_allocator: &mut self.reg_allocator,
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

        self.definition.push(
          std::format!("{}:", start_label)
        );

        let mut expression_compiler = ExpressionCompiler {
          definition: &mut self.definition,
          scope: scope,
          reg_allocator: &mut self.reg_allocator,
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

        let end_label = self.label_allocator.allocate_numbered(&"while_end".to_string());

        let mut jmpif_instr = "  jmpif %".to_string();
        jmpif_instr += &cond_reg;
        jmpif_instr += " :";
        jmpif_instr += &end_label;
        self.definition.push(jmpif_instr);

        self.reg_allocator.release(&cond_reg);

        self.statement(&*while_.body, false, scope);
        self.definition.push(std::format!("  jmp :{}", start_label));

        self.definition.push(std::format!("{}:", end_label));
      },
      DoWhile(do_while) => {
        let start_label = self.label_allocator.allocate_numbered(
          &"do_while".to_string()
        );

        self.definition.push(
          std::format!("{}:", start_label)
        );

        self.statement(&*do_while.body, false, scope);
        
        let mut expression_compiler = ExpressionCompiler {
          definition: &mut self.definition,
          scope: scope,
          reg_allocator: &mut self.reg_allocator,
        };

        let condition = expression_compiler.compile(&*do_while.test, None);

        for reg in condition.nested_registers {
          self.reg_allocator.release(&reg);
        }

        let mut jmpif_instr = "  jmpif ".to_string();
        jmpif_instr += &condition.value_assembly;
        jmpif_instr += " :";
        jmpif_instr += &start_label;
        self.definition.push(jmpif_instr);
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

        self.definition.push(
          format!("{}:", &for_test_label)
        );

        let for_end_label = self.label_allocator.allocate_numbered(
          &"for_end".to_string()
        );

        match &for_.test {
          Some(cond) => {
            let mut ec = ExpressionCompiler {
              definition: &mut self.definition,
              scope: &for_scope,
              reg_allocator: &mut self.reg_allocator,
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
      TsTypeAlias(_) => std::panic!("Not implemented: TsTypeAlias declaration"),
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
            definition: &mut self.definition,
            scope: scope,
            reg_allocator: &mut self.reg_allocator,
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
      definition: &mut self.definition,
      scope: scope,
      reg_allocator: &mut self.reg_allocator,
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
