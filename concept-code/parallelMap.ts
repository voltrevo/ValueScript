import { thread } from 'value-script';

export default function parallelMap<Value, MappedValue>(
  values: Value[],
  mapper: (value: Value) => MappedValue,
) {
  return values
    .map(v => thread(() => mapper(v)))
    .map(t => t());
}
