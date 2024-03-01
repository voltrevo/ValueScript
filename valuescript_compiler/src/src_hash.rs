use tiny_keccak::{Hasher, Keccak};

use crate::{asm::Hash, get_span_text::get_span_text};

pub fn src_hash(source: &str, span: swc_common::Span) -> Hash {
  let mut k = Keccak::v256();
  k.update(get_span_text(span, source).as_bytes());

  let mut output = [0u8; 32];
  k.finalize(&mut output);

  Hash(output)
}
