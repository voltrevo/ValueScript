// //! test_output({})

export default function main() {
  let x: Record<string, unknown> = { y: 3 };
  delete x.y;

  return x;
}
