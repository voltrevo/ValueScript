// test_output! [1,2,3,1,2,"H","i"]

export default function main() {
  return [...gen()];
}

function* gen() {
  yield* [1, 2, 3];
  yield* gen12();
  yield* "Hi";
}

function* gen12() {
  yield 1;
  yield 2;
}
