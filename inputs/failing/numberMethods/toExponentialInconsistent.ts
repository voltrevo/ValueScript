//! test_output([["1.20e+2","1.22e+2","1.22e+2","1.24e+2","1.24e+2"],["1.21e+0","1.22e+0","1.23e+0","1.24e+0","1.25e+0"]])
// This is wrong. There's some inconsistent banker's rounding going on.
// It appears to be inherent to rust's {:.*e} formatting - a technical oversight?
// Or maybe it has to do with parsing the source string into a number imprecisely?

export default function () {
  return [
    [ // Banker's rounding?
      (120.5).toExponential(2),
      (121.5).toExponential(2), // 1.22e+2
      (122.5).toExponential(2), // 1.22e+2 *again*
      (123.5).toExponential(2), // 1.24e+2
      (124.5).toExponential(2), // 1.24e+2 *again*
    ],
    [ // Not banker's rounding??
      (1.205).toExponential(2),
      (1.215).toExponential(2), // 1.22e+0
      (1.225).toExponential(2), // 1.23e+0 (*not* repeated)
      (1.235).toExponential(2), // 1.24e+0
      (1.245).toExponential(2), // 1.25e+0 (*not* repeated)
    ],
  ];
}
