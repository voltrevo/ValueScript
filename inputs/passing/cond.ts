//! test_output([{"a":1,"b":0,"x":0},{"a":1,"b":1,"x":0},{"a":2,"b":1,"x":1},{"a":2,"b":2,"x":2}])

export default function () {
  let a = 0;
  let b = 0;
  let x = 0;

  let logs = [];

  x += makeTrue() ? a++ : b++;
  logs.push({ a, b, x });

  x += makeFalse() ? a++ : b++;
  logs.push({ a, b, x });

  x += makeTrue() ? a++ : b++;
  logs.push({ a, b, x });

  x += makeFalse() ? a++ : b++;
  logs.push({ a, b, x });

  return logs;
}

function makeTrue() {
  return true;
}

function makeFalse() {
  return false;
}
