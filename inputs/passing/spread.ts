// test_output! [".abc.",".abc.",".abc.",".abc."]

export default function () {
  const middle = ["a", "b", "c"] as const;
  const arr = [".", ...middle, "."];
  const call = foo(".", ...middle, ".");

  let bar = new Bar(".", ...middle, ".");
  bar.method(".", ...middle, ".");

  const new_ = bar.value;
  const method = bar.method_value;

  return [arr, call, new_, method].map((x) => x.join(""));
}

function foo(a: string, b: string, c: string, d: string, e: string) {
  return [a, b, c, d, e];
}

class Bar {
  value: string[];
  method_value: string[] = [];

  constructor(a: string, b: string, c: string, d: string, e: string) {
    this.value = [a, b, c, d, e];
  }

  method(a: string, b: string, c: string, d: string, e: string) {
    this.method_value = [a, b, c, d, e];
  }
}
