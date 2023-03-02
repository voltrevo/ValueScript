// test_output! {"right":"right"}
// (This is wrong.)

export default function main() {
  const x = {} as any;
  let key = 'left';

  x[key] = (key = 'right');

  return x;
}
