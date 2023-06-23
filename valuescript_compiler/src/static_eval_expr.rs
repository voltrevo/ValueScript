use crate::{
  asm::{Builtin, Value},
  expression_compiler::value_from_literal,
};

pub fn static_eval_expr(expr: &swc_ecma_ast::Expr) -> Option<Value> {
  let symbol_iterator_opt = as_symbol_iterator(expr);

  if symbol_iterator_opt.is_some() {
    return symbol_iterator_opt;
  }

  match expr {
    swc_ecma_ast::Expr::Lit(lit) => match value_from_literal(lit) {
      Ok(value) => return Some(value),
      _ => {}
    },
    _ => {} // TODO: Array, Object
  }

  None
}

fn as_symbol_iterator(expr: &swc_ecma_ast::Expr) -> Option<Value> {
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
