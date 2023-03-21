// test_output! E: "TypeError: Cannot mix BigInt and other types"

export default function () {
  return 1 + (1n as unknown as number);
}
