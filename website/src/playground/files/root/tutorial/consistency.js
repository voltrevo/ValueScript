// ValueScript is not about what we think JavaScript should be. We only deviate
// from JavaScript on things related to value semantics and in a few other
// carefully considered cases.
//
// (It also deviates because of things that aren't implemented yet.)
//
// Otherwise, ValueScript behaves as JavaScript does. Sometimes that includes
// some rather unexpected things:

export default function () {
  return [
    [] + [],                // ""
    [10, 1, 3].sort(),      // [1, 10, 3]
    "b" + "a" + +"a" + "a", // "baNaNa"
  ];
  //                           JavaScript and ValueScript agree on these.
}

// We're not sure yet where exactly to draw this line. Another notable example
// is `with` syntax, which isn't included. If you have an opinion about what we
// should or shouldn't deviate on, let us know at:
//   https://github.com/voltrevo/ValueScript/issues/new