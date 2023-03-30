// test_output! ["a","b"]

export default function main() {
  let log = [];
  let x = {} as any;

  x[log.push("a")] = log.push("b");

  return log;
}
