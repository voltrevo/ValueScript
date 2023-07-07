use std::fmt;

#[derive(serde::Serialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
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

#[derive(serde::Serialize, Debug)]
pub struct Diagnostic {
  pub level: DiagnosticLevel,
  pub message: String,
  pub span: swc_common::Span,
}

impl Diagnostic {
  pub fn from_swc(swc_diagnostic: &swc_common::errors::Diagnostic) -> Option<Diagnostic> {
    use swc_common::errors::Level;

    let level = match swc_diagnostic.level {
      Level::Bug => DiagnosticLevel::InternalError,
      Level::Fatal => DiagnosticLevel::Error,
      Level::PhaseFatal => DiagnosticLevel::Error,
      Level::Error => DiagnosticLevel::Error,
      Level::Warning => DiagnosticLevel::Lint,
      Level::Note => return None,
      Level::Help => return None,
      Level::Cancelled => return None,
      Level::FailureNote => return None,
    };

    Some(Diagnostic {
      level,
      message: swc_diagnostic.message(),
      span: swc_diagnostic
        .span
        .primary_span()
        .unwrap_or(swc_common::DUMMY_SP),
    })
  }
}
