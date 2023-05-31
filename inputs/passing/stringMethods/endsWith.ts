//! test_output([true,false,true,true,true,true,false,true,true,false])

export default function () {
  return [
    "hello".endsWith("lo"),
    "hello".endsWith("l"),
    "hello".endsWith("o"),
    "hello".endsWith(""),
    "hello".endsWith("hello"),
    "Cats are the best!".endsWith("best!"),
    "Cats are the best!".endsWith("best"),
    "Cats are the best!".endsWith("best", 17),
    "Is this a question?".endsWith("?"),
    "Is this a question?".endsWith("question"),
  ];
}
