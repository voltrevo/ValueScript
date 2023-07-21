use std::mem::take;

use crate::{
  asm::{Instruction, Register, Value},
  expression_compiler::{CompiledExpression, ExpressionCompiler},
  ident::Ident as CrateIdent,
  Diagnostic, DiagnosticLevel,
};
use swc_common::Spanned;

pub struct NestedTargetAccess {
  pub obj: Box<TargetAccessor>,
  pub subscript: CompiledExpression,
  pub register: Register,
}

pub enum TargetAccessor {
  Register(Register),
  Nested(NestedTargetAccess),
}

impl TargetAccessor {
  pub fn is_eligible_expr(ec: &mut ExpressionCompiler, expr: &swc_ecma_ast::Expr) -> bool {
    use swc_ecma_ast::Expr::*;

    return match expr {
      Ident(ident) => match ec.fnc.lookup(&crate::ident::Ident::from_swc_ident(ident)) {
        Some(name) => !name.effectively_const,
        _ => false, // TODO: InternalError?
      },
      This(_) => true,
      Member(member) => TargetAccessor::is_eligible_expr(ec, &member.obj),

      Paren(e) => TargetAccessor::is_eligible_expr(ec, &e.expr),
      TsTypeAssertion(e) => TargetAccessor::is_eligible_expr(ec, &e.expr),
      //
      // Deliberately excluding this so you can get a const_subcall using (x as const).foo().
      // This is TypeScript syntax which is actually affecting the program behavior though.
      // TsConstAssertion(e) => TargetAccessor::is_eligible_expr(ec, &e.expr),
      // TODO: TypeScript often doesn't allow (x as const). Maybe this should be included after all?
      // (You can always do echo(x).foo() to get a const_subcall.)
      //
      TsNonNull(e) => TargetAccessor::is_eligible_expr(ec, &e.expr),
      TsAs(e) => TargetAccessor::is_eligible_expr(ec, &e.expr),
      //
      // TODO: OptChain
      //
      _ => false, // TODO: Others may be eligible but not implemented?
    };
  }

  pub fn compile(
    ec: &mut ExpressionCompiler,
    expr: &swc_ecma_ast::Expr,
    is_outermost: bool,
  ) -> TargetAccessor {
    use swc_ecma_ast::Expr::*;

    return match expr {
      Ident(ident) => TargetAccessor::compile_ident(ec, &CrateIdent::from_swc_ident(ident)),
      This(this) => TargetAccessor::compile_ident(ec, &CrateIdent::this(this.span)),
      Member(member) => {
        let obj = TargetAccessor::compile(ec, &member.obj, false);
        let subscript = ec.member_prop(&member.prop, None);

        let register = ec.fnc.allocate_tmp();

        if !is_outermost {
          ec.fnc.push(Instruction::Sub(
            Value::Register(obj.register()),
            subscript.value.clone(),
            register.clone(),
          ));
        }

        TargetAccessor::Nested(NestedTargetAccess {
          obj: Box::new(obj),
          subscript,
          register,
        })
      }

      Paren(e) => TargetAccessor::compile(ec, &e.expr, false),
      TsTypeAssertion(e) => TargetAccessor::compile(ec, &e.expr, false),
      TsNonNull(e) => TargetAccessor::compile(ec, &e.expr, false),
      TsAs(e) => TargetAccessor::compile(ec, &e.expr, false),

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

  pub fn compile_ident(ec: &mut ExpressionCompiler, ident: &CrateIdent) -> TargetAccessor {
    TargetAccessor::Register(ec.get_register_for_ident_mutation(ident))
  }

  pub fn make_bad(ec: &mut ExpressionCompiler) -> TargetAccessor {
    TargetAccessor::Register(ec.fnc.allocate_numbered_reg("_bad_lvalue"))
  }

  pub fn make_todo(ec: &mut ExpressionCompiler) -> TargetAccessor {
    TargetAccessor::Register(ec.fnc.allocate_numbered_reg("_todo_lvalue"))
  }

  pub fn assign_and_packup(
    &mut self,
    ec: &mut ExpressionCompiler,
    value: &Value,
    uses_this_subcall: bool,
  ) {
    use TargetAccessor::*;

    match self {
      Register(reg) => {
        // TODO: Should value just derive from Eq?
        if value.to_string() != reg.to_string() {
          ec.fnc.push(Instruction::Mov(value.clone(), reg.clone()));
        }
      }
      Nested(nta) => {
        let submov_instr = Instruction::SubMov(
          nta.subscript.value.clone(),
          value.clone(),
          nta.obj.register(),
        );

        if uses_this_subcall {
          // This avoids require_mutable_this when packing up a this_subcall. Technically it will
          // still assign to %this, but we've protected against the actual mutation because if %this
          // is const, then this_subcall won't allow its mutation.
          ec.fnc.push_raw(submov_instr);
        } else {
          ec.fnc.push(submov_instr);
        }

        ec.fnc.release_ce(take(&mut nta.subscript));
        ec.fnc.release_reg(&nta.register);

        nta.obj.packup(ec, uses_this_subcall);
      }
    }
  }

  pub fn read(&self, ec: &mut ExpressionCompiler) -> Register {
    use TargetAccessor::*;

    match self {
      Register(reg) => reg.clone(),
      Nested(nta) => {
        ec.fnc.push(Instruction::Sub(
          Value::Register(nta.obj.register()),
          nta.subscript.value.clone(),
          nta.register.clone(),
        ));

        nta.register.clone()
      }
    }
  }

  pub fn register(&self) -> Register {
    match self {
      TargetAccessor::Register(reg) => reg.clone(),
      TargetAccessor::Nested(nta) => nta.register.clone(),
    }
  }

  pub fn direct_register(&self) -> Option<Register> {
    match self {
      TargetAccessor::Register(reg) => Some(reg.clone()),
      TargetAccessor::Nested(_) => None,
    }
  }

  pub fn packup(&mut self, ec: &mut ExpressionCompiler, uses_this_subcall: bool) {
    use TargetAccessor::*;

    match self {
      Register(_) => {}
      Nested(nta) => {
        let submov_instr = Instruction::SubMov(
          nta.subscript.value.clone(),
          Value::Register(nta.register.clone()),
          nta.obj.register(),
        );

        if uses_this_subcall {
          ec.fnc.push_raw(submov_instr);
        } else {
          ec.fnc.push(submov_instr);
        }

        ec.fnc.release_ce(take(&mut nta.subscript));
        ec.fnc.release_reg(&nta.register);

        nta.obj.packup(ec, uses_this_subcall);
      }
    }
  }

  pub fn targets_this(&self) -> bool {
    match self {
      TargetAccessor::Register(reg) => reg == &Register::this(),
      TargetAccessor::Nested(nta) => nta.obj.targets_this(),
    }
  }
}

pub fn get_expr_type_str(expr: &swc_ecma_ast::Expr) -> &'static str {
  use swc_ecma_ast::Expr::*;

  match expr {
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
  }
}
