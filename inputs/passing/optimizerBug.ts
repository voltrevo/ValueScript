//! test_output(1)

export default function main() {
  return f([1]);
}

export function f(values: number[]): number {
  const x = [
    values[0],
  ];

  if (values[0]) {
    return x[0];
  }

  return 0;
}
