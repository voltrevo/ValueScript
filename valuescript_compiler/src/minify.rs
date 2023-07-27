use std::{
  iter::{Peekable, Skip, Take},
  str::Chars,
};

use swc_common::BytePos;

pub fn minify(source: &str, span: swc_common::Span) -> String {
  let mut minifier = Minifier::new(source, span);
  minifier.run();

  minifier.res
}

struct Minifier<'a> {
  chars: Peekable<Take<Skip<Chars<'a>>>>,
  res: String,
}

impl<'a> Minifier<'a> {
  fn new(source: &'a str, span: swc_common::Span) -> Self {
    let BytePos(start) = span.lo;
    let BytePos(end) = span.hi;

    let chars = source
      .chars()
      .skip(start as usize)
      .take((end - start) as usize)
      .peekable();

    Minifier {
      chars,
      res: String::new(),
    }
  }

  fn run(&mut self) {
    let mut punctuation_last = true;

    while let Some(c) = self.chars.peek().cloned() {
      if c.is_ascii_whitespace() {
        self.skip_ws(punctuation_last);
        continue;
      }

      self.res.push(c);
      self.chars.next();

      punctuation_last = is_js_punctuation(c);

      match c {
        '\'' | '"' => self.simple_string(c),
        '`' => self.template_string(),
        _ => {}
      }
    }
  }

  fn skip_ws(&mut self, punctuation_last: bool) {
    while let Some(c) = self.chars.peek().cloned() {
      if !c.is_ascii_whitespace() {
        if !punctuation_last && !is_js_punctuation(c) {
          self.res.push(' ');
        }

        break;
      }

      self.chars.next();
    }
  }

  fn simple_string(&mut self, quote_c: char) {
    let mut escaping = false;

    for c in &mut self.chars {
      self.res.push(c);

      if !escaping && c == quote_c {
        break;
      }

      escaping = c == '\\';
    }
  }

  fn template_string(&mut self) {
    todo!()
  }
}

fn is_js_punctuation(c: char) -> bool {
  c.is_ascii_punctuation() && !matches!(c, '$' | '_')
}
