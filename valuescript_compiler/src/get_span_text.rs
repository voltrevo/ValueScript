use valuescript_common::unicode_at;

pub fn get_span_text(span: swc_common::Span, source: &str) -> String {
  let mut res = String::new();

  for i in span.lo().0..span.hi().0 {
    if let Some(c) = unicode_at(source.as_bytes(), source.len(), i as usize) {
      res.push(c);
    }
  }

  res
}
