#[macro_export]
macro_rules! format_err {
  ($fmt:expr $(, $($arg:expr),*)?) => {{
      let formatted_string = format!($fmt $(, $($arg),*)?);

      // TODO: This should be a proper error type
      Err(Val::String(Rc::new(formatted_string)))
  }};
}

#[macro_export]
macro_rules! format_val {
  ($fmt:expr $(, $($arg:expr),*)?) => {{
      let formatted_string = format!($fmt $(, $($arg),*)?);
      Val::String(Rc::new(formatted_string))
  }};
}
