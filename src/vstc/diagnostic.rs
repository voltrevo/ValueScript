pub enum DiagnosticLevel {
  Lint,
  Error,
  InternalError,
}

pub struct Diagnostic {
  pub level: DiagnosticLevel,
  pub message: String,
  pub span: swc_common::Span,
}

pub fn handle_diagnostics_cli(diagnostics: &Vec<Diagnostic>) {
  let mut has_error = false;

  for diagnostic in diagnostics {
    match diagnostic.level {
      DiagnosticLevel::Lint => {
        println!("Lint: {}", diagnostic.message);
      }
      DiagnosticLevel::Error => {
        println!("Error: {}", diagnostic.message);
        has_error = true;
      }
      DiagnosticLevel::InternalError => {
        println!("Internal Error: {}", diagnostic.message);
        has_error = true;
      }
    }
  }

  if has_error {
    std::process::exit(1);
  }
}
