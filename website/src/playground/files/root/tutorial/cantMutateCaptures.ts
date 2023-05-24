// ValueScript is like TypeScript without side effects. We achieve this by
// deviating from JavaScript in three key ways:
//
// 2. Captured variables cannot be mutated

export default function main() {
  let counter = 0;

  const next = () => {
    counter++;
    return counter;
  };

  return [next(), next(), next()];
  // JavaScript:  [1, 2, 3]
  // ValueScript: Compilation error
}

// Both JavaScript and ValueScript allow:
// - Mutating variables
// - Capturing variables
//
// But in ValueScript you can only do one or the other.
//
// By allowing both, JavaScript allows the `next` function to return different
// values each time it is called. This is a side effect.
//
// However, you can get this kind of behavior in ValueScript by using a class...
