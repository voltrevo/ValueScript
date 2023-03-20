// test_output! [[true,true,true,true,true,true,true],[false,false,false,false,false]]

export default function () {
  const integers = [
    Number.isInteger(0),
    Number.isInteger(1),
    Number.isInteger(-1),
    Number.isInteger(10000000000000),
    Number.isInteger(Number.MAX_SAFE_INTEGER),
    Number.isInteger(Number.MIN_SAFE_INTEGER),
    Number.isInteger(Number.MAX_VALUE),
  ];

  const notIntegers = [
    Number.isInteger(0.1),
    Number.isInteger(-0.1),
    Number.isInteger(NaN),
    Number.isInteger(Infinity),
    Number.isInteger(-Infinity),
  ];

  return [integers, notIntegers];
}
