use std::{cell::RefCell, collections::HashMap, collections::HashSet, rc::Rc};

use super::scope::Builtin;

#[derive(Hash, PartialEq, Eq, Clone)]
pub enum NameId {
  Span(swc_common::Span),
  Builtin(Builtin),
}

#[derive(Clone)]
pub struct Capture {
  ref_: swc_common::Span,
  captor_id: OwnerId,
}

#[derive(Clone, Copy)]
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
}

impl ScopeAnalysis {
  pub fn run(module: &swc_ecma_ast::Module) -> ScopeAnalysis {
    let mut sa = ScopeAnalysis::default();
    let scope = init_std_scope();

    use swc_ecma_ast::ModuleDecl;
    use swc_ecma_ast::ModuleItem;

    for module_item in &module.body {
      match module_item {
        ModuleItem::ModuleDecl(module_decl) => match module_decl {
          ModuleDecl::Import(import_decl) => {
            sa.import_decl(&scope, import_decl);
          }
          ModuleDecl::ExportDecl(ed) => {
            sa.decl(&scope, &ed.decl);
          }
          ModuleDecl::ExportNamed(_) => {}
          ModuleDecl::ExportDefaultDecl(edd) => {
            sa.default_decl(&scope, &edd.decl);
          }
          ModuleDecl::ExportDefaultExpr(ede) => {
            sa.expr(&scope, &ede.expr);
          }
          ModuleDecl::ExportAll(_) => {}
          ModuleDecl::TsImportEquals(_) => {
            std::panic!("Not supported: TsImportEquals module declaration")
          }
          ModuleDecl::TsExportAssignment(_) => {
            std::panic!("Not supported: TsExportAssignment module declaration")
          }
          ModuleDecl::TsNamespaceExport(_) => {
            std::panic!("Not supported: TsNamespaceExport module declaration")
          }
        },
        ModuleItem::Stmt(stmt) => {
          sa.stmt(&scope, &stmt);
        }
      };
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

    scope
      .borrow_mut()
      .name_map
      .insert(origin_ident.sym.clone(), name.id);
  }

  fn insert_capture(&mut self, captor_id: &OwnerId, name_id: &NameId, ref_: &swc_common::Span) {
    self
      .captures
      .entry(captor_id.clone())
      .or_insert_with(HashSet::new)
      .insert(ref_.clone());

    let name = self
      .names
      .get_mut(name_id)
      .expect("Internal: expected name_id in names");

    name.captures.push(Capture {
      ref_: ref_.clone(),
      captor_id: captor_id.clone(),
    });
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
        self.insert_name(scope, NameType::Class, &class_decl.ident);
        self.class_(scope, &Some(class_decl.ident.clone()), &class_decl.class);
      }
      Decl::Fn(fn_decl) => {
        self.insert_name(scope, NameType::Function, &fn_decl.ident);
        self.function(scope, &Some(fn_decl.ident.clone()), &fn_decl.function);
      }
      Decl::Var(var_decl) => {
        for decl in &var_decl.decls {
          self.var_declarator(&scope, var_decl.kind, decl);
        }
      }
      Decl::TsInterface(_) => {}
      Decl::TsTypeAlias(_) => {}
      Decl::TsEnum(_) => {
        std::panic!("Not implemented: TsEnum declaration (TODO)")
      }
      Decl::TsModule(_) => {
        std::panic!("Not supported: TsModule declaration")
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

    for n in name {
      self.insert_name(&child_scope, NameType::Function, n);
    }

    for param in &function.params {
      self.param_pat(&child_scope, &param.pat);
    }

    for body in &function.body {
      self.block_stmt(&child_scope, &body);
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
      Pat::Ident(ident) => {
        self.insert_name(scope, type_, &ident.id);
      }
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
              self.insert_name(scope, type_, &assign.key);

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
      Pat::Invalid(_) => {
        std::panic!("Invalid pattern");
      }
      Pat::Expr(_) => {
        std::panic!("Not implemented: pattern expression (TODO: what is this?)");
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
      Expr::Await(_) => {
        std::panic!("Not supported: await")
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

      Expr::SuperProp(_) => {
        std::panic!("TODO");
      }
      Expr::JSXMember(_) => {
        std::panic!("TODO");
      }
      Expr::JSXNamespacedName(_) => {
        std::panic!("TODO");
      }
      Expr::JSXEmpty(_) => {
        std::panic!("TODO");
      }
      Expr::JSXElement(_) => {
        std::panic!("TODO");
      }
      Expr::JSXFragment(_) => {
        std::panic!("TODO");
      }
      Expr::TsInstantiation(_) => {
        std::panic!("TODO");
      }
      Expr::PrivateName(_) => {
        std::panic!("TODO");
      }
    }
  }

  fn mutate_expr(&mut self, scope: &XScope, expr: &swc_ecma_ast::Expr) {
    use swc_ecma_ast::Expr;

    match expr {
      Expr::Ident(ident) => {
        self.mutate_ident(ident);
      }
      Expr::Member(member) => {
        self.mutate_expr(scope, &member.obj);
      }
      Expr::Call(_) => {
        std::panic!("Unexpected mutation of call expression");
      }
      Expr::New(_) => {
        std::panic!("Unexpected mutation of new expression");
      }
      Expr::Paren(paren) => {
        self.mutate_expr(scope, &paren.expr);
      }
      Expr::Tpl(_) => {
        std::panic!("Unexpected mutation of template literal");
      }
      Expr::TaggedTpl(_) => {
        std::panic!("Unexpected mutation of tagged template literal");
      }
      Expr::Arrow(_) => {
        std::panic!("Unexpected mutation of arrow function");
      }
      Expr::Class(_) => {
        std::panic!("Unexpected mutation of class expression");
      }
      Expr::MetaProp(_) => {
        std::panic!("Unexpected mutation of meta property");
      }
      Expr::Invalid(_) => {
        std::panic!("Invalid expression");
      }
      Expr::TsTypeAssertion(_) => {
        std::panic!("Unexpected mutation of type assertion");
      }
      Expr::TsConstAssertion(_) => {
        std::panic!("Unexpected mutation of const assertion");
      }
      Expr::TsNonNull(_) => {
        std::panic!("Unexpected mutation of non-null assertion");
      }
      Expr::TsAs(as_expr) => {
        self.mutate_expr(scope, &as_expr.expr);
      }
      Expr::OptChain(_) => {
        std::panic!("Unexpected mutation of optional chain");
      }

      Expr::This(_) => {
        std::panic!("TODO");
      }
      Expr::Array(_) => {
        std::panic!("TODO");
      }
      Expr::Object(_) => {
        std::panic!("TODO");
      }
      Expr::Fn(_) => {
        std::panic!("TODO");
      }
      Expr::Unary(_) => {
        std::panic!("TODO");
      }
      Expr::Update(_) => {
        std::panic!("TODO");
      }
      Expr::Bin(_) => {
        std::panic!("TODO");
      }
      Expr::Assign(_) => {
        std::panic!("TODO");
      }
      Expr::SuperProp(_) => {
        std::panic!("TODO");
      }
      Expr::Cond(_) => {
        std::panic!("TODO");
      }
      Expr::Seq(_) => {
        std::panic!("TODO");
      }
      Expr::Lit(_) => {
        std::panic!("TODO");
      }
      Expr::Yield(_) => {
        std::panic!("TODO");
      }
      Expr::Await(_) => {
        std::panic!("TODO");
      }
      Expr::JSXMember(_) => {
        std::panic!("TODO");
      }
      Expr::JSXNamespacedName(_) => {
        std::panic!("TODO");
      }
      Expr::JSXEmpty(_) => {
        std::panic!("TODO");
      }
      Expr::JSXElement(_) => {
        std::panic!("TODO");
      }
      Expr::JSXFragment(_) => {
        std::panic!("TODO");
      }
      Expr::TsInstantiation(_) => {
        std::panic!("TODO");
      }
      Expr::PrivateName(_) => {
        std::panic!("TODO");
      }
    }
  }

  fn mutate_pat(&mut self, scope: &XScope, pat: &swc_ecma_ast::Pat) {
    use swc_ecma_ast::Pat;

    match pat {
      Pat::Ident(ident) => {
        self.mutate_ident(&ident.id);
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

              self.mutate_ident(&assign.key);

              if let Some(value) = &assign.value {
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
      Pat::Invalid(_) => {
        std::panic!("Invalid pattern");
      }
      Pat::Expr(_) => {
        std::panic!("Not implemented: pattern expression (TODO: what is this?)");
      }
    }
  }

  fn mutate_ident(&mut self, ident: &swc_ecma_ast::Ident) {
    let name = self
      .names
      .get_mut(&NameId::Span(ident.span)) // TODO: clone?
      .expect("Unresolved reference");

    // TODO: .clone?
    name.mutations.push(ident.span);
  }

  fn ident(&mut self, scope: &XScope, ident: &swc_ecma_ast::Ident) {
    let name_id = scope.get(&ident.sym).expect("Unresolved reference");

    let name = self
      .names
      .get(&name_id)
      .expect("Internal: expected name_id in names");

    if &name.owner_id != &scope.borrow().owner_id {
      self.insert_capture(&scope.borrow().owner_id, &name_id, &ident.span);
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
        swc_ecma_ast::Prop::Assign(_) => {
          std::panic!("Not implemented: property assignment (TODO: what is this?)");
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
      Stmt::With(_) => std::panic!("Not implemented: With statement"),
      Stmt::Return(_) => std::panic!("Invalid: module level Return statement"),
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
          // TODO: associate scope with body
          swc_ecma_ast::VarDeclOrPat::VarDecl(var_decl) => {
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
      Pat::Invalid(_) => {
        std::panic!("Invalid pattern");
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
  fn set(&self, name: &swc_atoms::JsWord, name_id: NameId);
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

  fn set(&self, name: &swc_atoms::JsWord, name_id: NameId) {
    let old_mapping = self.borrow_mut().name_map.insert(name.clone(), name_id);

    if old_mapping.is_some() {
      std::panic!("Scope overwrite occurred (not implemented: being permissive about this)");
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
    ]),
    parent: None,
  }))
  .nest(None);
}
