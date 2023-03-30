export default function main() {
  let result = foo; // Error: TDZ
  let foo = "oops";
  return result;
}
