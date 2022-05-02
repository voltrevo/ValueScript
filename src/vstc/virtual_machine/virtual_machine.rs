use super::vs_number::VsNumber;
use super::vs_string::VsString;
use super::operations::op_plus;

#[derive(Default)]
pub struct VirtualMachine {
  bytecode: Vec<u8>,
}

impl VirtualMachine {
  pub fn load(&mut self, bytecode: Vec<u8>) {
    self.bytecode = bytecode;
  }

  pub fn run(&mut self) {
    let a = VsNumber::from_f64(1_f64);
    let b = VsString::from_str("2");

    std::println!("a + b = {}", op_plus(&a, &b));

    std::panic!("Not implemented");
  }

  pub fn new() -> Self {
    return Default::default();
  }

  pub fn print(&self) {
    std::panic!("Not implemented");
  }
}
