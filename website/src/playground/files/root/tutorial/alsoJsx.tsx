// You can also use JSX.

export default function main() {
  return greet("world");
}

function greet(name: string) {
  return (
    <h1>
      Hello
      <span style="color: green;">{name}!</span>
    </h1>
  );
}
