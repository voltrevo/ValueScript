//! test_output({"iter":[4,5,6,7,8,9],"iterSnapshot":[2,3,4,5,6,7,8,9]})

export default function main() {
  let iter = range(10);
  iter.next();
  iter.next();

  let iterSnapshot = iter;

  iter.next();
  iter.next();

  return {
    iter: [...iter],
    iterSnapshot: [...iterSnapshot],
  };
}

function* range(n: number) {
  for (let i = 0; i < n; i++) {
    yield i;
  }
}
