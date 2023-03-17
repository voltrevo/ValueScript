// test_output! [["0e+0","1.2e+0","1.2e+1","1.2e+2","1.2e-1","1.2e-2","1.2e-3","1.2e-4"],["0.00e+0","1.23e+0","1.23e+1","1.23e-1","1.23e-2","1.23e-3","1.23e-4"],["NaN"]]

export default function () {
  const withoutPrecision = [
    (0).toExponential(),
    (1.2).toExponential(),
    (12).toExponential(),
    (120).toExponential(),
    (0.12).toExponential(),
    (0.012).toExponential(),
    (0.0012).toExponential(),
    (0.00012).toExponential(),
  ];

  const withPrecision = [
    (0).toExponential(2),
    (1.2345).toExponential(2),
    (12.345).toExponential(2),
    // (120.5).toExponential(2), incorrectly rounds to 1.20e+2
    // in JS: 1.21e+2
    // see inputs/failing/numberMethods/toExponentialInconsistent.ts
    (0.12345).toExponential(2),
    (0.012345).toExponential(2),
    (0.0012345).toExponential(2),
    (0.00012345).toExponential(2),
  ];

  const negative = [
    (0 / 0).toExponential(),
    // (1 / 0).toExponential(), TODO: Fix "inf", should be "Infinity"
  ];

  return [withoutPrecision, withPrecision, negative];
}
