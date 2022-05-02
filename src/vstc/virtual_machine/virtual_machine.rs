use super::vs_value;

#[derive(Default)]
pub struct VirtualMachine {
  bytecode: Vec<u8>,
}

impl VirtualMachine {
  pub fn load(&mut self, bytecode: Vec<u8>) {
    self.bytecode = bytecode;
  }

  pub fn run(&mut self) {
    let a = vs_value::VsNumber::from_f64(1_f64);
    let b = vs_value::VsString::from_str("2");

    std::println!("a + b = {}", vs_value::add(&a, &b));

    std::panic!("Not implemented");
  }

  pub fn new() -> Self {
    return Default::default();
  }

  pub fn print(&self) {
    std::panic!("Not implemented");
  }
}
