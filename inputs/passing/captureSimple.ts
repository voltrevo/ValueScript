//! test_output(37)

export default function main() {
  return constant(37)();
}

function constant<T>(value: T) {
  return () => value;
}
