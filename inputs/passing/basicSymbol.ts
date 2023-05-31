//! test_output([Symbol.iterator,{},3])

export default function main() {
  const x = { [Symbol.iterator]: 3 };

  return [Symbol.iterator, x, x[Symbol.iterator]];
}
