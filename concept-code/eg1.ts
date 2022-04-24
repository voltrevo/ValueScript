import { inc } from 'value-script';

export default function main() {
  function f1(a: number, b: number, c: number) {
    return a + b * c;
  }

  function f2() {
    inc.call(this);
  }

  let x = f1(1, 2, 3);
  f2.call(x);
  
  return x;
}