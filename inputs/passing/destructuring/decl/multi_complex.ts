//! test_output([55,145,237,43,31])

export default function main() {
  let res = [];

  {
    const [
      a,
      b,
      { c, d },
      [e, f],
      { g, h = 100 } = { g: 1, h: 2 },
      [i, j = 100] = [3, 4],
    ] = [1, 2, { c: 3, d: 4 }, [5, 6], { g: 7, h: 8 }, [9, 10]];

    res.push(a + b + c + d + e + f + g + h + i + j);
  }

  {
    const [
      a,
      b,
      { c, d },
      [e, f],
      { g, h = 100 } = { g: 1, h: 2 },
      [i, j = 100] = [3, 4],
    ] = [1, 2, { c: 3, d: 4 }, [5, 6], { g: 7, h: 8 }, [9]];

    res.push(a + b + c + d + e + f + g + h + i + j);
  }

  {
    const [
      a,
      b,
      { c, d },
      [e, f],
      { g, h = 100 } = { g: 1, h: 2 },
      [i, j = 100] = [3, 4],
    ] = [1, 2, { c: 3, d: 4 }, [5, 6], { g: 7 }, [9]];

    res.push(a + b + c + d + e + f + g + h + i + j);
  }

  {
    const [
      a,
      b,
      { c, d },
      [e, f],
      { g, h = 100 } = { g: 1, h: 2 },
      [i, j = 100] = [3, 4],
    ] = [1, 2, { c: 3, d: 4 }, [5, 6], { g: 7, h: 8 }];

    res.push(a + b + c + d + e + f + g + h + i + j);
  }

  {
    const [
      a,
      b,
      { c, d },
      [e, f],
      { g, h = 100 } = { g: 1, h: 2 },
      [i, j = 100] = [3, 4],
    ] = [1, 2, { c: 3, d: 4 }, [5, 6]];

    res.push(a + b + c + d + e + f + g + h + i + j);
  }

  return res;
}
