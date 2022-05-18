export default function() {
  let result = foo;
  let foo = 'oops';
  return result; // should throw
}
