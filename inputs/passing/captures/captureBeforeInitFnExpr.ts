export default function main() {
  const foo = () => x; // Error
  const x = 3;

  return foo();
}
