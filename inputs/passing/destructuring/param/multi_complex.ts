// test_output! [55,145,237,43,31]

export default function main() {
  return [
    foo(1, 2, { c: 3, d: 4 }, [5, 6], { g: 7, h: 8 }, [9, 10]),
    foo(1, 2, { c: 3, d: 4 }, [5, 6], { g: 7, h: 8 }, [9]),
    foo(1, 2, { c: 3, d: 4 }, [5, 6], { g: 7 }, [9]),
    foo(1, 2, { c: 3, d: 4 }, [5, 6], { g: 7, h: 8 }),
    foo(1, 2, { c: 3, d: 4 }, [5, 6]),
  ];
}

function foo(
  a: number,
  b: number,
  { c, d }: { c: number, d: number },
  [e, f]: [number, number],
  { g, h = 100 }: { g: number, h?: number } = { g: 1, h: 2 },
  [i, j = 100]: [number, number?] = [3, 4]
) {
  return a + b + c + d + e + f + g + h + i + j;
}
