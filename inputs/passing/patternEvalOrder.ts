//! test_output("right")

export default function main() {
  let x;
  let key = 'left';

  ({[key]: x} = (key = 'right') && {left:'left', right:'right'} as any)

  return x;
}
