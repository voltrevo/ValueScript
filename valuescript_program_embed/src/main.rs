use std::rc::Rc;

use valuescript_vm::VirtualMachine;

pub fn main() {
  let mut vm = VirtualMachine::new();
  let result = vm.run(
    &Rc::new(vec![
      //
      // This is the compiled bytecode for inputs/passing/projEuler/p28.ts.
      //
      // Using `RUSTFLAGS="-C opt-level=s" cargo build --release` it currently compiles to 534KiB.
      // A program with just println!("Test") is 315KiB, so we might be able to get down to around
      // 219KiB by simplifying the way we print the result.
      //
      // Since we're still in early development, the bytecode is subject to change, which means this
      // bytecode might break.
      //
      // If you need to fix it, or use a different program, use vstc:
      //     vstc compile program.ts
      //     vstc assemble out.vsm
      //     # Output is in out.vsb. Use the xxd program (or otherwise) to see the bytes.
      //
      // Another option is to checkout the commit from when this was originally written:
      //     git checkout 4e77747ae67e0ef27f9841111058599c8e916a2f
      //     cargo build
      //     ./target/debug/valuescript_program_embed
      //
      0x0d, 0x05, 0x00, 0x0a, 0x00, 0x0b, 0x08, 0x00, 0x21, 0x0d, 0x9c, 0x00, 0x09, 0x09, 0x06,
      0x01, 0x06, 0x09, 0x06, 0x19, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x7f, 0x40,
      0x00, 0x02, 0x05, 0x0e, 0x02, 0x06, 0x01, 0x03, 0x21, 0x0d, 0x9c, 0x00, 0x09, 0x09, 0x06,
      0x01, 0x06, 0x03, 0x06, 0x0d, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x7f, 0x40,
      0x00, 0x02, 0x05, 0x0e, 0x02, 0x06, 0x01, 0x04, 0x21, 0x0d, 0x9c, 0x00, 0x09, 0x09, 0x06,
      0x01, 0x06, 0x05, 0x06, 0x11, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x7f, 0x40,
      0x00, 0x02, 0x05, 0x0e, 0x02, 0x06, 0x01, 0x05, 0x21, 0x0d, 0x9c, 0x00, 0x09, 0x09, 0x06,
      0x01, 0x06, 0x07, 0x06, 0x15, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x7f, 0x40,
      0x00, 0x02, 0x05, 0x0e, 0x02, 0x06, 0x01, 0x06, 0x26, 0x09, 0x06, 0x01, 0x0e, 0x03, 0x0e,
      0x04, 0x0e, 0x05, 0x0e, 0x06, 0x00, 0x08, 0x06, 0x72, 0x65, 0x64, 0x75, 0x63, 0x65, 0x09,
      0x0d, 0x1e, 0x01, 0x00, 0x00, 0x00, 0x0b, 0x08, 0x02, 0x26, 0x0e, 0x02, 0x08, 0x06, 0x72,
      0x65, 0x64, 0x75, 0x63, 0x65, 0x09, 0x0d, 0x1e, 0x01, 0x00, 0x04, 0x01, 0x06, 0x03, 0x05,
      0x11, 0x0e, 0x05, 0x0e, 0x03, 0x06, 0x10, 0x0e, 0x06, 0x06, 0x28, 0x0e, 0x06, 0xe1, 0x00,
      0x21, 0x0d, 0xe6, 0x00, 0x09, 0x0e, 0x02, 0x00, 0x02, 0x24, 0x0e, 0x02, 0x06, 0x02, 0x06,
      0x04, 0x0e, 0x04, 0x0e, 0x06, 0x04, 0x01, 0x0e, 0x05, 0x06, 0x02, 0x05, 0x27, 0xb4, 0x00,
      0x01, 0x0e, 0x04, 0x00, 0x00, 0x0b, 0x09, 0x01, 0x24, 0x0e, 0x02, 0x06, 0x00, 0x03, 0x24,
      0x0e, 0x02, 0x06, 0x01, 0x04, 0x24, 0x0e, 0x02, 0x06, 0x02, 0x05, 0x06, 0x06, 0x03, 0x0e,
      0x05, 0x02, 0x06, 0x06, 0x03, 0x0e, 0x04, 0x06, 0x05, 0x0e, 0x02, 0x0e, 0x06, 0x07, 0x04,
      0x0e, 0x07, 0x0e, 0x03, 0x06, 0x01, 0x09, 0x0e, 0x04, 0x0e, 0x05, 0x0e, 0x06, 0x00, 0x00,
      0x00, 0x0b, 0x05, 0x02, 0x04, 0x0e, 0x02, 0x0e, 0x03, 0x00, 0x00,
    ]),
    &[],
  );

  println!("{}", result);
}
