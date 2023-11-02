use std::error::Error;

pub type GenericError = Box<dyn Error>;

pub fn error_str(s: &str) -> GenericError {
  s.to_string().into()
}
