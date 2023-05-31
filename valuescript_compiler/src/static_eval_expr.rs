use crate::asm::{Builtin, Value};

pub fn static_eval_expr(expr: &swc_ecma_ast::Expr) -> Option<Value> {
  let member_expr = match expr {
    swc_ecma_ast::Expr::Member(member_expr) => member_expr,
    _ => return None,
  };

  match &*member_expr.obj {
    swc_ecma_ast::Expr::Ident(ident) => {
      if ident.sym.to_string() != "Symbol" {
        return None;
      }
    }
    _ => return None,
  };

  match &member_expr.prop {
    swc_ecma_ast::MemberProp::Ident(ident) => {
      if ident.sym.to_string() != "iterator" {
        return None;
      }
    }
    _ => return None,
  }

  return Some(Value::Builtin(Builtin {
    name: "SymbolIterator".to_string(),
  }));
}
