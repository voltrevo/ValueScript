export default function main() {
  let pirateEnabled = false;

  function greet() {
    if (!pirateEnabled) {
      return "Hi";
    }

    return "Ahoy";
  }

  function enablePirate() {
    pirateEnabled = true;
    return "Done";
  }

  return [
    greet(),
    enablePirate(),
    greet(),
  ];
}