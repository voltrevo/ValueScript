# ValueScript

## About the language

- [Try the playground](https://valuescript.org/playground)
- [Learn more](https://github.com/voltrevo/ValueScript)

## This module

This is a module for running ValueScript in JavaScript.

ValueScript is in early development and this module is especially unlikely to be useful externally right now. Please drop me a line if you want to see this improved :).

It's possible you will find the lower level `WorkerPool` and `WorkerSlot` utilities useful:

```ts
import { WorkerPool } from 'valuescript';

function task() {
  console.log('Hi from pool');
}

const script = [
  'self.postMessage("ready");',
  task.toString(),
  'task();',
].join('\n\n');

const scriptUrl = URL.createObjectURL(
  new Blob([script], { type: 'application/javascript' }),
);

const pool = new WorkerPool(scriptUrl);

pool.use(async (worker, terminate) => {
  // Exclusive access to worker
  // terminate() terminates the worker and leads to the creation of a new one
  // for use by other consumers
});
```
