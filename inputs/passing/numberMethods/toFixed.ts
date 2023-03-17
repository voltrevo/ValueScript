// test_output! [["0.00","1.23","123.00","0.01","0.00123","12345.68","3.14","-1.00"],["NaN"]]

export default function () {
  const positive = [
    (0).toFixed(2),
    (1.2345).toFixed(2),
    (123).toFixed(2),
    (0.005678).toFixed(2),
    (0.00123456).toFixed(5),
    (12345.6789).toFixed(2),
    (3.14159).toFixed(2),
    (-1).toFixed(2),
  ];

  const negative = [
    (0 / 0).toFixed(2),
    // (1 / 0).toFixed(2), TODO: Fix "inf", should be "Infinity"
  ];

  return [positive, negative];
}
