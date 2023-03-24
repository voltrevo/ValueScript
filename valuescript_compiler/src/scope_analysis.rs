use std::{
  cell::RefCell,
  collections::HashMap,
  collections::{BTreeMap, HashSet},
  rc::Rc,
};

use swc_common::Spanned;
use valuescript_common::BUILTIN_NAMES;

use crate::{
  asm::{Builtin, Register, Value},
  constants::CONSTANTS,
  name_allocator::{PointerAllocator, RegAllocator},
};

use super::diagnostic::{Diagnostic, DiagnosticLevel};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum NameId {
  Span(swc_common::Span),
  Builtin(Builtin),
  Constant(&'static str),
}

impl Spanned for NameId {
  fn span(&self) -> swc_common::Span {
    match self {
      NameId::Span(span) => *span,
      NameId::Builtin(_) => swc_common::DUMMY_SP,
      NameId::Constant(_) => swc_common::DUMMY_SP,
    }
  }
}

// TODO: Make use of these in the next phase of the compiler, remove the
// allow(dead_code) attributes
#[derive(Clone, Debug)]
pub struct Capture {
  #[allow(dead_code)]
  ref_: swc_common::Span,

  #[allow(dead_code)]
  captor_id: OwnerId,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NameType {
  Var,
  Let,
  Const,
  Param,
  Function,
  Class,
  Import,
  Builtin,
  Constant,
}

#[derive(Clone, Debug)]
pub struct Name {
  pub id: NameId,
  pub owner_id: OwnerId,
  pub sym: swc_atoms::JsWord,
  pub type_: NameType,
  pub value: Value,
  pub mutations: Vec<swc_common::Span>,
  pub captures: Vec<Capture>,
}

#[derive(Default)]
pub struct ScopeAnalysis {
  pub names: HashMap<NameId, Name>,
  pub owners: HashMap<OwnerId, HashSet<swc_common::Span>>,
  pub captures: HashMap<OwnerId, HashSet<NameId>>,
  pub capture_values: HashMap<(OwnerId, NameId), Value>,
  pub mutations: BTreeMap<swc_common::Span, NameId>,
  pub refs: HashMap<swc_common::Span, NameId>,
  pub diagnostics: Vec<Diagnostic>,
  pub pointer_allocator: PointerAllocator,
  pub reg_allocators: HashMap<OwnerId, RegAllocator>,
}

impl ScopeAnalysis {
  pub fn run(module: &swc_ecma_ast::Module) -> ScopeAnalysis {
    let mut sa = ScopeAnalysis::default();
    let scope = init_std_scope();

    for builtin_name in BUILTIN_NAMES {
      let builtin = Builtin {
        name: builtin_name.to_string(),
      };

      sa.names.insert(
        NameId::Builtin(builtin.clone()),
        Name {
          id: NameId::Builtin(builtin.clone()),
          owner_id: OwnerId::Module,
          sym: swc_atoms::JsWord::from(builtin_name),
          type_: NameType::Builtin,
          value: Value::Builtin(builtin),
          mutations: vec![],
          captures: vec![],
        },
      );
    }

    for (name, value) in CONSTANTS {
      sa.names.insert(
        NameId::Constant(name),
        Name {
          id: NameId::Constant(name),
          owner_id: OwnerId::Module,
          sym: swc_atoms::JsWord::from(name),
          type_: NameType::Constant,
          value,
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
            NameId::Builtin(_) | NameId::Constant(_) => {
              sa.diagnostics.push(Diagnostic {
                level: DiagnosticLevel::InternalError,
                message: "Builtin/constant should not have type_ let".to_string(),
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

  pub fn lookup(&self, scope: &OwnerId, ident: &swc_ecma_ast::Ident) -> Option<Value> {
    let name_id = self.refs.get(&ident.span)?;
    self.lookup_name_id(scope, name_id)
  }

  pub fn lookup_name_id(&self, scope: &OwnerId, name_id: &NameId) -> Option<Value> {
    let name = self.names.get(name_id)?;

    match &name.value {
      Value::Register(_) => {
        if &name.owner_id == scope {
          Some(name.value.clone())
        } else {
          self.lookup_capture(scope, name_id)
        }
      }
      _ => Some(name.value.clone()),
    }
  }

  pub fn lookup_capture(&self, scope: &OwnerId, name_id: &NameId) -> Option<Value> {
    let value = self.capture_values.get(&(scope.clone(), name_id.clone()))?;
    Some(value.clone())
  }

  fn allocate_reg(&mut self, scope: &OwnerId, based_on_name: &str) -> Register {
    self
      .reg_allocators
      .entry(scope.clone())
      .or_insert_with(|| RegAllocator::default())
      .allocate(based_on_name)
  }

  fn insert_name(
    &mut self,
    scope: &XScope,
    type_: NameType,
    value: Value,
    origin_ident: &swc_ecma_ast::Ident,
  ) {
    let name = Name {
      id: NameId::Span(origin_ident.span),
      owner_id: scope.borrow().owner_id.clone(),
      sym: origin_ident.sym.clone(),
      type_,
      value,
      mutations: Vec::new(),
      captures: Vec::new(),
    };

    self.names.insert(name.id.clone(), name.clone());
    self.refs.insert(origin_ident.span, name.id.clone());

    self
      .owners
      .entry(name.owner_id.clone())
      .or_insert_with(HashSet::new)
      .insert(origin_ident.span);

    scope.set(
      &origin_ident.sym,
      name.id.clone(),
      origin_ident.span,
      &mut self.diagnostics,
    );
  }

  fn insert_pointer_name(
    &mut self,
    scope: &XScope,
    type_: NameType,
    origin_ident: &swc_ecma_ast::Ident,
  ) {
    let pointer = Value::Pointer(self.pointer_allocator.allocate(&origin_ident.sym));
    self.insert_name(scope, type_, pointer, origin_ident);
  }

  fn insert_reg_name(
    &mut self,
    scope: &XScope,
    type_: NameType,
    origin_ident: &swc_ecma_ast::Ident,
  ) {
    let reg = Value::Register(self.allocate_reg(&scope.borrow().owner_id, &origin_ident.sym));
    self.insert_name(scope, type_, reg, origin_ident);
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

    let inserted = self
      .captures
      .entry(captor_id.clone())
      .or_insert_with(HashSet::new)
      .insert(name_id.clone());

    if inserted {
      let key = (captor_id.clone(), name_id.clone());

      // This is just self.allocate_reg, but we can't borrow all of self right now
      let reg = self
        .reg_allocators
        .entry(captor_id.clone())
        .or_insert_with(|| RegAllocator::default())
        .allocate(&name.sym);

      self.capture_values.insert(key, Value::Register(reg));
    }

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

        self.insert_pointer_name(scope, NameType::Import, &named_specifier.local);
      }
      Default(default_specifier) => {
        self.insert_pointer_name(scope, NameType::Import, &default_specifier.local);
      }
      Namespace(namespace_specifier) => {
        self.insert_pointer_name(scope, NameType::Import, &namespace_specifier.local);
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
      self.insert_pointer_name(&child_scope, NameType::Function, name);
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
            self.insert_pointer_name(&scope, NameType::Class, &class_decl.ident);
          }
          swc_ecma_ast::Decl::Fn(fn_decl) => {
            self.insert_pointer_name(&scope, NameType::Function, &fn_decl.ident);
          }
          swc_ecma_ast::Decl::Var(var_decl) => {
            let name_type = match var_decl.kind {
              swc_ecma_ast::VarDeclKind::Const => NameType::Const,
              swc_ecma_ast::VarDeclKind::Let => NameType::Let,
              swc_ecma_ast::VarDeclKind::Var => NameType::Var,
            };

            for decl in &var_decl.decls {
              for ident in self.get_pat_idents(&decl.name) {
                self.insert_pointer_name(&scope, name_type, &ident);
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
              self.insert_pointer_name(&scope, NameType::Class, ident);
            }
          }
          swc_ecma_ast::DefaultDecl::Fn(fn_decl) => {
            if let Some(ident) = &fn_decl.ident {
              self.insert_pointer_name(&scope, NameType::Function, ident);
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
                self.insert_reg_name(scope, NameType::Var, &ident);
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
                self.insert_reg_name(scope, NameType::Var, &ident);
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
                self.insert_reg_name(scope, NameType::Var, &ident);
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
                self.insert_reg_name(scope, NameType::Var, &ident);
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
          self.insert_pointer_name(scope, NameType::Class, &class.ident);
        }
        Decl::Fn(fn_) => {
          self.insert_pointer_name(scope, NameType::Function, &fn_.ident);
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
        self.insert_reg_name(scope, name_type, &ident);
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
          span: expr.span(),
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
          span: expr.span(),
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
      self.insert_pointer_name(&child_scope, NameType::Class, ident);
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
                  self.insert_reg_name(&child_scope, NameType::Param, &ident.id);
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
      Expr::Assign(assign) => {
        match &assign.left {
          swc_ecma_ast::PatOrExpr::Pat(pat) => {
            self.pat(scope, pat);
            self.mutate_pat(scope, pat);
          }
          swc_ecma_ast::PatOrExpr::Expr(expr) => {
            self.expr(scope, expr);
            self.mutate_expr(scope, expr);
          }
        }

        self.expr(scope, &assign.right);
      }
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
      Expr::Tpl(tpl) => {
        for elem in &tpl.exprs {
          self.expr(scope, elem);
        }
      }
      Expr::TaggedTpl(_) => {
        // TODO (diagnostic emitted elsewhere)
      }
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
          span: expr.span(),
        });
      }
      Expr::JSXNamespacedName(_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: expr.span(),
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

  fn pat(&mut self, scope: &XScope, pat: &swc_ecma_ast::Pat) {
    use swc_ecma_ast::Pat;

    match pat {
      Pat::Ident(ident) => {
        self.ident(scope, &ident.id);
      }
      Pat::Array(array) => {
        for elem in &array.elems {
          if let Some(elem) = elem {
            self.pat(scope, elem);
          }
        }
      }
      Pat::Rest(rest) => {
        self.pat(scope, &rest.arg);
      }
      Pat::Object(object) => {
        for prop in &object.props {
          match prop {
            swc_ecma_ast::ObjectPatProp::KeyValue(key_value) => {
              self.pat(scope, &key_value.value);
            }
            swc_ecma_ast::ObjectPatProp::Assign(assign) => {
              self.ident(scope, &assign.key);

              if let Some(value) = &assign.value {
                self.expr(scope, value);
              }
            }
            swc_ecma_ast::ObjectPatProp::Rest(rest) => {
              self.pat(scope, &rest.arg);
            }
          }
        }
      }
      Pat::Assign(assign) => {
        self.pat(scope, &assign.left);
        self.expr(scope, &assign.right);
      }
      Pat::Invalid(_) => {}
      Pat::Expr(expr) => {
        self.expr(scope, expr);
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
          level: DiagnosticLevel::Error,
          message: "Mutating a (non-pattern) array expression is not valid. \
            This is an unusual case that can occur with things like [a,b]+=c."
            .to_string(),
          span: array.span,
        });
      }
      Expr::Object(object) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Mutating a (non-pattern) object expression is not valid. \
            This is an unusual case - it's not clear whether SWC ever emit it. \
            Please consider creating an issue: \
            https://github.com/ValueScript/issues/new."
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
          span: expr.span(),
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
          span: expr.span(),
        });
      }
      Expr::JSXNamespacedName(_) => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: "TODO".to_string(),
          span: expr.span(),
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
    self.mutations.insert(ident.span, name_id.clone());
    self.refs.insert(ident.span, name_id);
  }

  fn ident(&mut self, scope: &XScope, ident: &swc_ecma_ast::Ident) {
    if ident.sym.to_string() == "undefined" {
      // The way that `undefined` is considered to be an identifier is an artifact of history. It's
      // not an identifier (unless used in an identifier context like an object key), instead it's a
      // keyword like `null`.
      return;
    }

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

    self.refs.insert(ident.span, name_id.clone());

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
      Stmt::Throw(throw) => {
        self.expr(&scope, &throw.arg);
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
        self.insert_reg_name(&scope, NameType::Param, &ident.id);
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
              self.insert_reg_name(&scope, NameType::Param, &assign.key);

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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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
  let mut name_map = HashMap::new();

  for name in BUILTIN_NAMES {
    name_map.insert(
      swc_atoms::JsWord::from(name),
      NameId::Builtin(Builtin {
        name: name.to_string(),
      }),
    );
  }

  for (name, _) in CONSTANTS {
    name_map.insert(swc_atoms::JsWord::from(name), NameId::Constant(name));
  }

  Rc::new(RefCell::new(XScopeData {
    owner_id: OwnerId::Module,
    name_map,
    parent: None,
  }))
  .nest(None)
}
