//! test_output(true)

export default function () {
  return functions[0]() === functions[1]();
}

const functions = [
  function foo() {
    return function test() {
      return content();
    };
  },
  function foo() {
    return function test() {
      return content();
    };
  },
];

function content() {
  return "foo";
}
