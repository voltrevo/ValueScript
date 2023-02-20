use std::{cell::RefCell, collections::HashMap, collections::HashSet, rc::Rc};

use super::diagnostic::{Diagnostic, DiagnosticLevel};
use super::scope::Builtin;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum NameId {
  Span(swc_common::Span),
  Builtin(Builtin),
}

#[derive(Clone)]
pub struct Capture {
  ref_: swc_common::Span,
  captor_id: OwnerId,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NameType {
  Var,
  Let,
  Const,
  Param,
  Function,
  Class,
  Import,
  Builtin,
}

#[derive(Clone)]
pub struct Name {
  id: NameId,
  owner_id: OwnerId,
  sym: swc_atoms::JsWord,
  type_: NameType,
  mutations: Vec<swc_common::Span>,
  captures: Vec<Capture>,
}

#[derive(Default)]
pub struct ScopeAnalysis {
  pub names: HashMap<NameId, Name>,
  pub captures: HashMap<OwnerId, HashSet<swc_common::Span>>,
  pub diagnostics: Vec<Diagnostic>,
}

impl ScopeAnalysis {
  pub fn run(module: &swc_ecma_ast::Module) -> ScopeAnalysis {
    let mut sa = ScopeAnalysis::default();
    let scope = init_std_scope();

    for builtin in vec![Builtin::Debug, Builtin::Math, Builtin::undefined] {
      sa.names.insert(
        NameId::Builtin(builtin),
        Name {
          id: NameId::Builtin(builtin),
          owner_id: OwnerId::Module,
          sym: swc_atoms::JsWord::from(format!("{}", builtin)),
          type_: NameType::Builtin,
          mutations: vec![],
          captures: vec![],
        },
      );
    }

    sa.module_level_hoists(&scope, module);

    for module_item in &module.body {
      sa.module_item(&scope, module_item);
    }

    for (name_id, name) in &sa.names {
      if name.captures.len() > 0 {
        if name.type_ == NameType::Let {
          match name_id {
            NameId::Span(span) => {
              sa.diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Lint,
                message: format!(
                  "`{}` should be declared using `const` because it is implicitly \
                  const due to capture",
                  name.sym
                ),
                span: *span,
              });
            }
            NameId::Builtin(_) => {
              sa.diagnostics.push(Diagnostic {
                level: DiagnosticLevel::InternalError,
                message: "Builtin should not have type_ let".to_string(),
                span: swc_common::DUMMY_SP,
              });
            }
          }
        }

        for mutation in &name.mutations {
          sa.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            message: format!("Cannot mutate captured variable `{}`", name.sym),
            span: *mutation,
          });
        }
      }
    }

    return sa;
  }

  fn insert_name(&mut self, scope: &XScope, type_: NameType, origin_ident: &swc_ecma_ast::Ident) {
    let name = Name {
      id: NameId::Span(origin_ident.span),
      owner_id: scope.borrow().owner_id.clone(),
      sym: origin_ident.sym.clone(),
      type_,
      mutations: Vec::new(),
      captures: Vec::new(),
    };

    self.names.insert(name.id.clone(), name.clone());

    scope.set(
      &origin_ident.sym,
      name.id.clone(),
      origin_ident.span,
      &mut self.diagnostics,
    );
  }

  fn insert_capture(&mut self, captor_id: &OwnerId, name_id: &NameId, ref_: swc_common::Span) {
    let name = match self.names.get_mut(name_id) {
      Some(name) => name,
      None => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: format!("Expected name_id in names: {:?}", name_id),
          span: ref_,
        });
        return;
      }
    };

    self
      .captures
      .entry(captor_id.clone())
      .or_insert_with(HashSet::new)
      .insert(ref_);

    name.captures.push(Capture {
      ref_,
      captor_id: captor_id.clone(),
    });
  }

  fn module_item(&mut self, scope: &XScope, module_item: &swc_ecma_ast::ModuleItem) {
    use swc_ecma_ast::ModuleDecl;
    use swc_ecma_ast::ModuleItem;

    match module_item {
      ModuleItem::ModuleDecl(module_decl) => match module_decl {
        ModuleDecl::Import(_) => {}
        ModuleDecl::ExportDecl(ed) => {
          self.decl(&scope, &ed.decl);
        }
        ModuleDecl::ExportNamed(_) => {}
        ModuleDecl::ExportDefaultDecl(edd) => {
          self.default_decl(&scope, &edd.decl);
        }
        ModuleDecl::ExportDefaultExpr(ede) => {
          self.expr(&scope, &ede.expr);
        }
        ModuleDecl::ExportAll(_) => {}
        ModuleDecl::TsImportEquals(ts_import_equals) => {
          self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            message: "TsImportEquals is not supported".to_string(),
            span: ts_import_equals.span,
          });
        }
        ModuleDecl::TsExportAssignment(ts_export_assignment) => {
          self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            message: "TsExportAssignment is not supported".to_string(),
            span: ts_export_assignment.span,
          });
        }
        ModuleDecl::TsNamespaceExport(ts_namespace_export) => {
          self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            message: "TsNamespaceExport is not supported".to_string(),
            span: ts_namespace_export.span,
          });
        }
      },
      ModuleItem::Stmt(stmt) => {
        self.stmt(&scope, &stmt);
      }
    };
  }

  fn import_decl(&mut self, scope: &XScope, import_decl: &swc_ecma_ast::ImportDecl) {
    for specifier in &import_decl.specifiers {
      self.import_specifier(&scope, specifier);
    }
  }

  fn import_specifier(&mut self, scope: &XScope, import_specifier: &swc_ecma_ast::ImportSpecifier) {
    use swc_ecma_ast::ImportSpecifier::*;

    match import_specifier {
      Named(named_specifier) => {
        if named_specifier.is_type_only {
          return;
        }

        self.insert_name(scope, NameType::Import, &named_specifier.local);
      }
      Default(default_specifier) => {
        self.insert_name(scope, NameType::Import, &default_specifier.local);
      }
      Namespace(namespace_specifier) => {
        self.insert_name(scope, NameType::Import, &namespace_specifier.local);
      }
    }
  }

  fn decl(&mut self, scope: &XScope, decl: &swc_ecma_ast::Decl) {
    use swc_ecma_ast::Decl;

    match decl {
      Decl::Class(class_decl) => {
        self.class_(scope, &Some(class_decl.ident.clone()), &class_decl.class);
      }
      Decl::Fn(fn_decl) => {
        self.function(scope, &Some(fn_decl.ident.clone()), &fn_decl.function);
      }
      Decl::Var(var_decl) => {
        for decl in &var_decl.decls {
          self.var_declarator(&scope, var_decl.kind, decl);
        }
      }
      Decl::TsInterface(_) => {}
      Decl::TsTypeAlias(_) => {}
      Decl::TsEnum(ts_enum) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO: Implement TsEnum declarations".to_string(),
          span: ts_enum.span,
        });
      }
      Decl::TsModule(ts_module) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "TsModule declaration is not supported".to_string(),
          span: ts_module.span,
        });
      }
    }
  }

  fn function(
    &mut self,
    scope: &XScope,
    name: &Option<swc_ecma_ast::Ident>,
    function: &swc_ecma_ast::Function,
  ) {
    let child_scope = scope.nest(Some(OwnerId::Span(function.span.clone())));

    if let Some(name) = name {
      self.insert_name(&child_scope, NameType::Function, name);
    }

    for param in &function.params {
      self.param_pat(&child_scope, &param.pat);
    }

    for body in &function.body {
      self.function_level_hoists(&child_scope, &body);
      self.block_stmt(&child_scope, &body);
    }
  }

  fn module_level_hoists(&mut self, scope: &XScope, module: &swc_ecma_ast::Module) {
    for item in &module.body {
      self.module_level_hoists_item(scope, item);
    }
  }

  fn module_level_hoists_item(&mut self, scope: &XScope, module_item: &swc_ecma_ast::ModuleItem) {
    use swc_ecma_ast::ModuleDecl;
    use swc_ecma_ast::ModuleItem;

    match module_item {
      ModuleItem::ModuleDecl(module_decl) => match module_decl {
        ModuleDecl::Import(import_decl) => {
          self.import_decl(&scope, import_decl);
        }
        ModuleDecl::ExportDecl(ed) => match &ed.decl {
          swc_ecma_ast::Decl::Class(class_decl) => {
            self.insert_name(&scope, NameType::Class, &class_decl.ident);
          }
          swc_ecma_ast::Decl::Fn(fn_decl) => {
            self.insert_name(&scope, NameType::Function, &fn_decl.ident);
          }
          swc_ecma_ast::Decl::Var(var_decl) => {
            let name_type = match var_decl.kind {
              swc_ecma_ast::VarDeclKind::Const => NameType::Const,
              swc_ecma_ast::VarDeclKind::Let => NameType::Let,
              swc_ecma_ast::VarDeclKind::Var => NameType::Var,
            };

            for decl in &var_decl.decls {
              for ident in self.get_pat_idents(&decl.name) {
                self.insert_name(&scope, name_type, &ident);
              }
            }
          }
          swc_ecma_ast::Decl::TsInterface(_) => {}
          swc_ecma_ast::Decl::TsTypeAlias(_) => {}
          swc_ecma_ast::Decl::TsEnum(_) => {
            // Diagnostic emitted after hoist processing
          }
          swc_ecma_ast::Decl::TsModule(_) => {
            // Diagnostic emitted after hoist processing
          }
        },
        ModuleDecl::ExportNamed(_) => {}
        ModuleDecl::ExportDefaultDecl(edd) => match &edd.decl {
          swc_ecma_ast::DefaultDecl::Class(class_decl) => {
            if let Some(ident) = &class_decl.ident {
              self.insert_name(&scope, NameType::Class, ident);
            }
          }
          swc_ecma_ast::DefaultDecl::Fn(fn_decl) => {
            if let Some(ident) = &fn_decl.ident {
              self.insert_name(&scope, NameType::Function, ident);
            }
          }
          swc_ecma_ast::DefaultDecl::TsInterfaceDecl(_) => {}
        },
        ModuleDecl::ExportDefaultExpr(_) => {}
        ModuleDecl::ExportAll(_) => {}
        ModuleDecl::TsImportEquals(_) => {
          // Diagnostic emitted after hoist processing
        }
        ModuleDecl::TsExportAssignment(_) => {
          // Diagnostic emitted after hoist processing
        }
        ModuleDecl::TsNamespaceExport(_) => {
          // Diagnostic emitted after hoist processing
        }
      },
      ModuleItem::Stmt(stmt) => {
        self.function_level_hoists_stmt(&scope, stmt);
        self.block_level_hoists_stmt(&scope, stmt);
      }
    };
  }

  fn function_level_hoists(&mut self, scope: &XScope, block: &swc_ecma_ast::BlockStmt) {
    for stmt in &block.stmts {
      self.function_level_hoists_stmt(scope, stmt);
    }
  }

  fn function_level_hoists_stmt(&mut self, scope: &XScope, stmt: &swc_ecma_ast::Stmt) {
    use swc_ecma_ast::Decl;
    use swc_ecma_ast::Stmt;

    match stmt {
      Stmt::Decl(decl) => match decl {
        Decl::Var(var_decl) => {
          if var_decl.kind == swc_ecma_ast::VarDeclKind::Var {
            for decl in &var_decl.decls {
              for ident in self.get_pat_idents(&decl.name) {
                self.insert_name(scope, NameType::Var, &ident);
              }
            }
          }
        }
        _ => {}
      },
      Stmt::Block(block_stmt) => {
        for stmt in &block_stmt.stmts {
          self.function_level_hoists_stmt(scope, stmt);
        }
      }
      Stmt::For(for_) => {
        if let Some(swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl)) = &for_.init {
          if var_decl.kind == swc_ecma_ast::VarDeclKind::Var {
            for decl in &var_decl.decls {
              for ident in self.get_pat_idents(&decl.name) {
                self.insert_name(scope, NameType::Var, &ident);
              }
            }
          }
        }
      }
      Stmt::ForIn(for_in) => {
        if let swc_ecma_ast::VarDeclOrPat::VarDecl(var_decl) = &for_in.left {
          if var_decl.kind == swc_ecma_ast::VarDeclKind::Var {
            for decl in &var_decl.decls {
              for ident in self.get_pat_idents(&decl.name) {
                self.insert_name(scope, NameType::Var, &ident);
              }
            }
          }
        }
      }
      Stmt::ForOf(for_of) => {
        if let swc_ecma_ast::VarDeclOrPat::VarDecl(var_decl) = &for_of.left {
          if var_decl.kind == swc_ecma_ast::VarDeclKind::Var {
            for decl in &var_decl.decls {
              for ident in self.get_pat_idents(&decl.name) {
                self.insert_name(scope, NameType::Var, &ident);
              }
            }
          }
        }
      }
      _ => {}
    }
  }

  fn block_level_hoists(&mut self, scope: &XScope, block: &swc_ecma_ast::BlockStmt) {
    for stmt in &block.stmts {
      self.block_level_hoists_stmt(scope, stmt);
    }
  }

  fn block_level_hoists_stmt(&mut self, scope: &XScope, stmt: &swc_ecma_ast::Stmt) {
    use swc_ecma_ast::Decl;
    use swc_ecma_ast::Stmt;

    match stmt {
      Stmt::Decl(decl) => match decl {
        Decl::Class(class) => {
          self.insert_name(scope, NameType::Class, &class.ident);
        }
        Decl::Fn(fn_) => {
          self.insert_name(scope, NameType::Function, &fn_.ident);
        }
        Decl::Var(var_decl) => {
          self.block_level_hoists_var_decl(scope, var_decl);
        }
        Decl::TsInterface(_) => {}
        Decl::TsTypeAlias(_) => {}
        Decl::TsEnum(_) => {
          // Diagnostic emitted after hoist processing
        }
        Decl::TsModule(_) => {
          // Diagnostic emitted after hoist processing
        }
      },
      _ => {}
    }
  }

  fn block_level_hoists_var_decl(&mut self, scope: &XScope, var_decl: &swc_ecma_ast::VarDecl) {
    let name_type = match var_decl.kind {
      swc_ecma_ast::VarDeclKind::Var => return,
      swc_ecma_ast::VarDeclKind::Let => NameType::Let,
      swc_ecma_ast::VarDeclKind::Const => NameType::Const,
    };

    for decl in &var_decl.decls {
      for ident in self.get_pat_idents(&decl.name) {
        self.insert_name(scope, name_type, &ident);
      }
    }
  }

  fn get_pat_idents(&mut self, pat: &swc_ecma_ast::Pat) -> Vec<swc_ecma_ast::Ident> {
    let mut idents = Vec::new();
    self.get_pat_idents_impl(&mut idents, pat);
    idents
  }

  fn get_pat_idents_impl(
    &mut self,
    idents: &mut Vec<swc_ecma_ast::Ident>,
    pat: &swc_ecma_ast::Pat,
  ) {
    use swc_ecma_ast::Pat;

    match pat {
      Pat::Ident(ident) => {
        idents.push(ident.id.clone());
      }
      Pat::Array(array_pat) => {
        for elem in &array_pat.elems {
          if let Some(elem) = elem {
            self.get_pat_idents_impl(idents, elem);
          }
        }
      }
      Pat::Rest(rest_pat) => {
        self.get_pat_idents_impl(idents, &rest_pat.arg);
      }
      Pat::Object(object_pat) => {
        for prop in &object_pat.props {
          match prop {
            swc_ecma_ast::ObjectPatProp::KeyValue(key_value) => {
              self.get_pat_idents_impl(idents, &key_value.value);
            }
            swc_ecma_ast::ObjectPatProp::Assign(assign) => {
              idents.push(assign.key.clone());
            }
            swc_ecma_ast::ObjectPatProp::Rest(rest) => {
              self.get_pat_idents_impl(idents, &rest.arg);
            }
          }
        }
      }
      Pat::Assign(assign_pat) => {
        self.get_pat_idents_impl(idents, &assign_pat.left);
      }
      Pat::Expr(expr) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "Pattern expression not expected in this context".to_string(),
          span: get_expr_span(expr),
        });
      }
      Pat::Invalid(invalid) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Invalid pattern".to_string(),
          span: invalid.span,
        });
      }
    }
  }

  fn arrow(&mut self, scope: &XScope, arrow: &swc_ecma_ast::ArrowExpr) {
    let child_scope = scope.nest(Some(OwnerId::Span(arrow.span.clone())));

    for param in &arrow.params {
      self.param_pat(&child_scope, param);
    }

    match &arrow.body {
      swc_ecma_ast::BlockStmtOrExpr::BlockStmt(block) => {
        self.block_stmt(&child_scope, block);
      }
      swc_ecma_ast::BlockStmtOrExpr::Expr(expr) => {
        self.expr(&child_scope, expr);
      }
    }
  }

  fn var_declarator_pat(&mut self, scope: &XScope, type_: NameType, pat: &swc_ecma_ast::Pat) {
    use swc_ecma_ast::Pat;

    match pat {
      Pat::Ident(_) => {}
      Pat::Array(array) => {
        for elem in &array.elems {
          if let Some(elem) = elem {
            self.var_declarator_pat(scope, type_, elem);
          }
        }
      }
      Pat::Object(object) => {
        for prop in &object.props {
          match prop {
            swc_ecma_ast::ObjectPatProp::KeyValue(key_value) => {
              self.prop_key(scope, &key_value.key);
              self.var_declarator_pat(scope, type_, &key_value.value);
            }
            swc_ecma_ast::ObjectPatProp::Assign(assign) => {
              if let Some(value) = &assign.value {
                self.expr(scope, value);
              }
            }
            swc_ecma_ast::ObjectPatProp::Rest(rest) => {
              self.var_declarator_pat(scope, type_, &rest.arg);
            }
          }
        }
      }
      Pat::Rest(rest) => {
        self.var_declarator_pat(scope, type_, &rest.arg);
      }
      Pat::Assign(assign) => {
        self.var_declarator_pat(scope, type_, &assign.left);
        self.expr(scope, &assign.right);
      }
      Pat::Invalid(invalid) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Invalid pattern".to_string(),
          span: invalid.span,
        });
      }
      Pat::Expr(expr) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "Pattern expression not expected in declarator".to_string(),
          span: get_expr_span(expr),
        });
      }
    }
  }

  fn var_declarator(
    &mut self,
    scope: &XScope,
    kind: swc_ecma_ast::VarDeclKind,
    var_declarator: &swc_ecma_ast::VarDeclarator,
  ) {
    let type_ = match kind {
      swc_ecma_ast::VarDeclKind::Var => NameType::Var,
      swc_ecma_ast::VarDeclKind::Let => NameType::Let,
      swc_ecma_ast::VarDeclKind::Const => NameType::Const,
    };

    self.var_declarator_pat(scope, type_, &var_declarator.name);

    for init in &var_declarator.init {
      self.expr(scope, init);
    }
  }

  fn default_decl(&mut self, scope: &XScope, default_decl: &swc_ecma_ast::DefaultDecl) {
    use swc_ecma_ast::DefaultDecl;

    match &default_decl {
      DefaultDecl::Class(class_expr) => {
        self.class_(&scope, &class_expr.ident, &class_expr.class);
      }
      DefaultDecl::Fn(fn_expr) => {
        self.fn_expr(&scope, fn_expr);
      }
      DefaultDecl::TsInterfaceDecl(_) => {}
    }
  }

  fn class_(
    &mut self,
    scope: &XScope,
    ident: &Option<swc_ecma_ast::Ident>,
    class_: &swc_ecma_ast::Class,
  ) {
    let child_scope = scope.nest(Some(OwnerId::Span(class_.span)));

    if let Some(ident) = ident {
      self.insert_name(&child_scope, NameType::Class, ident);
    }

    for member in &class_.body {
      self.class_member(&child_scope, member);
    }
  }

  fn class_member(&mut self, scope: &XScope, class_member: &swc_ecma_ast::ClassMember) {
    use swc_ecma_ast::ClassMember::*;

    match class_member {
      Constructor(constructor) => {
        let child_scope = scope.nest(None);

        for param in &constructor.params {
          match param {
            swc_ecma_ast::ParamOrTsParamProp::Param(param) => {
              self.param_pat(&child_scope, &param.pat);
            }
            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(ts_param_prop) => {
              match &ts_param_prop.param {
                swc_ecma_ast::TsParamPropParam::Ident(ident) => {
                  self.insert_name(&child_scope, NameType::Param, &ident.id);
                }
                swc_ecma_ast::TsParamPropParam::Assign(assign) => {
                  self.param_pat(&child_scope, &assign.left);
                  self.expr(&child_scope, &assign.right);
                }
              }
            }
          }
        }

        for body in &constructor.body {
          self.block_stmt(&child_scope, body);
        }
      }
      Method(method) => {
        self.function(scope, &None, &method.function);
      }
      ClassProp(class_prop) => {
        self.prop_key(scope, &class_prop.key);

        if let Some(value) = &class_prop.value {
          self.expr(scope, value);
        }
      }
      TsIndexSignature(_) => {
        // Example: `[key: string]: string`
        // Type only. Nothing to do.
      }
      PrivateMethod(private_method) => {
        // The .key of a private method can only be an identifier, so .prop_key
        // is not needed.

        self.function(scope, &None, &private_method.function);
      }
      PrivateProp(private_prop) => {
        // The .key of a private prop can only be an identifier, so .prop_key
        // is not needed.

        if let Some(value) = &private_prop.value {
          self.expr(scope, value);
        }
      }
      Empty(_) => {}
      StaticBlock(static_block) => {
        self.block_stmt(&scope, &static_block.body);
      }
    }
  }

  fn fn_expr(&mut self, scope: &XScope, fn_expr: &swc_ecma_ast::FnExpr) {
    self.function(scope, &fn_expr.ident, &fn_expr.function);
  }

  fn expr(&mut self, scope: &XScope, expr: &swc_ecma_ast::Expr) {
    use swc_ecma_ast::Expr;

    match expr {
      Expr::This(_) => {}
      Expr::Ident(ident) => {
        self.ident(scope, ident);
      }
      Expr::Lit(_) => {}
      Expr::Array(array) => {
        for elem in &array.elems {
          if let Some(elem) = elem {
            self.expr(scope, &elem.expr);
          }
        }
      }
      Expr::Object(object) => {
        for prop_or_spread in &object.props {
          self.prop_or_spread(scope, prop_or_spread);
        }
      }
      Expr::Fn(fn_expr) => {
        self.fn_expr(scope, fn_expr);
      }
      Expr::Unary(unary) => {
        self.expr(scope, &unary.arg);

        match unary.op {
          swc_ecma_ast::UnaryOp::TypeOf => {}
          swc_ecma_ast::UnaryOp::Void => {}
          swc_ecma_ast::UnaryOp::Delete => {
            self.mutate_expr(scope, &unary.arg);
          }
          swc_ecma_ast::UnaryOp::Plus => {}
          swc_ecma_ast::UnaryOp::Minus => {}
          swc_ecma_ast::UnaryOp::Tilde => {}
          swc_ecma_ast::UnaryOp::Bang => {}
        }
      }
      Expr::Update(update) => {
        self.expr(scope, &update.arg);
        self.mutate_expr(scope, &update.arg);
      }
      Expr::Bin(bin) => {
        self.expr(scope, &bin.left);
        self.expr(scope, &bin.right);
      }
      Expr::Assign(assign) => match &assign.left {
        swc_ecma_ast::PatOrExpr::Pat(pat) => {
          self.mutate_pat(scope, pat);
        }
        swc_ecma_ast::PatOrExpr::Expr(expr) => {
          self.mutate_expr(scope, expr);
        }
      },
      Expr::Seq(seq) => {
        for expr in &seq.exprs {
          self.expr(scope, expr);
        }
      }
      Expr::Cond(cond) => {
        self.expr(scope, &cond.test);
        self.expr(scope, &cond.cons);
        self.expr(scope, &cond.alt);
      }
      Expr::Yield(yield_) => {
        if let Some(arg) = &yield_.arg {
          self.expr(scope, arg);
        }
      }
      Expr::Await(await_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "await is not supported".to_string(),
          span: await_.span,
        });
      }
      Expr::Member(member) => {
        self.expr(scope, &member.obj);
      }
      Expr::Call(call) => {
        match &call.callee {
          swc_ecma_ast::Callee::Super(_) => {}
          swc_ecma_ast::Callee::Import(_) => {}
          swc_ecma_ast::Callee::Expr(expr) => {
            self.expr(scope, expr);
          }
        }

        for arg in &call.args {
          self.expr(scope, &arg.expr);
        }
      }
      Expr::New(new) => {
        self.expr(scope, &new.callee);

        if let Some(args) = &new.args {
          for arg in args {
            self.expr(scope, &arg.expr);
          }
        }
      }
      Expr::Paren(paren) => {
        self.expr(scope, &paren.expr);
      }
      Expr::Tpl(_) => {}
      Expr::TaggedTpl(_) => {}
      Expr::Arrow(arrow) => {
        self.arrow(scope, arrow);
      }
      Expr::Class(class_expr) => {
        self.class_(scope, &class_expr.ident, &class_expr.class);
      }
      Expr::MetaProp(_) => {}
      Expr::Invalid(_) => {}
      Expr::TsTypeAssertion(_) => {}
      Expr::TsConstAssertion(_) => {}
      Expr::TsNonNull(_) => {}
      Expr::TsAs(_) => {}
      Expr::OptChain(_) => {}

      Expr::SuperProp(super_prop) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: super_prop.span,
        });
      }
      Expr::JSXMember(_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: get_expr_span(expr),
        });
      }
      Expr::JSXNamespacedName(_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: get_expr_span(expr),
        });
      }
      Expr::JSXEmpty(_) => {}
      Expr::JSXElement(jsx_element) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: jsx_element.span,
        });
      }
      Expr::JSXFragment(jsx_fragment) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: jsx_fragment.span,
        });
      }
      Expr::TsInstantiation(ts_instantiation) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: ts_instantiation.span,
        });
      }
      Expr::PrivateName(private_name) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: private_name.span,
        });
      }
    }
  }

  fn mutate_expr(&mut self, scope: &XScope, expr: &swc_ecma_ast::Expr) {
    use swc_ecma_ast::Expr;

    match expr {
      Expr::Ident(ident) => {
        self.mutate_ident(scope, ident);
      }
      Expr::Member(member) => {
        self.mutate_expr(scope, &member.obj);
      }
      Expr::Call(call) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Call expressions cannot be mutated".to_string(),
          span: call.span,
        });
      }
      Expr::New(new) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "New expressions cannot be mutated".to_string(),
          span: new.span,
        });
      }
      Expr::Paren(paren) => {
        self.mutate_expr(scope, &paren.expr);
      }
      Expr::Tpl(tpl) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Template literals cannot be mutated".to_string(),
          span: tpl.span,
        });
      }
      Expr::TaggedTpl(tagged_tpl) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Tagged template literals cannot be mutated".to_string(),
          span: tagged_tpl.span,
        });
      }
      Expr::Arrow(arrow) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Arrow functions cannot be mutated".to_string(),
          span: arrow.span,
        });
      }
      Expr::Class(class) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Class expressions cannot be mutated".to_string(),
          span: class.class.span,
        });
      }
      Expr::MetaProp(meta_prop) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Meta properties cannot be mutated".to_string(),
          span: meta_prop.span,
        });
      }
      Expr::Invalid(invalid) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Invalid expression".to_string(),
          span: invalid.span,
        });
      }
      Expr::TsTypeAssertion(ts_type_assertion) => {
        self.mutate_expr(scope, &ts_type_assertion.expr);
      }
      Expr::TsConstAssertion(ts_const_assertion) => {
        self.mutate_expr(scope, &ts_const_assertion.expr);
      }
      Expr::TsNonNull(ts_non_null) => {
        self.mutate_expr(scope, &ts_non_null.expr);
      }
      Expr::TsAs(as_expr) => {
        self.mutate_expr(scope, &as_expr.expr);
      }
      Expr::OptChain(opt_chain) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Optional property accesses (a?.b) cannot be mutated".to_string(),
          span: opt_chain.span,
        });
      }

      Expr::This(_) => {
        // TODO: Add capture+mutation analysis for `this`.
      }
      Expr::Array(array) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "This case is not expected to occur. Expected parser to \
            emit a pattern when mutating an array."
            .to_string(),
          span: array.span,
        });
      }
      Expr::Object(object) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "This case is not expected to occur. Expected parser to \
            emit a pattern when mutating an object."
            .to_string(),
          span: object.span,
        });
      }
      Expr::Fn(fn_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: fn_.function.span,
        });
      }
      Expr::Unary(unary) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: unary.span,
        });
      }
      Expr::Update(update) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: update.span,
        });
      }
      Expr::Bin(bin) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: bin.span,
        });
      }
      Expr::Assign(assign) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: assign.span,
        });
      }
      Expr::SuperProp(super_prop) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: super_prop.span,
        });
      }
      Expr::Cond(cond) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: cond.span,
        });
      }
      Expr::Seq(seq) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: seq.span,
        });
      }
      Expr::Lit(_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: get_expr_span(expr),
        });
      }
      Expr::Yield(yield_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: yield_.span,
        });
      }
      Expr::Await(await_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "await is not supported".to_string(),
          span: await_.span,
        });
      }
      Expr::JSXMember(_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: get_expr_span(expr),
        });
      }
      Expr::JSXNamespacedName(_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: get_expr_span(expr),
        });
      }
      Expr::JSXEmpty(_) => {}
      Expr::JSXElement(jsx_element) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: jsx_element.span,
        });
      }
      Expr::JSXFragment(jsx_fragment) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: jsx_fragment.span,
        });
      }
      Expr::TsInstantiation(ts_instantiation) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: ts_instantiation.span,
        });
      }
      Expr::PrivateName(private_name) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: private_name.span,
        });
      }
    }
  }

  fn mutate_pat(&mut self, scope: &XScope, pat: &swc_ecma_ast::Pat) {
    use swc_ecma_ast::Pat;

    match pat {
      Pat::Ident(ident) => {
        self.mutate_ident(scope, &ident.id);
      }
      Pat::Array(array_pat) => {
        for elem in &array_pat.elems {
          if let Some(elem) = elem {
            self.mutate_pat(scope, elem);
          }
        }
      }
      Pat::Rest(rest_pat) => {
        self.mutate_pat(scope, &rest_pat.arg);
      }
      Pat::Object(object_pat) => {
        for prop in &object_pat.props {
          match prop {
            swc_ecma_ast::ObjectPatProp::KeyValue(key_value) => {
              self.mutate_pat(scope, &key_value.value);
            }
            swc_ecma_ast::ObjectPatProp::Assign(assign) => {
              // TODO: How is `({ y: x = 3 } = { y: 4 })` handled?

              self.mutate_ident(scope, &assign.key);

              if let Some(value) = &assign.value {
                // Note: Generally mutate_* only processes the mutation aspect
                // of an expression, but here, because this only occurs in the
                // context of mutation, we call back into expr. This is
                // consistent with calling into expr from var_declarator_pat and
                // param_pat.
                self.expr(scope, value);
              }
            }
            swc_ecma_ast::ObjectPatProp::Rest(rest) => {
              self.mutate_pat(scope, &rest.arg);
            }
          }
        }
      }
      Pat::Assign(assign_pat) => {
        self.mutate_pat(scope, &assign_pat.left);
        self.expr(scope, &assign_pat.right);
      }
      Pat::Invalid(invalid) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Invalid pattern".to_string(),
          span: invalid.span,
        });
      }
      Pat::Expr(expr) => {
        self.mutate_expr(&scope, expr);
      }
    }
  }

  fn mutate_ident(&mut self, scope: &XScope, ident: &swc_ecma_ast::Ident) {
    let name_id = match scope.get(&ident.sym) {
      Some(name_id) => name_id,
      None => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Unresolved reference".to_string(),
          span: ident.span,
        });
        return;
      }
    };

    let name = match self.names.get_mut(&name_id) {
      Some(name) => name,
      None => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "Expected name_id in names".to_string(),
          span: ident.span,
        });
        return;
      }
    };

    name.mutations.push(ident.span);
  }

  fn ident(&mut self, scope: &XScope, ident: &swc_ecma_ast::Ident) {
    let name_id = match scope.get(&ident.sym) {
      Some(name_id) => name_id,
      None => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Unresolved reference".to_string(),
          span: ident.span,
        });
        return;
      }
    };

    let name = match self.names.get(&name_id) {
      Some(name) => name,
      None => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "Expected name_id in names".to_string(),
          span: ident.span,
        });
        return;
      }
    };

    if &name.owner_id != &scope.borrow().owner_id {
      self.insert_capture(&scope.borrow().owner_id, &name_id, ident.span);
    }
  }

  fn prop_key(&mut self, scope: &XScope, prop_name: &swc_ecma_ast::PropName) {
    use swc_ecma_ast::PropName;

    match prop_name {
      PropName::Ident(_) => {}
      PropName::Str(_) => {}
      PropName::Num(_) => {}
      PropName::Computed(computed) => {
        self.expr(scope, &computed.expr);
      }
      PropName::BigInt(_) => {}
    }
  }

  fn prop_or_spread(&mut self, scope: &XScope, prop_or_spread: &swc_ecma_ast::PropOrSpread) {
    use swc_ecma_ast::PropOrSpread;

    match prop_or_spread {
      PropOrSpread::Prop(prop) => match &**prop {
        swc_ecma_ast::Prop::Shorthand(ident) => {
          self.ident(scope, &ident);
        }
        swc_ecma_ast::Prop::KeyValue(key_value) => {
          self.prop_key(scope, &key_value.key);
          self.expr(scope, &key_value.value);
        }
        swc_ecma_ast::Prop::Getter(getter) => {
          self.prop_key(scope, &getter.key);

          if let Some(body) = &getter.body {
            self.block_stmt(&scope, body);
          }
        }
        swc_ecma_ast::Prop::Setter(setter) => {
          self.prop_key(scope, &setter.key);
          self.param_pat(scope, &setter.param);

          if let Some(body) = &setter.body {
            self.block_stmt(&scope, body);
          }
        }
        swc_ecma_ast::Prop::Method(method) => {
          self.prop_key(scope, &method.key);
          self.function(scope, &None, &method.function);
        }
        swc_ecma_ast::Prop::Assign(assign) => {
          self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::InternalError,
            message: "TODO: implement property assignments (what are these?)".to_string(),
            span: assign.key.span, // TODO: Proper span of assign
          });
        }
      },
      PropOrSpread::Spread(spread) => {
        self.expr(scope, &spread.expr);
      }
    }
  }

  fn stmt(&mut self, scope: &XScope, stmt: &swc_ecma_ast::Stmt) {
    use swc_ecma_ast::Stmt;

    match stmt {
      Stmt::Block(block) => {
        self.block_stmt(&scope, block);
      }
      Stmt::Empty(_) => {}
      Stmt::Debugger(_) => {}
      Stmt::With(with) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Not supported: with statement".to_string(),
          span: with.span,
        });
      }
      Stmt::Return(return_) => {
        if let Some(arg) = &return_.arg {
          self.expr(&scope, arg);
        }
      }
      Stmt::Labeled(labeled_stmt) => {
        self.stmt(&scope, &labeled_stmt.body);
      }
      Stmt::Break(_) => {}
      Stmt::Continue(_) => {}
      Stmt::If(if_) => {
        self.expr(&scope, &if_.test);
        self.stmt(&scope, &if_.cons);

        for alt in &if_.alt {
          self.stmt(&scope, alt);
        }
      }
      Stmt::Switch(switch_) => {
        self.expr(&scope, &switch_.discriminant);
        let child_scope = scope.nest(None);

        for case in &switch_.cases {
          for test in &case.test {
            self.expr(&child_scope, test);
          }

          for stmt in &case.cons {
            self.stmt(&child_scope, stmt);
          }
        }
      }
      Stmt::Throw(throw_) => {
        self.expr(&scope, &throw_.arg);
      }
      Stmt::Try(try_) => {
        self.block_stmt(&scope, &try_.block);

        for catch in &try_.handler {
          let child_scope = scope.nest(None);

          for param in &catch.param {
            // TODO: Associate the catch param with the catch block
            self.param_pat(&child_scope, param);
          }

          self.block_stmt(&child_scope, &catch.body);
        }

        for finally in &try_.finalizer {
          self.block_stmt(&scope, finally);
        }
      }
      Stmt::While(while_) => {
        self.expr(&scope, &while_.test);
        self.stmt(&scope, &while_.body);
      }
      Stmt::DoWhile(do_while_) => {
        self.stmt(&scope, &do_while_.body);
        self.expr(&scope, &do_while_.test);
      }
      Stmt::For(for_) => {
        let child_scope = scope.nest(None);

        for init in &for_.init {
          if let swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl) = init {
            self.block_level_hoists_var_decl(&child_scope, var_decl);
          }

          match init {
            swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl) => {
              self.var_decl(&child_scope, var_decl);
            }
            swc_ecma_ast::VarDeclOrExpr::Expr(expr) => {
              self.expr(&child_scope, expr);
            }
          };
        }

        for test in &for_.test {
          self.expr(&child_scope, test);
        }

        for update in &for_.update {
          self.expr(&child_scope, update);
        }

        self.stmt(&child_scope, &for_.body);
      }
      Stmt::ForIn(for_in) => {
        let child_scope = scope.nest(None);

        match &for_in.left {
          swc_ecma_ast::VarDeclOrPat::VarDecl(var_decl) => {
            self.block_level_hoists_var_decl(&child_scope, var_decl);
            self.var_decl(&child_scope, var_decl);
          }
          swc_ecma_ast::VarDeclOrPat::Pat(pat) => {
            self.param_pat(&child_scope, pat);
          }
        }

        self.expr(&child_scope, &for_in.right);
        self.stmt(&child_scope, &for_in.body);
      }
      Stmt::ForOf(for_of) => {
        let child_scope = scope.nest(None);

        match &for_of.left {
          swc_ecma_ast::VarDeclOrPat::VarDecl(var_decl) => {
            self.block_level_hoists_var_decl(&child_scope, var_decl);
            self.var_decl(&child_scope, var_decl);
          }
          swc_ecma_ast::VarDeclOrPat::Pat(pat) => {
            self.param_pat(&child_scope, pat);
          }
        }

        self.expr(&child_scope, &for_of.right);
        self.stmt(&child_scope, &for_of.body);
      }
      Stmt::Decl(decl) => {
        self.decl(&scope, decl);
      }
      Stmt::Expr(expr) => {
        self.expr(&scope, &expr.expr);
      }
    };
  }

  fn block_stmt(&mut self, scope: &XScope, block_stmt: &swc_ecma_ast::BlockStmt) {
    let child_scope = scope.nest(None);
    self.block_level_hoists(&child_scope, block_stmt);

    for stmt in &block_stmt.stmts {
      self.stmt(&child_scope, stmt);
    }
  }

  fn param_pat(&mut self, scope: &XScope, param_pat: &swc_ecma_ast::Pat) {
    use swc_ecma_ast::Pat;

    match param_pat {
      Pat::Ident(ident) => {
        self.insert_name(&scope, NameType::Param, &ident.id);
      }
      Pat::Array(array_pat) => {
        for elem in &array_pat.elems {
          match elem {
            Some(pat) => self.param_pat(&scope, pat),
            None => {}
          }
        }
      }
      Pat::Rest(rest_pat) => {
        self.param_pat(&scope, &rest_pat.arg);
      }
      Pat::Object(object_pat) => {
        for prop in &object_pat.props {
          match prop {
            swc_ecma_ast::ObjectPatProp::KeyValue(key_value) => {
              self.param_pat(&scope, &key_value.value);
            }
            swc_ecma_ast::ObjectPatProp::Assign(assign) => {
              self.insert_name(&scope, NameType::Param, &assign.key);

              if let Some(default) = &assign.value {
                self.expr(&scope, default);
              }
            }
            swc_ecma_ast::ObjectPatProp::Rest(rest) => {
              self.param_pat(&scope, &rest.arg);
            }
          }
        }
      }
      Pat::Assign(assign_pat) => {
        self.param_pat(&scope, &assign_pat.left);
        self.expr(&scope, &assign_pat.right);
      }
      Pat::Expr(expr) => {
        self.expr(&scope, &expr);
      }
      Pat::Invalid(invalid) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Invalid pattern".to_string(),
          span: invalid.span,
        });
      }
    }
  }

  fn var_decl(&mut self, scope: &XScope, var_decl: &swc_ecma_ast::VarDecl) {
    for decl in &var_decl.decls {
      self.var_declarator(&scope, var_decl.kind, decl);
    }
  }
}

struct XScopeData {
  pub owner_id: OwnerId,
  pub name_map: HashMap<swc_atoms::JsWord, NameId>,
  pub parent: Option<Rc<RefCell<XScopeData>>>,
}

type XScope = Rc<RefCell<XScopeData>>;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum OwnerId {
  Span(swc_common::Span),
  Module,
}

trait XScopeTrait {
  fn get(&self, name: &swc_atoms::JsWord) -> Option<NameId>;
  fn set(
    &self,
    name: &swc_atoms::JsWord,
    name_id: NameId,
    span: swc_common::Span,
    diagnostics: &mut Vec<Diagnostic>,
  );
  fn nest(&self, name_owner_location: Option<OwnerId>) -> Rc<RefCell<XScopeData>>;
}

impl XScopeTrait for XScope {
  fn get(&self, name: &swc_atoms::JsWord) -> Option<NameId> {
    match self.borrow().name_map.get(name) {
      Some(mapped_name) => Some(mapped_name.clone()),
      None => match &self.borrow().parent {
        Some(parent) => parent.get(name),
        None => None,
      },
    }
  }

  fn set(
    &self,
    name: &swc_atoms::JsWord,
    name_id: NameId,
    span: swc_common::Span,
    diagnostics: &mut Vec<Diagnostic>,
  ) {
    let old_mapping = self.borrow_mut().name_map.insert(name.clone(), name_id);

    if old_mapping.is_some() {
      diagnostics.push(Diagnostic {
        level: DiagnosticLevel::Error,
        message: "Scope overwrite occurred (TODO: being permissive about this)".to_string(),
        span,
      });
    }
  }

  fn nest(&self, name_owner_location: Option<OwnerId>) -> Rc<RefCell<XScopeData>> {
    return Rc::new(RefCell::new(XScopeData {
      owner_id: name_owner_location.unwrap_or(self.borrow().owner_id.clone()),
      name_map: Default::default(),
      parent: Some(self.clone()),
    }));
  }
}

fn init_std_scope() -> XScope {
  return Rc::new(RefCell::new(XScopeData {
    owner_id: OwnerId::Module,
    name_map: HashMap::from([
      (swc_atoms::js_word!("Math"), NameId::Builtin(Builtin::Math)),
      (
        swc_atoms::JsWord::from("Debug"),
        NameId::Builtin(Builtin::Debug),
      ),
      (
        swc_atoms::js_word!("undefined"),
        NameId::Builtin(Builtin::undefined),
      ),
    ]),
    parent: None,
  }))
  .nest(None);
}

fn get_expr_span(expr: &swc_ecma_ast::Expr) -> swc_common::Span {
  use swc_ecma_ast::Expr;

  match expr {
    Expr::This(this) => this.span,
    Expr::Ident(ident) => ident.span,
    Expr::Lit(lit) => match lit {
      swc_ecma_ast::Lit::Str(str_lit) => str_lit.span,
      swc_ecma_ast::Lit::Bool(bool_lit) => bool_lit.span,
      swc_ecma_ast::Lit::Null(null_lit) => null_lit.span,
      swc_ecma_ast::Lit::Num(num_lit) => num_lit.span,
      swc_ecma_ast::Lit::BigInt(big_int_lit) => big_int_lit.span,
      swc_ecma_ast::Lit::Regex(regex_lit) => regex_lit.span,
      swc_ecma_ast::Lit::JSXText(jsx_text_lit) => jsx_text_lit.span,
    },
    Expr::Array(array) => array.span,
    Expr::Object(object) => object.span,
    Expr::Fn(fn_expr) => fn_expr.function.span,
    Expr::Unary(unary) => unary.span,
    Expr::Update(update) => update.span,
    Expr::Bin(bin) => bin.span,
    Expr::Assign(assign) => assign.span,
    Expr::Member(member) => member.span,
    Expr::Cond(cond) => cond.span,
    Expr::Call(call) => call.span,
    Expr::New(new) => new.span,
    Expr::Seq(seq) => seq.span,
    Expr::Paren(paren) => paren.span,
    Expr::Yield(yield_expr) => yield_expr.span,
    Expr::Await(await_expr) => await_expr.span,
    Expr::MetaProp(meta_prop) => meta_prop.span,
    Expr::Tpl(tpl) => tpl.span,
    Expr::TaggedTpl(tagged_tpl) => tagged_tpl.span,
    Expr::Arrow(arrow) => arrow.span,
    Expr::Class(class) => class.class.span,
    Expr::Invalid(invalid) => invalid.span,
    Expr::JSXMember(_) => std::panic!("TODO: span of JSXMember"),
    Expr::JSXNamespacedName(_) => {
      std::panic!("TODO: span of JSXNamespacedName")
    }
    Expr::JSXEmpty(jsx_empty) => jsx_empty.span,
    Expr::JSXElement(jsx_element) => jsx_element.span,
    Expr::JSXFragment(jsx_fragment) => jsx_fragment.span,
    Expr::TsTypeAssertion(ts_type_assertion) => ts_type_assertion.span,
    Expr::TsConstAssertion(ts_const_assertion) => ts_const_assertion.span,
    Expr::TsNonNull(ts_non_null) => ts_non_null.span,
    Expr::OptChain(opt_chain) => opt_chain.span,
    Expr::SuperProp(super_prop) => super_prop.span,
    Expr::TsAs(ts_as) => ts_as.span,
    Expr::PrivateName(private_name) => private_name.span,
    Expr::TsInstantiation(ts_instantiation) => ts_instantiation.span,
  }
}
