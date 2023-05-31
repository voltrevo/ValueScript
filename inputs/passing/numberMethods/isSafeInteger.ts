//! test_output([[true,true,true,true,true,true],[false,false,false,false,false,false]])

export default function () {
  const minSafe = -9007199254740991;
  const maxSafe = 9007199254740991;

  const safe_integers = [
    Number.isSafeInteger(0),
    Number.isSafeInteger(1),
    Number.isSafeInteger(-1),
    Number.isSafeInteger(maxSafe),
    Number.isSafeInteger(minSafe),
    Number.isSafeInteger(10000000000000),
  ];

  const not_safe_integers = [
    Number.isSafeInteger(maxSafe + 1),
    Number.isSafeInteger(minSafe - 1),
    Number.isSafeInteger(0.1),
    Number.isSafeInteger(NaN),
    Number.isSafeInteger(Infinity),
    Number.isSafeInteger(-Infinity),
  ];

  return [safe_integers, not_safe_integers];
}
