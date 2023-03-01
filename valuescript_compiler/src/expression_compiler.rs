use queues::*;

use swc_common::Spanned;

use super::capture_finder::CaptureFinder;
use super::diagnostic::{Diagnostic, DiagnosticLevel};
use super::function_compiler::{FunctionCompiler, Functionish, QueuedFunction};
use super::scope::{init_std_scope, MappedName, Scope, ScopeTrait};

pub struct CompiledExpression {
  /** It is usually better to access this via functionCompiler.use_ */
  pub value_assembly: String,

  pub nested_registers: Vec<String>,
  pub release_checker: ReleaseChecker,
}

pub struct ReleaseChecker {
  pub has_unreleased_registers: bool,
}

impl ReleaseChecker {
  pub fn new(has_unreleased_registers: bool) -> ReleaseChecker {
    ReleaseChecker {
      has_unreleased_registers,
    }
  }
}

impl CompiledExpression {
  pub fn empty() -> CompiledExpression {
    CompiledExpression {
      value_assembly: "void".to_string(), // TODO: Allocate register instead
      nested_registers: vec![],
      release_checker: ReleaseChecker::new(false),
    }
  }

  pub fn new(value_assembly: String, nested_registers: Vec<String>) -> CompiledExpression {
    let has_unreleased_registers = nested_registers.len() > 0;

    CompiledExpression {
      value_assembly,
      nested_registers,
      release_checker: ReleaseChecker::new(has_unreleased_registers),
    }
  }
}

impl Drop for ReleaseChecker {
  fn drop(&mut self) {
    if self.has_unreleased_registers {
      panic!("CompiledExpression dropped with unreleased registers");
    }
  }
}

pub struct ExpressionCompiler<'a> {
  pub scope: &'a Scope,
  pub fnc: &'a mut FunctionCompiler,
}

impl<'a> ExpressionCompiler<'a> {
  pub fn compile(
    &mut self,
    expr: &swc_ecma_ast::Expr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    use swc_ecma_ast::Expr::*;

    match expr {
      This(_) => {
        return self.inline("%this".to_string(), target_register);
      }
      Array(array_exp) => {
        return self.array_expression(array_exp, target_register);
      }
      Object(object_exp) => {
        return self.object_expression(object_exp, target_register);
      }
      Fn(fn_) => {
        return self.fn_expression(fn_, target_register);
      }
      Unary(un_exp) => {
        return self.unary_expression(un_exp, target_register);
      }
      Update(update_exp) => {
        return self.update_expression(update_exp, target_register);
      }
      Bin(bin_exp) => {
        return self.binary_expression(bin_exp, target_register);
      }
      Assign(assign_exp) => {
        return self.assign_expression(assign_exp, target_register);
      }
      Member(member_exp) => {
        return self.member_expression(member_exp, target_register);
      }
      SuperProp(super_prop) => {
        self.fnc.todo(super_prop.span, "SuperProp expression");
        return CompiledExpression::empty();
      }
      Cond(cond_exp) => {
        self.fnc.todo(cond_exp.span, "Cond expression");
        return CompiledExpression::empty();
      }
      Call(call_exp) => {
        return match &call_exp.callee {
          swc_ecma_ast::Callee::Expr(callee_expr) => match &**callee_expr {
            swc_ecma_ast::Expr::Member(member_expr) => {
              self.method_call_expression(&member_expr, &call_exp.args, target_register)
            }
            _ => self.call_expression(call_exp, target_register),
          },
          _ => {
            self
              .fnc
              .todo(call_exp.callee.span(), "non-expression callee");

            CompiledExpression::empty()
          }
        };
      }
      New(new_exp) => {
        return self.new_expression(new_exp, target_register);
      }
      Seq(seq_exp) => {
        self.fnc.todo(seq_exp.span, "Seq expression");
        return CompiledExpression::empty();
      }
      Ident(ident) => {
        return self.identifier(ident, target_register);
      }
      Lit(lit) => {
        return self.literal(lit, target_register);
      }
      Tpl(tpl) => {
        return self.template_literal(tpl, target_register);
      }
      TaggedTpl(tagged_tpl) => {
        self.fnc.todo(tagged_tpl.span, "TaggedTpl expression");
        return CompiledExpression::empty();
      }
      Arrow(arrow) => return self.arrow_expression(arrow, target_register),
      Class(class_exp) => {
        self.fnc.todo(class_exp.span(), "Class expression");
        return CompiledExpression::empty();
      }
      Yield(yield_exp) => {
        self.fnc.todo(yield_exp.span, "Yield expression");
        return CompiledExpression::empty();
      }
      MetaProp(meta_prop) => {
        self.fnc.todo(meta_prop.span, "MetaProp expression");
        return CompiledExpression::empty();
      }
      Await(await_exp) => {
        self.fnc.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Await expression is not supported".to_string(),
          span: await_exp.span,
        });

        return CompiledExpression::empty();
      }
      Paren(p) => {
        return self.compile(&*p.expr, target_register);
      }
      JSXMember(jsx_member) => {
        self.fnc.todo(jsx_member.span(), "JSXMember expression");
        return CompiledExpression::empty();
      }
      JSXNamespacedName(jsx_namespaced_name) => {
        self
          .fnc
          .todo(jsx_namespaced_name.span(), "JSXNamespacedName expression");
        return CompiledExpression::empty();
      }
      JSXEmpty(jsx_empty) => {
        self.fnc.todo(jsx_empty.span(), "JSXEmpty expression");
        return CompiledExpression::empty();
      }
      JSXElement(jsx_element) => {
        self.fnc.todo(jsx_element.span(), "JSXElement expression");
        return CompiledExpression::empty();
      }
      JSXFragment(jsx_fragment) => {
        self.fnc.todo(jsx_fragment.span(), "JSXFragment expression");
        return CompiledExpression::empty();
      }
      TsTypeAssertion(ts_type_assertion) => {
        self
          .fnc
          .todo(ts_type_assertion.span, "TsTypeAssertion expression");

        return CompiledExpression::empty();
      }
      TsConstAssertion(ts_const_assertion) => {
        self
          .fnc
          .todo(ts_const_assertion.span, "TsConstAssertion expression");

        return CompiledExpression::empty();
      }
      TsNonNull(ts_non_null_exp) => {
        return self.compile(&ts_non_null_exp.expr, target_register);
      }
      TsAs(ts_as_exp) => {
        return self.compile(&ts_as_exp.expr, target_register);
      }
      TsInstantiation(ts_instantiation) => {
        self
          .fnc
          .todo(ts_instantiation.span, "TsInstantiation expression");

        return CompiledExpression::empty();
      }
      PrivateName(private_name) => {
        self.fnc.todo(private_name.span, "PrivateName expression");
        return CompiledExpression::empty();
      }
      OptChain(opt_chain) => {
        self.fnc.todo(opt_chain.span, "OptChain expression");
        return CompiledExpression::empty();
      }
      Invalid(invalid) => {
        self.fnc.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Invalid expression".to_string(),
          span: invalid.span,
        });

        return CompiledExpression::empty();
      }
    };
  }

  pub fn unary_expression(
    &mut self,
    un_exp: &swc_ecma_ast::UnaryExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<String>::new();

    let unary_op_str = match get_unary_op_str(un_exp.op) {
      Some(s) => s,
      None => {
        self
          .fnc
          .todo(un_exp.span, &format!("Unary operator {:?}", un_exp.op));

        return CompiledExpression::empty();
      }
    };

    let arg = self.compile(&un_exp.arg, None);

    let mut instr = "  ".to_string();
    instr += unary_op_str;
    instr += " ";
    instr += &self.fnc.use_(arg);

    let target: String = match &target_register {
      None => {
        let res = self
          .fnc
          .reg_allocator
          .allocate_numbered(&"_tmp".to_string());
        nested_registers.push(res.clone());
        res
      }
      Some(t) => t.clone(),
    };

    instr += " %";
    instr += &target;

    self.fnc.definition.push(instr);

    return CompiledExpression::new(format!("%{}", target), nested_registers);
  }

  pub fn binary_expression(
    &mut self,
    bin: &swc_ecma_ast::BinExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<String>::new();

    let left = self.compile(&bin.left, None);

    let right = self.compile(&bin.right, None);

    let mut instr = "  ".to_string();

    // FIXME: && and || need to avoid executing the right side where applicable
    // (mandatory if they mutate)
    instr += get_binary_op_str(bin.op);

    instr += " ";
    instr += &self.fnc.use_(left);
    instr += " ";
    instr += &self.fnc.use_(right);

    let target: String = match &target_register {
      None => {
        let res = self
          .fnc
          .reg_allocator
          .allocate_numbered(&"_tmp".to_string());
        nested_registers.push(res.clone());
        res
      }
      Some(t) => t.clone(),
    };

    instr += " %";
    instr += &target;

    self.fnc.definition.push(instr);

    return CompiledExpression::new(format!("%{}", target), nested_registers);
  }

  fn get_register_for_ident_mutation(&mut self, ident: &swc_ecma_ast::Ident) -> String {
    let (reg, err_msg) = match self.scope.get(&ident.sym.to_string()) {
      None => (None, Some("Unresolved identifier")),
      Some(MappedName::Definition(_)) => (None, Some("Invalid: definition mutation")),
      Some(MappedName::QueuedFunction(_)) => (None, Some("Invalid: declaration mutation")),
      Some(MappedName::Register(reg)) => (Some(reg), None),
      Some(MappedName::Builtin(_)) => (None, Some("Invalid: builtin mutation")),
    };

    if let Some(err_msg) = err_msg {
      self.fnc.diagnostics.push(Diagnostic {
        level: DiagnosticLevel::Error,
        message: err_msg.to_string(),
        span: ident.span,
      });
    }

    if let Some(reg) = reg {
      return reg;
    }

    return self
      .fnc
      .reg_allocator
      .allocate_numbered(&format!("_couldnt_mutate_{}_", ident.sym.to_string()));
  }

  pub fn assign_expression(
    &mut self,
    assign_expr: &swc_ecma_ast::AssignExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    match get_assign_op_str(assign_expr.op) {
      None => self.assign_expr_eq(assign_expr, target_register),
      Some(op_str) => self.assign_expr_compound(assign_expr, op_str, target_register),
    }
  }

  pub fn assign_expr_eq(
    &mut self,
    assign_expr: &swc_ecma_ast::AssignExpr,
    _target_register: Option<String>,
  ) -> CompiledExpression {
    enum AssignTarget {
      Register(String),
      Member(TargetAccessor, swc_ecma_ast::MemberProp),
    }

    impl AssignTarget {
      fn from_expr(ec: &mut ExpressionCompiler, expr: &swc_ecma_ast::Expr) -> AssignTarget {
        return match expr {
          swc_ecma_ast::Expr::Ident(ident) => {
            AssignTarget::Register(ec.get_register_for_ident_mutation(&ident))
          }
          swc_ecma_ast::Expr::This(_) => AssignTarget::Register("this".to_string()),
          swc_ecma_ast::Expr::Member(member) => AssignTarget::Member(
            TargetAccessor::compile(ec, &member.obj),
            member.prop.clone(),
          ),
          swc_ecma_ast::Expr::SuperProp(super_prop) => {
            ec.fnc.todo(super_prop.span(), "SuperProp");

            let bad_reg = ec
              .fnc
              .reg_allocator
              .allocate_numbered(&"_todo_super_prop".to_string());

            AssignTarget::Register(bad_reg)
          }
          _ => {
            ec.fnc.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::Error,
              message: "Invalid lvalue expression".to_string(),
              span: expr.span(),
            });

            let bad_reg = ec
              .fnc
              .reg_allocator
              .allocate_numbered(&"_bad_lvalue".to_string());

            AssignTarget::Register(bad_reg)
          }
        };
      }
    }

    let at = match &assign_expr.left {
      swc_ecma_ast::PatOrExpr::Expr(expr) => AssignTarget::from_expr(self, expr),
      swc_ecma_ast::PatOrExpr::Pat(pat) => match &**pat {
        swc_ecma_ast::Pat::Ident(ident) => {
          AssignTarget::Register(self.get_register_for_ident_mutation(&ident.id))
        }
        swc_ecma_ast::Pat::Expr(expr) => AssignTarget::from_expr(self, expr),
        _ => {
          self.fnc.todo(pat.span(), "destructuring (a)");

          let bad_reg = self
            .fnc
            .reg_allocator
            .allocate_numbered(&"_todo_destructuring".to_string());

          AssignTarget::Register(bad_reg)
        }
      },
    };

    match at {
      AssignTarget::Register(treg) => {
        self.compile(&assign_expr.right, Some(treg.clone()));

        return CompiledExpression::new(format!("%{}", treg), vec![]);
      }
      AssignTarget::Member(mut obj_accessor, prop) => {
        let subscript = match prop {
          swc_ecma_ast::MemberProp::Ident(ident) => {
            CompiledExpression::new(format!("\"{}\"", ident.sym.to_string()), vec![])
          }
          swc_ecma_ast::MemberProp::Computed(computed) => self.compile(&computed.expr, None),
          swc_ecma_ast::MemberProp::PrivateName(_) => {
            self.fnc.todo(prop.span(), "private name");
            CompiledExpression::empty()
          }
        };

        let rhs = self.compile(&assign_expr.right, None);

        let submov_instr = format!(
          "  submov {} {} %{}",
          self.fnc.use_(subscript),
          rhs.value_assembly,
          obj_accessor.register(),
        );

        self.fnc.definition.push(submov_instr);

        obj_accessor.packup(self);

        let res_reg = self
          .fnc
          .reg_allocator
          .allocate_numbered(&"_tmp".to_string());

        let mov_instr = format!("  mov {} %{}", self.fnc.use_(rhs), res_reg);

        self.fnc.definition.push(mov_instr);

        return CompiledExpression::new(format!("%{}", res_reg), vec![res_reg]);
      }
    };
  }

  pub fn assign_expr_compound(
    &mut self,
    assign_expr: &swc_ecma_ast::AssignExpr,
    op_str: &str,
    target_register: Option<String>,
  ) -> CompiledExpression {
    use swc_ecma_ast::Pat;
    use swc_ecma_ast::PatOrExpr;

    let mut target = match &assign_expr.left {
      PatOrExpr::Expr(expr) => TargetAccessor::compile(self, &expr),
      PatOrExpr::Pat(pat) => match &**pat {
        Pat::Ident(ident) => {
          TargetAccessor::Register(self.get_register_for_ident_mutation(&ident.id))
        }
        _ => {
          self.fnc.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            message: "Invalid lvalue expression".to_string(),
            span: pat.span(),
          });

          let bad_reg = self
            .fnc
            .reg_allocator
            .allocate_numbered(&"_bad_lvalue".to_string());

          TargetAccessor::Register(bad_reg)
        }
      },
    };

    let tmp_reg = self
      .fnc
      .reg_allocator
      .allocate_numbered(&"_tmp".to_string());

    let pre_rhs = self.compile(&assign_expr.right, Some(tmp_reg.clone()));

    // TODO: Consider making two variations of compile, one that takes a target
    // register and one that doesn't. This may simplify things eg by not
    // returning any nested registers when there's a target.
    assert_eq!(pre_rhs.nested_registers.len(), 0);

    self.fnc.definition.push(format!(
      "  {} %{} %{} %{}",
      op_str,
      target.register(),
      tmp_reg,
      target.register(),
    ));

    self.fnc.reg_allocator.release(&tmp_reg);

    let mut nested_registers = Vec::<String>::new();

    let result_reg = match &target {
      TargetAccessor::Register(treg) => {
        match target_register {
          None => {}
          Some(tr) => {
            self.fnc.definition.push(format!("  mov %{} %{}", treg, tr));
          }
        }

        treg.clone()
      }
      TargetAccessor::Nested(nta) => {
        let res_reg = match target_register {
          None => {
            let reg = self
              .fnc
              .reg_allocator
              .allocate_numbered(&"_tmp".to_string());
            nested_registers.push(reg.clone());

            reg
          }
          Some(tr) => tr,
        };

        self
          .fnc
          .definition
          .push(format!("  mov %{} %{}", nta.register, res_reg));

        res_reg
      }
    };

    target.packup(self);

    CompiledExpression::new(format!("%{}", result_reg), nested_registers)
  }

  pub fn array_expression(
    &mut self,
    array_exp: &swc_ecma_ast::ArrayLit,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut value_assembly = "[".to_string();
    let mut sub_nested_registers = Vec::<String>::new();

    for i in 0..array_exp.elems.len() {
      match &array_exp.elems[i] {
        None => {
          value_assembly += "void";
        }
        Some(elem) => {
          if elem.spread.is_some() {
            self.fnc.todo(elem.span(), "spread expression");

            let reg = self
              .fnc
              .reg_allocator
              .allocate_numbered(&"_todo_spread".to_string());

            value_assembly += format!("%{}", reg).as_str();
          } else {
            let mut compiled_elem = self.compile(&*elem.expr, None);
            value_assembly += &compiled_elem.value_assembly;
            sub_nested_registers.append(&mut compiled_elem.nested_registers);
            compiled_elem.release_checker.has_unreleased_registers = false;
          }
        }
      }

      if i != array_exp.elems.len() - 1 {
        value_assembly += ", ";
      }
    }

    value_assembly += "]";

    return match target_register {
      None => CompiledExpression::new(value_assembly, sub_nested_registers),
      Some(tr) => {
        self
          .fnc
          .definition
          .push(std::format!("  mov {} %{}", value_assembly, tr));

        for reg in sub_nested_registers {
          self.fnc.reg_allocator.release(&reg);
        }

        CompiledExpression::new(std::format!("%{}", tr), vec![])
      }
    };
  }

  pub fn object_expression(
    &mut self,
    object_exp: &swc_ecma_ast::ObjectLit,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut sub_nested_registers = Vec::<String>::new();
    let mut prop_elements = Vec::<String>::new();

    for i in 0..object_exp.props.len() {
      use swc_ecma_ast::Prop;
      use swc_ecma_ast::PropOrSpread;

      match &object_exp.props[i] {
        PropOrSpread::Spread(spread) => {
          self.fnc.todo(spread.span(), "spread expression");
        }
        PropOrSpread::Prop(prop) => match &**prop {
          Prop::Shorthand(ident) => {
            let mut prop_element = "".to_string();

            prop_element += &std::format!("\"{}\"", ident.sym.to_string());
            prop_element += ": ";

            let mut compiled_value = self.identifier(ident, None);
            sub_nested_registers.append(&mut compiled_value.nested_registers);
            compiled_value.release_checker.has_unreleased_registers = false;
            prop_element += &compiled_value.value_assembly;

            prop_elements.push(prop_element);
          }
          Prop::KeyValue(kv) => {
            let mut prop_element = "".to_string();

            let mut compiled_key = self.prop_name(&kv.key);
            compiled_key.release_checker.has_unreleased_registers = false;
            sub_nested_registers.append(&mut compiled_key.nested_registers);

            prop_element += &compiled_key.value_assembly;
            prop_element += ": ";

            let mut compiled_value = self.compile(&kv.value, None);
            compiled_value.release_checker.has_unreleased_registers = false;
            sub_nested_registers.append(&mut compiled_value.nested_registers);
            prop_element += &compiled_value.value_assembly;

            prop_elements.push(prop_element);
          }
          Prop::Assign(assign) => self.fnc.todo(assign.span(), "Assign prop"),
          Prop::Getter(getter) => self.fnc.todo(getter.span(), "Getter prop"),
          Prop::Setter(setter) => self.fnc.todo(setter.span(), "Setter prop"),
          Prop::Method(method) => self.fnc.todo(method.span(), "Method prop"),
        },
      }
    }

    let value_assembly = format!("{{ {} }}", prop_elements.join(", "));

    return match target_register {
      None => CompiledExpression::new(value_assembly, sub_nested_registers),
      Some(tr) => {
        self
          .fnc
          .definition
          .push(std::format!("  mov {} %{}", value_assembly, tr));

        for reg in sub_nested_registers {
          self.fnc.reg_allocator.release(&reg);
        }

        CompiledExpression::new(std::format!("%{}", tr), vec![])
      }
    };
  }

  pub fn prop_name(&mut self, prop_name: &swc_ecma_ast::PropName) -> CompiledExpression {
    use swc_ecma_ast::PropName;

    let mut nested_registers = Vec::<String>::new();

    let assembly = match &prop_name {
      PropName::Ident(ident) => std::format!("\"{}\"", ident.sym.to_string()),
      PropName::Str(str_) =>
      // TODO: Escaping
      {
        std::format!("\"{}\"", str_.value.to_string())
      }
      PropName::Num(num) =>
      // TODO: JS number stringification (different from rust)
      {
        std::format!("\"{}\"", num.value.to_string())
      }
      PropName::Computed(comp) => {
        // TODO: Always using a register is maybe not ideal
        // At the least, the assembly supports definitions and should
        // maybe support any value here
        let reg = self
          .fnc
          .reg_allocator
          .allocate_numbered(&"computed_key".to_string());
        let compiled = self.compile(&comp.expr, Some(reg.clone()));
        assert_eq!(compiled.nested_registers.len(), 0);
        nested_registers.push(reg.clone());

        std::format!("%{}", reg)
      }
      PropName::BigInt(bigint) => {
        std::format!("\"{}\"", bigint.value.to_string())
      }
    };

    CompiledExpression::new(assembly, nested_registers)
  }

  pub fn member_prop(
    &mut self,
    member_prop: &swc_ecma_ast::MemberProp,
    target_register: Option<String>,
  ) -> CompiledExpression {
    return match member_prop {
      swc_ecma_ast::MemberProp::Ident(ident) => {
        self.inline(string_literal(&ident.sym.to_string()), target_register)
      }
      swc_ecma_ast::MemberProp::Computed(computed) => self.compile(&computed.expr, target_register),
      swc_ecma_ast::MemberProp::PrivateName(private_name) => {
        self
          .fnc
          .todo(private_name.span(), "private name member property");

        CompiledExpression::empty()
      }
    };
  }

  pub fn member_expression(
    &mut self,
    member_exp: &swc_ecma_ast::MemberExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let compiled_obj = self.compile(&member_exp.obj, None);
    let compiled_prop = self.member_prop(&member_exp.prop, None);

    let (dest, nested_registers) = match &target_register {
      Some(tr) => ("%".to_string() + &tr, vec![]),
      None => {
        let reg = self
          .fnc
          .reg_allocator
          .allocate_numbered(&"_tmp".to_string());

        ("%".to_string() + &reg, vec![reg.clone()])
      }
    };

    let sub_instr = format!(
      "  sub {} {} {}",
      self.fnc.use_(compiled_obj),
      self.fnc.use_(compiled_prop),
      dest
    );

    self.fnc.definition.push(sub_instr);

    CompiledExpression::new(dest, nested_registers)
  }

  pub fn update_expression(
    &mut self,
    update_exp: &swc_ecma_ast::UpdateExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut target = TargetAccessor::compile(self, &update_exp.arg);

    let op_str = match update_exp.op {
      swc_ecma_ast::UpdateOp::PlusPlus => "op++",
      swc_ecma_ast::UpdateOp::MinusMinus => "op--",
    };

    let res = match update_exp.prefix {
      true => {
        self
          .fnc
          .definition
          .push(format!("  {} %{}", op_str, &target.register()));

        let mut nested_registers = Vec::<String>::new();

        let result_reg = match &target {
          TargetAccessor::Register(reg) => {
            for tr in &target_register {
              if tr != reg {
                self.fnc.definition.push(format!("  mov %{} %{}", reg, tr));
              }
            }

            reg.clone()
          }
          TargetAccessor::Nested(nta) => match target_register {
            Some(tr) => {
              self
                .fnc
                .definition
                .push(format!("  mov %{} %{}", nta.register, tr));

              tr
            }
            None => {
              let res = self
                .fnc
                .reg_allocator
                .allocate_numbered(&"_tmp".to_string());
              nested_registers.push(res.clone());

              self
                .fnc
                .definition
                .push(format!("  mov %{} %{}", nta.register, res));

              res
            }
          },
        };

        CompiledExpression::new(format!("%{}", result_reg), nested_registers)
      }
      false => {
        let mut nested_registers = Vec::<String>::new();

        let old_value_reg = match target_register {
          Some(tr) => tr,
          None => {
            let res = self
              .fnc
              .reg_allocator
              .allocate_numbered(&"_tmp".to_string());
            nested_registers.push(res.clone());

            res
          }
        };

        self
          .fnc
          .definition
          .push(format!("  mov %{} %{}", &target.register(), &old_value_reg));

        self
          .fnc
          .definition
          .push(format!("  {} %{}", op_str, &target.register()));

        CompiledExpression::new(format!("%{}", &old_value_reg), nested_registers)
      }
    };

    target.packup(self);

    return res;
  }

  pub fn call_expression(
    &mut self,
    call_exp: &swc_ecma_ast::CallExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<String>::new();
    let mut sub_nested_registers = Vec::<String>::new();

    let mut callee = match &call_exp.callee {
      swc_ecma_ast::Callee::Expr(expr) => self.compile(&*expr, None),
      _ => {
        self
          .fnc
          .todo(call_exp.callee.span(), "non-expression callee");

        CompiledExpression::empty()
      }
    };

    callee.release_checker.has_unreleased_registers = false;
    sub_nested_registers.append(&mut callee.nested_registers);

    let mut instr = "  call ".to_string();
    instr += &callee.value_assembly;
    instr += " [";

    for i in 0..call_exp.args.len() {
      let arg = &call_exp.args[i];

      if arg.spread.is_some() {
        self.fnc.todo(arg.spread.span(), "argument spreading");
      }

      let mut compiled_arg = self.compile(&*arg.expr, None);
      compiled_arg.release_checker.has_unreleased_registers = false;
      sub_nested_registers.append(&mut compiled_arg.nested_registers);

      instr += &compiled_arg.value_assembly;

      if i != call_exp.args.len() - 1 {
        instr += ", ";
      }
    }

    instr += "] ";

    let dest = match &target_register {
      Some(tr) => "%".to_string() + &tr,
      None => {
        let reg = self
          .fnc
          .reg_allocator
          .allocate_numbered(&"_tmp".to_string());
        nested_registers.push(reg.clone());

        "%".to_string() + &reg
      }
    };

    instr += &dest;

    self.fnc.definition.push(instr);

    for reg in sub_nested_registers {
      self.fnc.reg_allocator.release(&reg);
    }

    CompiledExpression::new(dest, nested_registers)
  }

  pub fn new_expression(
    &mut self,
    new_exp: &swc_ecma_ast::NewExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    // TODO: Try to deduplicate with call_expression

    let mut nested_registers = Vec::<String>::new();
    let mut sub_nested_registers = Vec::<String>::new();

    let mut callee = self.compile(&new_exp.callee, None);

    callee.release_checker.has_unreleased_registers = false;
    sub_nested_registers.append(&mut callee.nested_registers);

    let mut instr = "  new ".to_string();
    instr += &callee.value_assembly;
    instr += " [";

    for args in &new_exp.args {
      for i in 0..args.len() {
        let arg = &args[i];

        if arg.spread.is_some() {
          self.fnc.todo(arg.spread.span(), "argument spreading");
        }

        let mut compiled_arg = self.compile(&*arg.expr, None);
        sub_nested_registers.append(&mut compiled_arg.nested_registers);

        instr += &compiled_arg.value_assembly;

        if i != args.len() - 1 {
          instr += ", ";
        }
      }
    }

    instr += "] ";

    let dest = match &target_register {
      Some(tr) => "%".to_string() + &tr,
      None => {
        let reg = self
          .fnc
          .reg_allocator
          .allocate_numbered(&"_tmp".to_string());
        nested_registers.push(reg.clone());

        "%".to_string() + &reg
      }
    };

    instr += &dest;

    self.fnc.definition.push(instr);

    for reg in sub_nested_registers {
      self.fnc.reg_allocator.release(&reg);
    }

    CompiledExpression::new(dest, nested_registers)
  }

  pub fn method_call_expression(
    &mut self,
    callee_expr: &swc_ecma_ast::MemberExpr,
    args: &Vec<swc_ecma_ast::ExprOrSpread>,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<String>::new();
    let mut sub_nested_registers = Vec::<String>::new();

    enum TargetAccessorOrCompiledExpression {
      TargetAccessor(TargetAccessor),
      CompiledExpression(CompiledExpression),
    }

    let obj = match TargetAccessor::is_eligible_expr(self, &callee_expr.obj) {
      true => TargetAccessorOrCompiledExpression::TargetAccessor(TargetAccessor::compile(
        self,
        &callee_expr.obj,
      )),
      false => {
        TargetAccessorOrCompiledExpression::CompiledExpression(self.compile(&callee_expr.obj, None))
      }
    };

    let mut prop = self.member_prop(&callee_expr.prop, None);

    prop.release_checker.has_unreleased_registers = false;
    sub_nested_registers.append(&mut prop.nested_registers);

    let mut instr = format!(
      "  subcall {} {} [",
      match &obj {
        TargetAccessorOrCompiledExpression::TargetAccessor(ta) => format!("%{}", ta.register()),
        TargetAccessorOrCompiledExpression::CompiledExpression(ce) => ce.value_assembly.clone(),
      },
      prop.value_assembly,
    );

    for i in 0..args.len() {
      let arg = &args[i];

      if arg.spread.is_some() {
        self.fnc.todo(arg.spread.span(), "argument spreading");
      }

      let mut compiled_arg = self.compile(&*arg.expr, None);
      compiled_arg.release_checker.has_unreleased_registers = false;
      sub_nested_registers.append(&mut compiled_arg.nested_registers);

      instr += &compiled_arg.value_assembly;

      if i != args.len() - 1 {
        instr += ", ";
      }
    }

    instr += "] ";

    let dest = match &target_register {
      Some(tr) => "%".to_string() + &tr,
      None => {
        let reg = self
          .fnc
          .reg_allocator
          .allocate_numbered(&"_tmp".to_string());
        nested_registers.push(reg.clone());

        "%".to_string() + &reg
      }
    };

    instr += &dest;

    self.fnc.definition.push(instr);

    match obj {
      TargetAccessorOrCompiledExpression::TargetAccessor(mut ta) => {
        ta.packup(self);
      }
      TargetAccessorOrCompiledExpression::CompiledExpression(ce) => {
        self.fnc.use_(ce);
      }
    }

    for reg in sub_nested_registers {
      self.fnc.reg_allocator.release(&reg);
    }

    CompiledExpression::new(dest, nested_registers)
  }

  pub fn fn_expression(
    &mut self,
    fn_: &swc_ecma_ast::FnExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let fn_name = fn_
      .ident
      .clone()
      .and_then(|ident| Some(ident.sym.to_string()));

    let definition_name = match &fn_name {
      Some(name) => self.fnc.definition_allocator.borrow_mut().allocate(&name),
      None => self
        .fnc
        .definition_allocator
        .borrow_mut()
        .allocate_numbered(&"_anon".to_string()),
    };

    let mut cf = CaptureFinder::new(self.scope.clone());
    cf.fn_expr(&init_std_scope(), fn_);

    self
      .fnc
      .queue
      .add(QueuedFunction {
        definition_name: definition_name.clone(),
        fn_name: fn_name.clone(),
        capture_params: cf.ordered_names.clone(),
        functionish: Functionish::Fn(fn_.function.clone()),
      })
      .expect("Failed to queue function");

    if cf.ordered_names.len() == 0 {
      return self.inline(format!("@{}", definition_name), target_register);
    }

    return self.capturing_fn_ref(
      fn_name,
      fn_.ident.span(),
      &definition_name,
      &cf.ordered_names,
      target_register,
    );
  }

  pub fn arrow_expression(
    &mut self,
    arrow_expr: &swc_ecma_ast::ArrowExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let definition_name = self
      .fnc
      .definition_allocator
      .borrow_mut()
      .allocate_numbered(&"_anon".to_string());

    let mut cf = CaptureFinder::new(self.scope.clone());
    cf.arrow_expr(&init_std_scope(), arrow_expr);

    self
      .fnc
      .queue
      .add(QueuedFunction {
        definition_name: definition_name.clone(),
        fn_name: None,
        capture_params: cf.ordered_names.clone(),
        functionish: Functionish::Arrow(arrow_expr.clone()),
      })
      .expect("Failed to queue function");

    if cf.ordered_names.len() == 0 {
      return self.inline(format!("@{}", definition_name), target_register);
    }

    return self.capturing_fn_ref(
      None,
      arrow_expr.span(),
      &definition_name,
      &cf.ordered_names,
      target_register,
    );
  }

  pub fn capturing_fn_ref(
    &mut self,
    fn_name: Option<String>,
    span: swc_common::Span,
    definition_name: &String,
    captures: &Vec<String>,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<String>::new();
    let mut sub_nested_registers = Vec::<String>::new();

    let reg = match target_register {
      None => {
        let alloc_reg = match &fn_name {
          Some(name) => self.fnc.reg_allocator.allocate(&name),
          None => self
            .fnc
            .reg_allocator
            .allocate_numbered(&"_anon".to_string()),
        };

        nested_registers.push(alloc_reg.clone());

        alloc_reg
      }
      Some(tr) => tr.clone(),
    };

    let mut bind_instr = format!("  bind @{} [", definition_name);

    for i in 0..captures.len() {
      let captured_name = &captures[i];

      if i > 0 {
        bind_instr += ", ";
      }

      bind_instr += &match self.scope.get(captured_name) {
        None => {
          self.fnc.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::InternalError,
            message: format!(
              "Failed to resolve captured name {} (captured names should always resolve)",
              captured_name
            ),
            span,
          });

          let reg = self
            .fnc
            .reg_allocator
            .allocate_numbered(&format!("_failed_cap_{}", captured_name).to_string());

          format!("%{}", reg)
        }
        Some(MappedName::Definition(_)) => {
          self.fnc.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::InternalError,
            message: format!(
              "Captured name {} resolved to a definition (this should never happen)",
              captured_name
            ),
            span,
          });

          let reg = self
            .fnc
            .reg_allocator
            .allocate_numbered(&format!("_failed_cap_{}", captured_name).to_string());

          format!("%{}", reg)
        }
        Some(MappedName::Register(cap_reg)) => format!("%{}", cap_reg),
        Some(MappedName::QueuedFunction(qfn)) => {
          let mut compiled_ref = self.capturing_fn_ref(
            qfn.fn_name.clone(),
            match &qfn.functionish {
              Functionish::Fn(fn_) => fn_.span,
              Functionish::Arrow(arrow) => arrow.span,
              Functionish::Constructor(_, constructor) => constructor.span,
            },
            &qfn.definition_name,
            &qfn.capture_params,
            None,
          );

          compiled_ref.release_checker.has_unreleased_registers = false;
          sub_nested_registers.append(&mut compiled_ref.nested_registers);

          compiled_ref.value_assembly
        }
        Some(MappedName::Builtin(_)) => {
          self.fnc.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::InternalError,
            message: format!(
              "Captured name {} resolved to a builtin (this should never happen)",
              captured_name
            ),
            span,
          });

          let reg = self
            .fnc
            .reg_allocator
            .allocate_numbered(&format!("_failed_cap_{}", captured_name).to_string());

          format!("%{}", reg)
        }
      };
    }

    bind_instr += &format!("] %{}", reg);
    self.fnc.definition.push(bind_instr);

    for reg in sub_nested_registers {
      self.fnc.reg_allocator.release(&reg);
    }

    return CompiledExpression::new(format!("%{}", reg), nested_registers);
  }

  pub fn template_literal(
    &mut self,
    tpl: &swc_ecma_ast::Tpl,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let len = tpl.exprs.len();

    assert_eq!(tpl.quasis.len(), len + 1);

    if len == 0 {
      return self.inline(
        string_literal(&tpl.quasis[0].raw.to_string()),
        target_register,
      );
    }

    let mut nested_registers = Vec::<String>::new();

    let acc_reg = match target_register {
      Some(tr) => tr,
      None => {
        let reg = self
          .fnc
          .reg_allocator
          .allocate_numbered(&"_tmp".to_string());
        nested_registers.push(reg.clone());

        reg
      }
    };

    let first_expr = self.compile(&tpl.exprs[0], None);

    let plus_instr = format!(
      "  op+ {} {} %{}",
      string_literal(&tpl.quasis[0].raw.to_string()),
      self.fnc.use_(first_expr),
      acc_reg,
    );

    self.fnc.definition.push(plus_instr);

    for i in 1..len {
      self.fnc.definition.push(format!(
        "  op+ %{} {} %{}",
        acc_reg,
        string_literal(&tpl.quasis[i].raw.to_string()),
        acc_reg,
      ));

      let expr_i = self.compile(&tpl.exprs[i], None);

      let plus_instr = format!("  op+ %{} {} %{}", acc_reg, self.fnc.use_(expr_i), acc_reg);

      self.fnc.definition.push(plus_instr);
    }

    let last_str = tpl.quasis[len].raw.to_string();

    if last_str != "" {
      self.fnc.definition.push(format!(
        "  op+ %{} {} %{}",
        acc_reg,
        string_literal(&last_str),
        acc_reg,
      ));
    }

    return CompiledExpression::new(format!("%{}", acc_reg), nested_registers);
  }

  pub fn literal(
    &mut self,
    lit: &swc_ecma_ast::Lit,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let compiled_literal = self.compile_literal(lit);
    return self.inline(compiled_literal, target_register);
  }

  pub fn inline(
    &mut self,
    value_assembly: String,
    target_register: Option<String>,
  ) -> CompiledExpression {
    return match target_register {
      None => CompiledExpression::new(value_assembly, vec![]),
      Some(t) => {
        let mut instr = "  mov ".to_string();
        instr += &value_assembly;
        instr += " %";
        instr += &t;
        self.fnc.definition.push(instr);

        CompiledExpression::new(std::format!("%{}", t), vec![])
      }
    };
  }

  pub fn identifier(
    &mut self,
    ident: &swc_ecma_ast::Ident,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let ident_string = ident.sym.to_string();

    if ident_string == "undefined" {
      return self.inline("undefined".to_string(), target_register);
    }

    let mapped = self
      .scope
      .get(&ident_string)
      .expect(&format!("Identifier not found in scope {:?}", ident.span()));

    return match mapped {
      MappedName::Register(reg) => self.inline("%".to_string() + &reg, target_register),
      MappedName::Definition(def) => self.inline("@".to_string() + &def, target_register),
      MappedName::QueuedFunction(qfn) => self.capturing_fn_ref(
        qfn.fn_name.clone(),
        match &qfn.functionish {
          Functionish::Fn(fn_) => fn_.span,
          Functionish::Arrow(arrow) => arrow.span,
          Functionish::Constructor(_, constructor) => constructor.span,
        },
        &qfn.definition_name,
        &qfn.capture_params,
        target_register,
      ),
      MappedName::Builtin(builtin) => self.inline(format!("${}", builtin), target_register),
    };
  }

  pub fn compile_literal(&mut self, lit: &swc_ecma_ast::Lit) -> String {
    use swc_ecma_ast::Lit::*;

    let (todo_name, message) = match lit {
      Str(str_) => return string_literal(&str_.value.to_string()),
      Bool(bool_) => return bool_.value.to_string(),
      Null(_) => return "null".to_string(),
      Num(num) => return num.value.to_string(),
      BigInt(_) => ("_todo_bigint_literal", "BigInt literals"),
      Regex(_) => ("_todo_regex_literal", "Regex literals"),
      JSXText(_) => ("_todo_jsxtext_literal", "JSXText literals"),
    };

    self.fnc.todo(lit.span(), message);

    let todo_reg = self
      .fnc
      .reg_allocator
      .allocate_numbered(&todo_name.to_string());

    return format!("%{}", todo_reg);
  }
}

pub fn string_literal(str_: &String) -> String {
  return format!("\"{}\"", str_); // TODO: Escaping
}

pub fn get_binary_op_str(op: swc_ecma_ast::BinaryOp) -> &'static str {
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

pub fn get_unary_op_str(op: swc_ecma_ast::UnaryOp) -> Option<&'static str> {
  use swc_ecma_ast::UnaryOp::*;

  return match op {
    Minus => Some("unary-"),
    Plus => Some("unary+"),
    Bang => Some("op!"),
    Tilde => Some("op~"),
    TypeOf => Some("typeof"),
    Void => None,   // TODO
    Delete => None, // TODO
  };
}

pub fn get_assign_op_str(op: swc_ecma_ast::AssignOp) -> Option<&'static str> {
  use swc_ecma_ast::AssignOp::*;

  return match op {
    Assign => None,
    AddAssign => Some("op+"),
    SubAssign => Some("op-"),
    MulAssign => Some("op*"),
    DivAssign => Some("op/"),
    ModAssign => Some("op%"),
    LShiftAssign => Some("op<<"),
    RShiftAssign => Some("op>>"),
    ZeroFillRShiftAssign => Some("op>>>"),
    BitOrAssign => Some("op|"),
    BitXorAssign => Some("op^"),
    BitAndAssign => Some("op&"),
    ExpAssign => Some("op**"),
    AndAssign => Some("op&&"),
    OrAssign => Some("op||"),
    NullishAssign => Some("op??"),
  };
}

struct NestedTargetAccess {
  obj: Box<TargetAccessor>,
  subscript: CompiledExpression,
  register: String,
}

enum TargetAccessor {
  Register(String),
  Nested(NestedTargetAccess),
}

impl TargetAccessor {
  fn is_eligible_expr(ec: &mut ExpressionCompiler, expr: &swc_ecma_ast::Expr) -> bool {
    use swc_ecma_ast::Expr::*;

    return match expr {
      Ident(ident) => match ec.scope.get(&ident.sym.to_string()) {
        None => false,
        Some(MappedName::Definition(_)) => false,
        Some(MappedName::QueuedFunction(_)) => false,
        Some(MappedName::Register(_)) => true,
        Some(MappedName::Builtin(_)) => false,
      },
      This(_) => true,
      Member(member) => TargetAccessor::is_eligible_expr(ec, &member.obj),
      _ => false, // TODO: Others may be eligible but not implemented?
    };
  }

  fn compile(ec: &mut ExpressionCompiler, expr: &swc_ecma_ast::Expr) -> TargetAccessor {
    use swc_ecma_ast::Expr::*;

    return match expr {
      Ident(ident) => match ec.scope.get(&ident.sym.to_string()) {
        None => {
          ec.fnc.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            span: ident.span,
            message: format!("Unresolved identifier: {}", ident.sym.to_string()),
          });

          TargetAccessor::make_bad(ec)
        }
        Some(MappedName::Definition(def_name)) => {
          ec.fnc.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            span: ident.span,
            message: format!("Cannot assign to definition: {}", def_name),
          });

          TargetAccessor::make_bad(ec)
        }
        Some(MappedName::QueuedFunction(qfn)) => {
          ec.fnc.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            span: ident.span,
            message: format!("Cannot assign to function: {}", qfn.definition_name),
          });

          TargetAccessor::make_bad(ec)
        }
        Some(MappedName::Register(reg)) => TargetAccessor::Register(reg),
        // Some(MappedName::Builtin(_)) => None,
        Some(MappedName::Builtin(builtin)) => {
          ec.fnc.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            span: ident.span,
            message: format!("Cannot assign to builtin: {}", builtin),
          });

          TargetAccessor::make_bad(ec)
        }
      },
      This(_) => TargetAccessor::Register("this".to_string()),
      Member(member) => {
        let obj = TargetAccessor::compile(ec, &member.obj);
        let subscript = ec.member_prop(&member.prop, None);

        let register = ec.fnc.reg_allocator.allocate_numbered(&"_tmp".to_string());

        ec.fnc.definition.push(format!(
          "  sub %{} {} %{}",
          obj.register(),
          subscript.value_assembly,
          register,
        ));

        TargetAccessor::Nested(NestedTargetAccess {
          obj: Box::new(obj),
          subscript: subscript,
          register: register,
        })
      }
      SuperProp(super_prop) => {
        ec.fnc.todo(super_prop.span, "SuperProp expressions");
        TargetAccessor::make_todo(ec)
      }
      _ => {
        ec.fnc.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          span: expr.span(),
          message: format!("Invalid target {}", get_expr_type_str(expr)),
        });

        TargetAccessor::make_bad(ec)
      }
    };
  }

  fn make_bad(ec: &mut ExpressionCompiler) -> TargetAccessor {
    return TargetAccessor::Register(
      ec.fnc
        .reg_allocator
        .allocate_numbered(&"_bad_lvalue".to_string()),
    );
  }

  fn make_todo(ec: &mut ExpressionCompiler) -> TargetAccessor {
    return TargetAccessor::Register(
      ec.fnc
        .reg_allocator
        .allocate_numbered(&"_todo_lvalue".to_string()),
    );
  }

  fn register(&self) -> String {
    use TargetAccessor::*;

    return match self {
      Register(reg) => reg.clone(),
      Nested(nta) => nta.register.clone(),
    };
  }

  fn packup(&mut self, ec: &mut ExpressionCompiler) {
    use TargetAccessor::*;

    match self {
      Register(_) => {}
      Nested(nta) => {
        let submov_instr = format!(
          "  submov {} %{} %{}",
          ec.fnc.use_ref(&mut nta.subscript),
          &nta.register,
          nta.obj.register(),
        );

        ec.fnc.definition.push(submov_instr);

        ec.fnc.reg_allocator.release(&nta.register);

        nta.obj.packup(ec);
      }
    }
  }
}

fn get_expr_type_str(expr: &swc_ecma_ast::Expr) -> &'static str {
  use swc_ecma_ast::Expr::*;

  return match expr {
    This(_) => "This",
    Ident(_) => "Ident",
    Array(_) => "Array",
    Object(_) => "Object",
    Fn(_) => "Fn",
    Unary(_) => "Unary",
    Update(_) => "Update",
    Bin(_) => "Bin",
    Assign(_) => "Assign",
    Seq(_) => "Seq",
    Cond(_) => "Cond",
    Call(_) => "Call",
    Member(_) => "Member",
    New(_) => "New",
    Paren(_) => "Paren",
    Arrow(_) => "Arrow",
    Yield(_) => "Yield",
    Await(_) => "Await",
    Lit(_) => "Lit",
    Tpl(_) => "Tpl",
    TaggedTpl(_) => "TaggedTpl",
    Class(_) => "Class",
    MetaProp(_) => "MetaProp",
    Invalid(_) => "Invalid",
    TsTypeAssertion(_) => "TsTypeAssertion",
    TsConstAssertion(_) => "TsConstAssertion",
    TsNonNull(_) => "TsNonNull",
    TsAs(_) => "TsAs",
    OptChain(_) => "OptChain",
    PrivateName(_) => "PrivateName",
    SuperProp(_) => "SuperProp",
    JSXMember(_) => "JSXMember",
    JSXNamespacedName(_) => "JSXNamespacedName",
    JSXEmpty(_) => "JSXEmpty",
    JSXElement(_) => "JSXElement",
    JSXFragment(_) => "JSXFragment",
    TsInstantiation(_) => "TsInstantiation",
  };
}
