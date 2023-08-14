use swc_common::BytePos;
use tiny_keccak::{Hasher, Keccak};

use crate::asm::Value;

pub fn source_hash(source: &str, span: swc_common::Span) -> [u8; 32] {
  let BytePos(start) = span.lo;
  let BytePos(end) = span.hi;

  let chars = source
    .chars()
    .skip(start as usize)
    .take((end - start) as usize);

  let mut k = Keccak::v256();
  k.update(chars.collect::<String>().as_bytes());

  let mut output = [0u8; 32];
  k.finalize(&mut output);

  output
}

pub fn source_hash_asm(source: &str, span: swc_common::Span) -> Value {
  let mut result = String::with_capacity(66);
  result.push_str("0x");

  for byte in &source_hash(source, span) {
    result.push_str(&format!("{:02x}", byte));
  }

  Value::String(result)
}
