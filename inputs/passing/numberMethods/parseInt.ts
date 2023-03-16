// test_output! [[37,-37,15,37,37,31],[NaN,NaN]]

export default function () {
  const positive_and_negative = [
    Number.parseInt("37"),
    Number.parseInt("-37"),
    Number.parseInt("F", 16),
    Number.parseInt("37.1"),
    Number.parseInt("  37 "),
    Number.parseInt("1F", 16),
  ];

  const nan_values = [
    Number.parseInt("hello"),
    Number.parseInt("10", 40),
  ];

  return [positive_and_negative, nan_values];
}
