// This is an SWC bug. It appears to be fixed in the latest version, but the latest version also
// doesn't compile in stable rust. Hopefully this is also fixed in the latest stable version, we
// just need to figure out what that is.
// Related: https://github.com/swc-project/cli/issues/218.

export default function main() {
  return [3n < "asdf", 3n >= "asdf"];
}
