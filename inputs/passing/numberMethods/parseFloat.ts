// test_output! [[37,-37,37.1,37,37.1,1.5,0.1,1.23],[NaN]]

export default function () {
  const numbers = [
    Number.parseFloat("37"),
    Number.parseFloat("-37"),
    Number.parseFloat("37.1"),
    Number.parseFloat("  37 "),
    Number.parseFloat("37.1"),
    Number.parseFloat("1.5"),
    Number.parseFloat("0.1"),
    Number.parseFloat("1.23"),
  ];

  const nan_values = [
    Number.parseFloat("hello"),
  ];

  return [numbers, nan_values];
}
