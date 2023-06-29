use crate::asm::Module;
use crate::name_allocator::NameAllocator;

use super::collapse_pointers_of_pointers::collapse_pointers_of_pointers;
use super::extract_constants::extract_constants;
use super::shake_tree::shake_tree;

pub fn optimize(module: &mut Module, pointer_allocator: &mut NameAllocator) {
  collapse_pointers_of_pointers(module);
  extract_constants(module, pointer_allocator);
  shake_tree(module);
}
