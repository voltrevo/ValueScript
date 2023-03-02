// test_output! ["c","a","b"]

export default function main() {
  let log = [];
  let x = {} as any;

  [
    x[log.push('a')] = log.push('b')
  ] = [
    log.push('c') && undefined
  ];

  return log;
}
