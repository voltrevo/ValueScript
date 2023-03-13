// test_output! [128640,undefined,undefined,undefined,127865,undefined,undefined,undefined,97,98,99,163,undefined,54620,undefined,undefined,127912,undefined,undefined,undefined]

// ValueScript uses utf8, so it deviates from JavaScript's utf16.
// We're also not really doing the utf8 equivalent of JavaScript's utf16 either.
// JavaScript will interpret something at all positions inside the string, but
// we return undefined at invalid positions instead.

export default function () {
  const str = "🚀🍹abc£한🎨";
  let outputs = [];

  for (let i = 0; i < str.length; i++) {
    outputs.push(str.codePointAt(i));
  }

  return outputs;
}
