import nil from '../helpers/nil';
import raw from './raw';

const files: Record<string, string | nil> = {
  ...pick(raw, [
    '/tutorial/hello.ts',
    '/tutorial/valueSemantics.ts',
    '/tutorial/revertOnCatch.ts',
    '/tutorial/binaryTree.ts',
  ]),
  ...raw,
};

export default files;

function pick<T, K extends keyof T>(obj: T, keys: K[]): Pick<T, K> {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const result: any = {};

  for (const key of keys) {
    result[key] = obj[key];
  }

  return result;
}
