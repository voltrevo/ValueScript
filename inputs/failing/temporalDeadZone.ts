export default function main() {
  let result = foo;
  let foo = "oops";
  return result; // should throw
}
