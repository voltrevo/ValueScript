use std::collections::HashSet;

use crate::{asm::Pointer, scope::scope_reg};

use super::scope::{MappedName, Scope};

pub struct CaptureFinder {
  outside_scope: Scope,
  pub ordered_names: Vec<String>,
  names: HashSet<String>,
}

impl CaptureFinder {
  pub fn new(outside_scope: Scope) -> CaptureFinder {
    return CaptureFinder {
      outside_scope: outside_scope,
      ordered_names: Default::default(),
      names: Default::default(),
    };
  }

  fn ref_(&mut self, scope: &Scope, name: String) {
    if name == "undefined" {
      return;
    }

    if scope.get(&name).is_some() {
      return;
    }

    let mut insert = |n: &String| {
      let inserted = self.names.insert(n.clone());

      if inserted {
        self.ordered_names.push(n.clone());
      }
    };

    match self.outside_scope.get(&name) {
      None => std::panic!("Unresolved name {}", name),
      Some(MappedName::Definition(_)) => {} // Not capture - just definition
      Some(MappedName::Register(_)) => insert(&name),
      Some(MappedName::QueuedFunction(qfn)) => {
        for cap in &qfn.capture_params {
          if scope.get(cap).is_some() {
            std::panic!("Not implemented: Nested capture edge case");
          }

          insert(cap);
        }
      }
      Some(MappedName::Builtin(_)) => {}
      Some(MappedName::Constant(_)) => {}
    }
  }

  pub fn fn_decl(&mut self, parent_scope: &Scope, decl: &swc_ecma_ast::FnDecl) {
    let scope = parent_scope.nest();

    scope.set(decl.ident.sym.to_string(), scope_reg("".to_string()));

    self.function(&scope, &decl.function);
  }

  pub fn fn_expr(&mut self, parent_scope: &Scope, expr: &swc_ecma_ast::FnExpr) {
    let scope = parent_scope.nest();

    for ident in &expr.ident {
      scope.set(ident.sym.to_string(), scope_reg("".to_string()));
    }

    self.function(&scope, &expr.function);
  }

  pub fn arrow_expr(&mut self, parent_scope: &Scope, arrow: &swc_ecma_ast::ArrowExpr) {
    let scope = parent_scope.nest();

    for param in &arrow.params {
      match &param {
        swc_ecma_ast::Pat::Ident(ident) => {
          scope.set(ident.id.sym.to_string(), scope_reg("".to_string()))
        }
        _ => std::panic!("Not implemented: destructuring"),
      }
    }

    match &arrow.body {
      swc_ecma_ast::BlockStmtOrExpr::BlockStmt(block_stmt) => {
        self.populate_fn_scope(&scope, block_stmt);
        self.block(&scope, block_stmt);
      }
      swc_ecma_ast::BlockStmtOrExpr::Expr(expr) => {
        self.expr(&scope, expr);
      }
    }
  }

  fn function(&mut self, scope: &Scope, fn_: &swc_ecma_ast::Function) {
    for param in &fn_.params {
      match &param.pat {
        swc_ecma_ast::Pat::Ident(ident) => {
          scope.set(ident.id.sym.to_string(), scope_reg("".to_string()))
        }
        _ => std::panic!("Not implemented: destructuring"),
      }
    }

    for block_stmt in &fn_.body {
      self.populate_fn_scope(scope, block_stmt);
      self.block(scope, block_stmt);
    }
  }

  fn populate_fn_scope(&mut self, scope: &Scope, block: &swc_ecma_ast::BlockStmt) {
    for statement in &block.stmts {
      self.populate_fn_scope_statement(scope, statement);
    }
  }

  fn populate_fn_scope_statement(&mut self, scope: &Scope, statement: &swc_ecma_ast::Stmt) {
    use swc_ecma_ast::Stmt::*;

    match statement {
      Block(nested_block) => {
        self.populate_fn_scope(scope, nested_block);
      }
      Empty(_) => {}
      Debugger(_) => {}
      With(_) => std::panic!("Not supported: With statement"),
      Return(_) => {}
      Labeled(_) => std::panic!("Not implemented: Labeled statement"),
      Break(_) => {}
      Continue(_) => {}
      If(if_) => {
        self.populate_fn_scope_statement(scope, &if_.cons);

        for stmt in &if_.alt {
          self.populate_fn_scope_statement(scope, stmt);
        }
      }
      Switch(_) => std::panic!("Not implemented: Switch statement"),
      Throw(_) => {}
      Try(_) => std::panic!("Not implemented: Try statement"),
      While(while_) => {
        self.populate_fn_scope_statement(scope, &while_.body);
      }
      DoWhile(do_while) => {
        self.populate_fn_scope_statement(scope, &do_while.body);
      }
      For(for_) => {
        match &for_.init {
          Some(swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl)) => {
            self.populate_fn_scope_var_decl(var_decl, scope);
          }
          _ => {}
        };

        self.populate_fn_scope_statement(scope, &for_.body);
      }
      ForIn(_) => std::panic!("Not implemented: ForIn statement"),
      ForOf(_) => std::panic!("Not implemented: ForOf statement"),
      Decl(decl) => {
        use swc_ecma_ast::Decl::*;

        match decl {
          Class(_) => std::panic!("Not implemented: Class declaration"),
          Fn(_) => {}
          Var(var_decl) => self.populate_fn_scope_var_decl(var_decl, scope),
          TsInterface(_) => {}
          TsTypeAlias(_) => {}
          TsEnum(_) => std::panic!("Not implemented: TsEnum declaration"),
          TsModule(_) => std::panic!("Not implemented: TsModule declaration"),
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

          scope.set(name.clone(), scope_reg("".to_string()));
        }
        _ => std::panic!("Not implemented: destructuring"),
      }
    }
  }

  fn populate_block_scope(&mut self, scope: &Scope, block: &swc_ecma_ast::BlockStmt) {
    for statement in &block.stmts {
      use swc_ecma_ast::Stmt::*;

      match statement {
        Block(_) => {}
        Empty(_) => {}
        Debugger(_) => {}
        With(_) => std::panic!("Not supported: With statement"),
        Return(_) => {}
        Labeled(_) => std::panic!("Not implemented: Labeled statement"),
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
            Class(_) => std::panic!("Not implemented: Class declaration"),
            Fn(fn_) => {
              let fn_name = fn_.ident.sym.to_string();

              scope.set(
                fn_name.clone(),
                MappedName::Definition(Pointer {
                  name: "".to_string(),
                }),
              );
            }
            Var(var_decl) => self.populate_block_scope_var_decl(scope, var_decl),
            TsInterface(_) => {}
            TsTypeAlias(_) => {}
            TsEnum(_) => std::panic!("Not implemented: TsEnum declaration"),
            TsModule(_) => {}
          }
        }
        Expr(_) => {}
      };
    }
  }

  fn populate_block_scope_var_decl(&mut self, scope: &Scope, var_decl: &swc_ecma_ast::VarDecl) {
    if var_decl.kind == swc_ecma_ast::VarDeclKind::Var {
      return;
    }

    for decl in &var_decl.decls {
      match &decl.name {
        swc_ecma_ast::Pat::Ident(ident) => {
          let name = ident.id.sym.to_string();

          scope.set(name.clone(), scope_reg("".to_string()));
        }
        _ => std::panic!("Not implemented: destructuring"),
      }
    }
  }

  fn block(&mut self, parent_scope: &Scope, block_stmt: &swc_ecma_ast::BlockStmt) {
    let scope = parent_scope.nest();
    self.populate_block_scope(&scope, block_stmt);

    for statement in &block_stmt.stmts {
      self.statement(&scope, statement);
    }
  }

  fn statement(&mut self, scope: &Scope, statement: &swc_ecma_ast::Stmt) {
    use swc_ecma_ast::Stmt::*;

    match statement {
      Block(block) => self.block(scope, block),
      Empty(_) => {}
      Debugger(_) => {}
      With(_) => std::panic!("Not supported: With statement"),
      Return(return_) => {
        for arg in &return_.arg {
          self.expr(scope, arg);
        }
      }
      Labeled(_) => std::panic!("Not implemented: Labeled statement"),
      Break(_) => {}
      Continue(_) => {}
      If(if_) => {
        self.expr(scope, &if_.test);
        self.statement(scope, &if_.cons);

        for alt in &if_.alt {
          self.statement(scope, alt);
        }
      }
      Switch(_) => std::panic!("Not implemented: Switch statement"),
      Throw(throw) => {
        self.expr(scope, &throw.arg);
      }
      Try(_) => std::panic!("Not implemented: Try statement"),
      While(while_) => {
        self.expr(scope, &while_.test);
        self.statement(scope, &while_.body);
      }
      DoWhile(do_while) => {
        self.statement(scope, &do_while.body);
        self.expr(scope, &do_while.test);
      }
      For(for_) => {
        let for_scope = scope.nest();

        match &for_.init {
          None => {}
          Some(swc_ecma_ast::VarDeclOrExpr::Expr(expr)) => self.expr(&for_scope, expr),
          Some(swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl)) => {
            self.var_decl(&for_scope, var_decl)
          }
        }

        for test in &for_.test {
          self.expr(&for_scope, test);
        }

        self.statement(&for_scope, &for_.body);
      }
      ForIn(_) => std::panic!("Not implemented: ForIn statement"),
      ForOf(_) => std::panic!("Not implemented: ForOf statement"),
      Decl(decl) => {
        use swc_ecma_ast::Decl::*;

        match decl {
          Class(_) => std::panic!("Not implemented: Class declaration"),
          Fn(fn_) => self.fn_decl(scope, fn_),
          Var(var_decl) => self.var_decl(scope, var_decl),
          TsInterface(_) => {}
          TsTypeAlias(_) => {}
          TsEnum(_) => std::panic!("Not implemented: TsEnum declaration"),
          TsModule(_) => {}
        }
      }
      Expr(expr) => self.expr(scope, &expr.expr),
    }
  }

  fn expr(&mut self, scope: &Scope, expr: &swc_ecma_ast::Expr) {
    use swc_ecma_ast::Expr::*;

    match expr {
      This(_) => {}
      Array(array_exp) => {
        for option_elem in &array_exp.elems {
          for elem in option_elem {
            self.expr(scope, &elem.expr);
          }
        }
      }
      Object(object_exp) => {
        for prop in &object_exp.props {
          match prop {
            swc_ecma_ast::PropOrSpread::Spread(spread) => {
              self.expr(scope, &spread.expr);
            }
            swc_ecma_ast::PropOrSpread::Prop(p) => {
              use swc_ecma_ast::Prop::*;

              match &**p {
                Shorthand(ident) => {
                  self.ref_(scope, ident.sym.to_string());
                }
                KeyValue(kv) => {
                  match &kv.key {
                    swc_ecma_ast::PropName::Ident(_) => {}
                    swc_ecma_ast::PropName::Str(_) => {}
                    swc_ecma_ast::PropName::Num(_) => {}
                    swc_ecma_ast::PropName::Computed(comp) => {
                      self.expr(scope, &comp.expr);
                    }
                    swc_ecma_ast::PropName::BigInt(_) => {}
                  }

                  self.expr(scope, &kv.value);
                }
                Assign(_) => std::panic!("Not implemented: Assign prop"),
                Getter(_) => std::panic!("Not implemented: Getter prop"),
                Setter(_) => std::panic!("Not implemented: Setter prop"),
                Method(_) => std::panic!("Not implemented: Method prop"),
              }
            }
          }
        }
      }
      Fn(fn_) => self.fn_expr(scope, fn_),
      Unary(un_exp) => self.expr(scope, &un_exp.arg),
      Update(update_exp) => self.expr(scope, &update_exp.arg),
      Bin(bin_exp) => {
        self.expr(scope, &bin_exp.left);
        self.expr(scope, &bin_exp.right);
      }
      Assign(assign_exp) => {
        match &assign_exp.left {
          swc_ecma_ast::PatOrExpr::Pat(pat) => match &**pat {
            swc_ecma_ast::Pat::Ident(ident) => self.ref_(scope, ident.id.sym.to_string()),
            swc_ecma_ast::Pat::Expr(expr) => self.expr(scope, expr),
            _ => std::panic!("Not implemented: destructuring"),
          },
          swc_ecma_ast::PatOrExpr::Expr(expr) => self.expr(scope, expr),
        }

        self.expr(scope, &assign_exp.right);
      }
      Member(member_exp) => {
        self.expr(scope, &member_exp.obj);

        match &member_exp.prop {
          swc_ecma_ast::MemberProp::Ident(_) => {}
          swc_ecma_ast::MemberProp::Computed(computed) => {
            self.expr(scope, &computed.expr);
          }
          swc_ecma_ast::MemberProp::PrivateName(_) => {
            std::panic!("Not implemented: private name");
          }
        }
      }
      SuperProp(_) => std::panic!("Not implemented: SuperProp expression"),
      Cond(_) => std::panic!("Not implemented: Cond expression"),
      Call(call_exp) => {
        match &call_exp.callee {
          swc_ecma_ast::Callee::Expr(expr) => self.expr(scope, expr),
          _ => std::panic!("Not implemented: non-expression callee"),
        };

        for arg in &call_exp.args {
          self.expr(scope, &arg.expr);
        }
      }
      New(_) => std::panic!("Not implemented: New expression"),
      Seq(_) => std::panic!("Not implemented: Seq expression"),
      Ident(ident) => self.ref_(scope, ident.sym.to_string()),
      Lit(_) => {}
      Tpl(_) => std::panic!("Not implemented: Tpl expression"),
      TaggedTpl(_) => std::panic!("Not implemented: TaggedTpl expression"),
      Arrow(_) => std::panic!("Not implemented: Arrow expression"),
      Class(_) => std::panic!("Not implemented: Class expression"),
      Yield(_) => std::panic!("Not implemented: Yield expression"),
      MetaProp(_) => std::panic!("Not implemented: MetaProp expression"),
      Await(_) => std::panic!("Not implemented: Await expression"),
      Paren(p) => self.expr(scope, &p.expr),
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

  fn var_decl(&mut self, scope: &Scope, var_decl: &swc_ecma_ast::VarDecl) {
    for decl in &var_decl.decls {
      for init in &decl.init {
        self.expr(scope, init);
      }
    }
  }
}
