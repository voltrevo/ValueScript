use swc_common::Spanned;

use crate::{
  asm::{Number, Object, Value},
  static_eval_expr::static_eval_expr,
  Diagnostic, DiagnosticLevel,
};

pub fn compile_enum_value(
  ts_enum: &swc_ecma_ast::TsEnumDecl,
  diagnostics: &mut Vec<Diagnostic>,
) -> Value {
  let mut properties = Vec::<(Value, Value)>::new();
  let mut next_default_id: Option<f64> = Some(0.0);

  for member in &ts_enum.members {
    let key = match &member.id {
      swc_ecma_ast::TsEnumMemberId::Ident(ident) => ident.sym.to_string(),
      swc_ecma_ast::TsEnumMemberId::Str(str) => str.value.to_string(),
    };

    let init_value = match &member.init {
      Some(init) => match static_eval_expr(&init) {
        Some(init_value) => match init_value {
          Value::Number(Number(n)) => {
            next_default_id = Some(n + 1.0);
            Some(Value::Number(Number(n)))
          }
          Value::String(_) => Some(init_value),
          _ => None,
        },
        None => {
          diagnostics.push(Diagnostic {
            level: DiagnosticLevel::InternalError,
            message: "TODO: Static eval failed".to_string(),
            span: init.span(),
          });

          None
        }
      },
      None => None,
    };

    let value = match init_value {
      Some(value) => value,
      None => {
        let id = match next_default_id {
          Some(id) => id,
          None => {
            diagnostics.push(Diagnostic {
              level: DiagnosticLevel::Error,
              message: format!("Missing required initializer"),
              span: member.span,
            });

            0.0
          }
        };

        let value = Value::Number(Number(id));
        next_default_id = Some(id + 1.0);

        value
      }
    };

    properties.push((Value::String(key.clone()), value.clone()));
    properties.push((value, Value::String(key)));
  }

  Value::Object(Box::new(Object { properties }))
}