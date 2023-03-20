// test_output! [true,false,false,[1,2,3],["a","b","c"],[],[1,2,3],[],[,,],[3,1],[true]]

export default function () {
  return [
    Array.isArray([]),
    Array.isArray({}),
    Array.isArray(1),
    Array.from([1, 2, 3]),
    Array.from({ length: 3, 0: "a", 1: "b", 2: "c" }),
    Array.from(true as any),
    Array.of(1, 2, 3),
    Array.of(),
    Array(3),
    Array(3, 1),
    Array(true),
  ];
}
