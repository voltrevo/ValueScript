//! test_output([true,false,true])

export default () => {
  return [
    new X() instanceof X,
    new X() instanceof Y,
    new Error("") instanceof Error,
  ];
};

class X {
  x() {}
}

class Y {
  y() {}
}
