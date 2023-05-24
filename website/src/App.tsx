import { useState } from "react";

function App() {
  const [count, setCount] = useState(0);

  return (
    <div className="App">
      <button onClick={() => setCount((count) => count + 1)}>
        count is {count}
      </button>
    </div>
  );
}

export default App;
