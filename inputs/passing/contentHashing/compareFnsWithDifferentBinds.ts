//! test_output(false)

export default function () {
  return make("foo") === make("bar");
}

function make(bind: unknown) {
  return function test() {
    return bind;
  };
}
