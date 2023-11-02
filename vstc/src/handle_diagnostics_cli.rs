use std::{collections::HashMap, path::PathBuf};

use serde_qs as qs;
use url::Url;

use valuescript_compiler::{Diagnostic, DiagnosticLevel};

pub fn handle_diagnostics_cli(file_path: &String, diagnostics: &Vec<Diagnostic>) {
  let path = 'b: {
    if file_path == "(str)" {
      // TODO: Fix this hack
      break 'b None;
    }

    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let abs_path = PathBuf::from(file_path);

    Some(match abs_path.strip_prefix(&current_dir) {
      Ok(p) => p.into(),
      Err(_) => abs_path,
    })
  };

  let path_str = match path {
    Some(path) => path.to_string_lossy().to_string(),
    None => file_path.clone(),
  };

  let mut level_counts = HashMap::<DiagnosticLevel, usize>::new();

  let text = if file_path == "(str)" {
    None
  } else {
    Some(std::fs::read_to_string(file_path).unwrap())
  };

  let mut lines = Vec::<String>::new();

  for diagnostic in diagnostics {
    let (line, col) = match &text {
      Some(text) => pos_to_line_col(text, diagnostic.span.lo.0),
      None => (0, 0),
    };

    let line = format!(
      "{}:{}:{}: {}: {}",
      path_str, line, col, diagnostic.level, diagnostic.message
    );

    println!("{}", line);

    lines.push(line);

    let count = level_counts.entry(diagnostic.level).or_insert(0);
    *count += 1;
  }

  let error_count = level_counts.get(&DiagnosticLevel::Error).unwrap_or(&0);

  let internal_error_count = level_counts
    .get(&DiagnosticLevel::InternalError)
    .unwrap_or(&0);

  let total_error_count = error_count + internal_error_count;

  if total_error_count > 0 {
    println!("\nFailed with {} error(s)", total_error_count);
  }

  if internal_error_count > &0 {
    println!();
    println!("===============================");
    println!("=== INTERNAL ERROR(S) FOUND ===");
    println!("===============================");
    println!();

    // Create a github issue link
    let mut url = Url::parse("https://github.com/voltrevo/ValueScript/issues/new").unwrap();

    #[derive(serde::Serialize)]
    struct TitleAndBody {
      title: String,
      body: String,
    }

    let query_string = qs::to_string(&TitleAndBody {
      title: "Internal error(s) found".to_string(),
      body: format!(
        "Input:\n```\n(Please provide if you can)\n```\n\nOutput:\n```\n{}\n```",
        lines.join("\n")
      ),
    })
    .unwrap();

    url.set_query(Some(&query_string));

    println!("This is a bug in ValueScript, please consider reporting it:");
    println!();
    println!("{}", url);
    println!();
  }

  if total_error_count > 0 {
    std::process::exit(1);
  }
}

fn pos_to_line_col(text: &str, pos: u32) -> (u32, u32) {
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

  (line, col)
}
