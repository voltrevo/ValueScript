export default function main() {
  let actor = new Actor();

  return [
    actor.greet(),
    actor.enablePirate(),
    actor.greet(),
  ];
}

class Actor {
  pirateEnabled = false;

  greet() {
    if (!this.pirateEnabled) {
      return "Hi";
    }

    return "Ahoy";
  }

  enablePirate() {
    this.pirateEnabled = true;
    return "Done";
  }
}