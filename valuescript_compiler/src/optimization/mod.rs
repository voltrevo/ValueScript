mod collapse_pointers_of_pointers;
mod extract_constants;
mod kal;
mod optimize;
mod remove_meta_lines;
mod remove_noops;
mod shake_tree;
mod simplify;
pub mod try_to_kal;
pub mod try_to_val;

pub use optimize::optimize;
