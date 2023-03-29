import nil from '../helpers/nil';
import raw from './raw';

export const orderedFiles = [
  '/tutorial/hello.ts',
  '/tutorial/alsoJavaScript.js',
  '/tutorial/valueSemantics.ts',
  '/tutorial/cantMutateCaptures.ts',
  '/tutorial/classBehavior.ts',
  '/tutorial/consistency.js',
  '/tutorial/revertOnCatch.ts',
  '/tutorial/strings.ts',
  '/tutorial/binaryTree.ts',
  '/tutorial/specialFunctions.ts',
  '/tutorial/treeShaking.ts',
];

export const defaultFiles: Record<string, string | nil> = {
  ...pick(raw, orderedFiles),
  ...raw,
};

console.log(Object.entries(defaultFiles));

function pick<T, K extends keyof T>(obj: T, keys: K[]): Pick<T, K> {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const result: any = {};

  for (const key of keys) {
    result[key] = obj[key];
  }

  return result;
}
