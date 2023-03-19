// test_output! [true,false,false,true,true,true,true,false,true,false,false,false]

export default function () {
  return [
    true,
    false,
    "",
    "0",
    "1",
    {},
    [],
    0,
    1,
    null,
    undefined,
    0 / 0,
  ].map(Boolean);
}
