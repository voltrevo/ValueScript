#[derive(Default)]
pub struct VirtualMachine {
  bytecode: Vec<u8>,
}

impl VirtualMachine {
  pub fn load(&mut self, bytecode: Vec<u8>) {
    self.bytecode = bytecode;
  }

  pub fn run(&mut self) {
    std::panic!("Not implemented");
  }

  pub fn new() -> Self {
    return Default::default();
  }

  pub fn print(&self) {
    std::panic!("Not implemented");
  }
}
