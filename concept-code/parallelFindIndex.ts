import { thread } from 'value-script';

export default function parallelFindIndex<Value>(
  values: Value[],
  predicate: (value: Value) => boolean,
  concurrencyLimit = 10,
): number | undefined {
  let poolSize = Math.min(values.length, concurrencyLimit);

  const pool = values
    .slice(0, poolSize)
    .map(v => thread(() => predicate(v)));

  for (let i = 0; i < values.length; i++) {
    if (pool[i % poolSize]()) {
      // When exiting here, everything gets dereferenced, which will notify all
      // the other threads to stop working and clean up
      return i;
    }

    if (i + poolSize < values.length) {
      pool[i % poolSize] = thread(() => predicate(values[i + poolSize]));
    }
  }

  return undefined;
}
