//! test_output(true)
// Should be: false.

export default function () {
  // The test functions returned have exactly the same source code, but they should not be equal
  // because they reference `content` functions that are not equal.
  return foo() === bar();
}

function foo() {
  function content() {
    return "foo";
  }

  return function test() {
    return content();
  };
}

function bar() {
  function content() {
    return "bar";
  }

  return function test() {
    return content();
  };
}
