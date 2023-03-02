// test_output! ["c","a","b"]

export default function main() {
  let log: string[] = [];
  let x;

  ({[log.push('a')]: x = log.push('b')} = [log.push('c')] && {} as any);

  return log;
}
