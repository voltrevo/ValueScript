// ValueScript supports JSX 🚀.

export default function main() {
  return greet("world");
}

function greet(name: string) {
  return (
    <h1>
      Hello
      <span style="color: green;">{name}</span>!
    </h1>
  );
}

// Note: This is not a react-like framework, it's just a literal evaluation.
// The nature of frontends in ValueScript is still being explored.
