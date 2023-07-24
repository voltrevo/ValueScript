//! test_output(105)

export default () => {
  return [
    new stuff[0]().x,
    stuff[1](),
    stuff[2](),
  ].reduce((a, b) => a * b);
};

const stuff = [
  class Foo { x = 3; },
  function bar() { return 5 },
  () => 7,
] as const;
