// //! test_output("Hi")

export default () => {
  return (new Foo())["a method"]();
};

class Foo {
  "a method"() {
    return "Hi";
  }
}
