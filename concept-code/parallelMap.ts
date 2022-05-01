import { thread } from 'value-script';

export default function parallelMap<Value, MappedValue>(
  values: Value[],
  mapper: (value: Value) => MappedValue,
  concurrencyLimit = 10,
) {
  let poolSize = Math.min(values.length, concurrencyLimit);

  const pool = values
    .slice(0, poolSize)
    .map(v => thread(() => mapper(v)));

  let results: MappedValue[] = [];

  for (let i = 0; i < values.length; i++) {
    results[i] = pool[i % poolSize]();

    if (i + poolSize < values.length) {
      pool[i % poolSize] = thread(() => mapper(values[i + poolSize]));
    }
  }

  return results;
}
