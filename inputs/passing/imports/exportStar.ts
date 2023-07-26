//! test_output(["a","b (local)","this is the foo function","this is the bar function","baz",42])

import * as lotsOfThings from "./stuff/lotsOfThings.ts";

export default () => {
  return [
    lotsOfThings.a(),
    lotsOfThings.b(),
    lotsOfThings.foo(),
    lotsOfThings.bar(),
    lotsOfThings.baz(),
    lotsOfThings.x,
  ];
};
