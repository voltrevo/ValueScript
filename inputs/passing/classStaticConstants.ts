//! test_output(-246)

export default () => {
  return Foo.Nested.double(Foo.bar);
};

class Foo {
  static bar = -123;
  static Nested = class {
    static double = (x: number) => x * 2;
  };
}
