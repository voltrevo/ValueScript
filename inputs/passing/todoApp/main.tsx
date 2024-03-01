export default class TodosApp {
  todos: Todo[] = [];

  ["GET /"]() {
    return (
      <html>
        <head>
          <title>Todos</title>
          {this.renderScript()}
        </head>
        <body>
          <h1>Todos</h1>
          <table>
            {this.renderTodos()}
            <tr>
              <td colSpan={2}>
                <input
                  id="todo"
                  onKeyDown={`onTodoInputKeyDown(event);` as ExplicitAny}
                />
              </td>
              <td>
                <button
                  onClick={`addTodo();` as ExplicitAny}
                >
                  Add
                </button>
              </td>
            </tr>
          </table>
        </body>
      </html>
    );
  }

  ["POST /add"](todo: string) {
    this.todos.push({ text: todo, done: false });

    return this.todos.length - 1;
  }

  ["POST /setDone"]({ index, done }: { index: number; done: boolean }) {
    this.todos[index].done = done;
  }

  ["POST /delete"](index: number) {
    this.todos.splice(index, 1);
  }

  renderTodos() {
    return this.todos.map((todo, i) => (
      <tr>
        <td>
          <input
            type="checkbox"
            checked={todo.done}
            onClick={`post('/setDone', { index: ${i}, done: ${!todo
              .done}, }); return false;` as ExplicitAny}
          />
        </td>
        <td style={{ textDecoration: todo.done ? "line-through" : "" }}>
          {todo.text}
        </td>
        <td
          style={{ cursor: "pointer" }}
          onClick={`post('/delete', ${i}); return false;` as ExplicitAny}
        >
          ❌
        </td>
      </tr>
    ));
  }

  renderScript() {
    return (
      <script>
        {`
          'use strict';

          async function post(path, body) {
            const res = await fetch(path, {
              method: "POST",
              body: JSON.stringify(body),
            });
  
            location.reload();
          }

          async function addTodo() {
            await post('/add', document.getElementById('todo').value);
          }

          async function onTodoInputKeyDown(e) {
            if (e.key === 'Enter') {
              await addTodo();
            }
          }
        `}
      </script>
    );
  }
}

type Todo = {
  text: string;
  done: boolean;
};

// deno-lint-ignore no-explicit-any
type ExplicitAny = any;
