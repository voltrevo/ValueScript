use std::{error::Error, process::exit, rc::Rc};

use storage::{storage_head_ptr, SledBackend, Storage};
use valuescript_compiler::asm;
use valuescript_vm::{
  vs_value::{ToVal, Val},
  DecoderMaker, VirtualMachine,
};

use crate::to_bytecode::{format_from_path, to_bytecode};

pub fn create_db(
  storage: &mut Storage<SledBackend>,
  class_path: &str,
  args: &[Val],
) -> Result<(), Box<dyn Error>> {
  let class = Rc::new(to_bytecode(format_from_path(class_path), class_path))
    .decoder(0)
    .decode_val(&mut vec![]);

  // TODO: Use compile_str instead. Need to implement rest params: `new Class(...args)`.
  let create = asm::inline(
    "export @create {}

    @create = function (%class, %args) {
      new %class %args %return
    }",
  );

  let mut vm = VirtualMachine::default();

  let instance = match vm.run(
    None,
    &mut Val::Undefined,
    create,
    vec![class, args.to_vec().to_val()],
  ) {
    Ok(instance) => instance,
    Err(err) => {
      println!("Uncaught exception: {}", err.pretty());
      exit(1);
    }
  };

  storage.set_head(storage_head_ptr(b"state"), &instance)
}
