// test_output! [[4,2,4,6],[-1,-1]]

export default function () {
  const positive = [
    "  xxx".lastIndexOf("x"),
    "xxx  ".lastIndexOf("x"),
    "  xxx  ".lastIndexOf("x"),
    "  xyxyxy  ".lastIndexOf("xy"),
  ];

  const negative = [
    "  xxx  ".lastIndexOf("x", 5),
    "abc".lastIndexOf("abcd"),
  ];

  return [positive, negative];
}
