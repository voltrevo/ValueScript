use std::fmt;

pub enum DiagnosticLevel {
  Lint,
  Error,
  InternalError,
  CompilerDebug,
}

impl fmt::Display for DiagnosticLevel {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      DiagnosticLevel::Lint => write!(f, "Lint"),
      DiagnosticLevel::Error => write!(f, "Error"),
      DiagnosticLevel::InternalError => write!(f, "Internal Error"),
      DiagnosticLevel::CompilerDebug => write!(f, "Compiler Debug"),
    }
  }
}

pub struct Diagnostic {
  pub level: DiagnosticLevel,
  pub message: String,
  pub span: swc_common::Span,
}

pub fn handle_diagnostics_cli(file_path: &String, diagnostics: &Vec<Diagnostic>) {
  let mut has_error = false;

  let text = std::fs::read_to_string(file_path).unwrap();

  for diagnostic in diagnostics {
    let (line, col) = pos_to_line_col(&text, diagnostic.span.lo.0);

    println!(
      "{}:{}:{}: {}: {}",
      file_path, line, col, diagnostic.level, diagnostic.message
    );

    match diagnostic.level {
      DiagnosticLevel::Error | DiagnosticLevel::InternalError => {
        has_error = true;
      }
      DiagnosticLevel::Lint => {}
      DiagnosticLevel::CompilerDebug => {}
    }
  }

  if has_error {
    std::process::exit(1);
  }
}

fn pos_to_line_col(text: &String, pos: u32) -> (u32, u32) {
  let mut line = 1u32;
  let mut col = 1u32;

  for (i, c) in text.chars().enumerate() {
    if i as u32 == pos {
      break;
    }

    if c == '\n' {
      line += 1;
      col = 1;
    } else {
      col += 1;
    }
  }

  return (line, col);
}
