// //! test_output("foo")

export default () => {
  class X {
    foo = "foo";
  }

  return new X().foo;
};

// deno-lint-ignore no-unused-vars
class X {}
