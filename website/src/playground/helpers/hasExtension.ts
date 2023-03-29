export default function hasExtension(path: string) {
  return path.split('/').at(-1)?.includes('.') ?? false;
}