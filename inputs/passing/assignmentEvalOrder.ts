//! test_output({"left":"right"})

export default function main() {
  let x = {} as any;
  let key = "left";

  x[key] = key = "right";

  return x;
}
