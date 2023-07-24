//! test_output("foo")

export default () => {
  const stuff = {
    foo() {
      return "foo";
    },
  };

  return stuff.foo();
};
