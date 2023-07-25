use std::{cell::RefCell, fmt};

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

  pub fn todo(span: swc_common::Span, message: &str) -> Self {
    Diagnostic {
      level: DiagnosticLevel::InternalError,
      message: format!("TODO: {}", message),
      span,
    }
  }

  pub fn error(span: swc_common::Span, message: &str) -> Self {
    Diagnostic {
      level: DiagnosticLevel::Error,
      message: message.to_string(),
      span,
    }
  }

  pub fn internal_error(span: swc_common::Span, message: &str) -> Self {
    Diagnostic {
      level: DiagnosticLevel::InternalError,
      message: message.to_string(),
      span,
    }
  }

  pub fn not_supported(span: swc_common::Span, message: &str) -> Self {
    Diagnostic {
      level: DiagnosticLevel::Error,
      message: format!("Not supported: {}", message),
      span,
    }
  }

  pub fn lint(span: swc_common::Span, message: &str) -> Self {
    Diagnostic {
      level: DiagnosticLevel::Lint,
      message: message.to_string(),
      span,
    }
  }
}

pub trait DiagnosticContainer {
  fn diagnostics_mut(&self) -> &RefCell<Vec<Diagnostic>>;
}

pub trait DiagnosticReporter {
  fn todo(&self, span: swc_common::Span, message: &str);
  fn error(&self, span: swc_common::Span, message: &str);
  fn internal_error(&self, span: swc_common::Span, message: &str);
  fn not_supported(&self, span: swc_common::Span, message: &str);
  fn lint(&self, span: swc_common::Span, message: &str);
}

impl<T> DiagnosticReporter for T
where
  T: DiagnosticContainer,
{
  fn todo(&self, span: swc_common::Span, message: &str) {
    self
      .diagnostics_mut()
      .borrow_mut()
      .push(Diagnostic::todo(span, message));
  }

  fn error(&self, span: swc_common::Span, message: &str) {
    self
      .diagnostics_mut()
      .borrow_mut()
      .push(Diagnostic::error(span, message));
  }

  fn internal_error(&self, span: swc_common::Span, message: &str) {
    self
      .diagnostics_mut()
      .borrow_mut()
      .push(Diagnostic::internal_error(span, message));
  }

  fn not_supported(&self, span: swc_common::Span, message: &str) {
    self
      .diagnostics_mut()
      .borrow_mut()
      .push(Diagnostic::not_supported(span, message));
  }

  fn lint(&self, span: swc_common::Span, message: &str) {
    self
      .diagnostics_mut()
      .borrow_mut()
      .push(Diagnostic::lint(span, message));
  }
}
