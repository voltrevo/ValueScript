# ValueScript

## About

ValueScript is a dialect of TypeScript with value semantics.

The syntax is the same, in fact we use [SWC](https://swc.rs/)'s TypeScript
parser, but the engine that evaluates the code is different.

The big idea is that variables change, but data does not. For example:

```ts
export default function main() {
  const leftBowl = ["apple", "mango"];

  let rightBowl = leftBowl;
  rightBowl.push("peach");

  return { leftBowl, rightBowl };
}
```

In TypeScript, `main()` produces:

```js
{
  leftBowl: ['apple', 'mango', 'peach'],
  rightBowl: ['apple', 'mango', 'peach'],
}
```

This is because TypeScript interprets the code to mean that `leftBowl` and
`rightBowl` are the _same_ object, and that object _changes_.

In ValueScript, objects do not change, but variables do. Pushing onto
`rightBowl` is interpreted as a change to the `rightBowl` variable itself, not
the data it points to. `rightBowl` points to some new data, which may reference
the old data, but only as a performance optimization.

This is how ValueScript achieves the output below for the same input program:

```js
{
  leftBowl: ['apple', 'mango'],
  rightBowl: ['apple', 'mango', 'peach'],
}
```

To try this yourself, run the following:

```sh
cargo build --bin vstc
export PATH="$PATH:$(pwd)/target/debug"
vstc run inputs/passing/readme-demo.ts
```

## Intended Usage

ValueScript has its roots in a programming school of thought that discourages
the use of mutation.

While this idea is indeed useful, the result of this approach can only push the
mutation to the edges of the program. At the edge, you need to interact with
things that are inherently mutable, like users. The code you write becomes a
subsystem of some larger framework that dictates the interface with the outside
world.

This is why we expect that ValueScript will be most useful as a tool within a
TypeScript project, rather than an alternative to it. This way you can benefit
from TypeScript's rich ecosystem to interact with users and external systems,
and also have a clearly separated immutable subsystem to define the core of your
application.

Because ValueScript shares the same syntax as TypeScript, you'll be able to
inline ValueScript code into TypeScript like this:

```ts
const points = runValueScript(() => {
  const x = [3, 5];
  const y = x;
  y[0]--;

  return { x, y };
});

// ValueScript doesn't have console.log, but TypeScript does.
console.log(points); // { x: [3, 5], y: [2, 5] }
```

Additionally, ValueScript has benefits that make it a suitable target for
secondary storage. Rather than writing to a file, your ValueScript code can read
and update persistent objects the same way it interacts with regular in-memory
objects.

## Benefits

<details>
<summary>Eliminate mutation bugs</summary>

Mutating things across your program is frequently intended, but it's also
frequently unintended, causing bugs.

This is why you are usually encouraged not to mutate function arguments, among
other things. Sometimes you'll see workaround like `const a = [...b];`. In
ValueScript, just write it the natural way.

</details>

<details>
<summary>`const` means what you think it does</summary>

Ever felt weird about using `const` in situations like this?

```ts
const values = [];

values.push(123);

return items;
```

Us too. The reason is that, in a mutable world, it's the array that `values`
points to that is mutating. Pushing to that array doesn't change `values` - it
still points to the same array, right?

In ValueScript, it's not the same array, because arrays don't change. Instead,
it is indeed the variable that changes, and therefore, if you mark it as
`const`, attempting to do so is a compile-time error.

</details>

<details>
<summary>Testable code</summary>

Testing code is all about being able to draw a boundary around something that
can be given inputs so that you can check its outputs against your expectations.

Being able to draw these boundaries is usually challenging in real-world
systems, because by default everything wants to connect to something tangible to
serve its purpose as directly as possible. Most things that matter to you become
untested because of their coupling to externalities that are too difficult to
meaningfully replicate in a test case. Testing degrades into an inauthentic
add-on that focuses on trivialities.

By using ValueScript, you can maintain a clear separation between a domain that
should be easy to test - the core of what your application does, and a domain
that is difficult to test - how your application talks to the world.

A ValueScript program is always a function that, when called with the same
inputs, produces the same outputs.

</details>

<details>
<summary>No garbage collection</summary>

In ValueScript, it's impossible to create data that circularly references
itself. This isn't because something is keeping watch and producing an error if
you do it accidentally. Rather, it's just an inherent consequence of how
ValueScript works:

```ts
let x = {};
x.x = x; // { x: {} }

// (In TypeScript: { x: { x: { x: { x: { ... } } }} })
```

Circular references are the whole reason why garbage collectors are needed
(assuming you want to reuse memory and don't want to figure out when it's safe
to do so). Without them, ValueScript is able to simply keep a count of how many
references each object has, and when that count drops to zero, it cleans up the
memory immediately.

</details>

<details>
<summary>Persistence</summary>

In a traditional mutable program, the important entities in that program often
can't be stored authentically without also capturing the state of the entire
program that contains them. Even when that isn't true, the entity needs to be
translated into a form that can be stored in a process we know and love called
_serialization_.

ValueScript is different. Everything can be persisted as its direct contents and
a recursive inclusion of its dependencies. This includes functions and class
instances (and the methods on those class instances). In ValueScript, everything
is plain data.

In fact, because ValueScript doesn't require garbage collection, it's also
possible to build up large structures that wouldn't fit into memory. Garbage
collection is a limiting factor on traditional languages on this point, because
you need to periodically fully traverse the memory to find things that can be
cleaned up.

</details>

<details>
<summary>Make use of TypeScript's type checking</summary>

ValueScript is similar enough to TypeScript that the type checker correctly
identifies type errors in ValueScript.

In fact, when the differences matter, the type checker often actually favors
ValueScript, not TypeScript.

E.g.

```ts
let a: { value?: string | number } = {};
a.value = "str";

let b = a;
b.value = 37;

type T = typeof a.value;
//               ^^^^^^^
//               TypeScript: 37
//               ValueScript: 'str'

// The type checker assigns `string` to `T`.
```

</details>

<details>
<summary>Concurrency</summary>

By using value semantics, ValueScript ensures that a function, called with the
same arguments, always returns the same value. This includes instance methods by
considering the instance data to be one of the arguments.

This means that if you wrap some calculation in a function that takes no
arguments, it is destined to return the same value, regardless of what happens
elsewhere in the program:

```ts
const f = () => {
  const x = widget.calculate(37);
  const y = expensiveCalculation(3, 5);

  return x + y;
};
```

Above, `widget` is captured by `f`. ValueScript requires that captured variables
are `const`, which means that `widget` cannot change, and therefore
`widget.calculate(37)` cannot change. This means that the value of `f()` is
independent of any other work that happen in our program.

Therefore, we could safely evaluate `f()` concurrently. In future, some
calculations might automatically be upgraded to concurrent execution, but
knowing when it is worthwhile to create a separate thread of execution is a
complex and inexact science.

Instead, in the foreseeable future, ValueScript will have a primitive called
`vs.thread`:

```ts
const f = vs.thread(() => {
  const x = widget.calculate(37);
  const y = expensiveCalculation(3, 5);

  return x + y;
});
```

On the surface, `vs.thread` simply returns the function that is provided to it,
unaltered. However, this signals the runtime to evaluate the function on another
thread.

</details>

<details>
<summary>Static Analysis & Optimization</summary>

ValueScript dramatically expands the cases where program behavior can be
determined statically. In traditional languages, inferences about data in
variables are quickly lost because it is impossible to know whether some other
code might modify that data.

A relatively simple application of this is tree-shaking. ValueScript analyzers
will be able to determine much more accurately what code is actually used, and
only include that code for distribution. During development you can also get a
lot more feedback like 'this statement has no effect'.

Another important use-case here is testing. In the future, ValueScript will
include `vs.staticTest(name, fn)` which accepts a function taking no arguments,
which can therefore be computed statically. The compiler will emit an error if
the test fails.

</details>

## Status

ValueScript is in early development. There may be some descriptions of
ValueScript elsewhere here that represent how ValueScript is intended to work,
not the subset of ValueScript that has actually been implemented.

A lot of the essential language features are implemented, including:

- Classes
- Closures
- Loops
- Recursion

ValueScript doesn't yet bind to the outside world (including TypeScript
interop), except that excess command line arguments are passed to the main
function as strings.
