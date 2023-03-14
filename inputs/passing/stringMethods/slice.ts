// test_output! ["","","","🎨","🎨","🎨","한🎨","한🎨","🍹abc£한","🚀🍹abc£한🎨","🚀🍹abc£한🎨","🚀🍹abc£한🎨",""]

export default function () {
  const str = "🚀🍹abc£한🎨";

  return [
    str.slice(-1),
    str.slice(-2),
    str.slice(-3),
    str.slice(-4),
    str.slice(-5),
    str.slice(-6),
    str.slice(-7),
    str.slice(-8),
    str.slice(1, -1),
    str.slice(),
    str.slice(0, 100),
    str.slice(0, 20),
    str.slice(10, -11),
  ];
}
