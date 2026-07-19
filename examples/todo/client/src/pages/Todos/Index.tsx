import { Head, router } from "@inertiajs/react";
import {
  type FormEvent,
  type KeyboardEvent,
  useMemo,
  useRef,
  useState,
} from "react";

type Todo = {
  id: number;
  title: string;
  completed: boolean;
};

type Props = {
  todos: Todo[];
  filter: Filter;
};

type Filter = "all" | "active" | "completed";

const filters: Array<{ value: Filter; label: string }> = [
  { value: "all", label: "All" },
  { value: "active", label: "Active" },
  { value: "completed", label: "Completed" },
];

export default function Index({ todos, filter }: Props) {
  const [newTitle, setNewTitle] = useState("");
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editingTitle, setEditingTitle] = useState("");
  const [busyIds, setBusyIds] = useState<number[]>([]);
  const [creating, setCreating] = useState(false);
  const newTodoInput = useRef<HTMLInputElement>(null);

  const activeCount = todos.filter((todo) => !todo.completed).length;
  const completedCount = todos.length - activeCount;
  const completion = todos.length === 0 ? 0 : Math.round((completedCount / todos.length) * 100);

  const visibleTodos = useMemo(() => {
    if (filter === "active") return todos.filter((todo) => !todo.completed);
    if (filter === "completed") return todos.filter((todo) => todo.completed);
    return todos;
  }, [filter, todos]);

  const markBusy = (id: number, busy: boolean) => {
    setBusyIds((current) =>
      busy ? [...current, id] : current.filter((currentId) => currentId !== id),
    );
  };

  const addTodo = (event: FormEvent) => {
    event.preventDefault();
    const title = newTitle.trim();
    if (!title || creating) return;

    router.post(
      "/todos",
      { title },
      {
        preserveScroll: true,
        onStart: () => setCreating(true),
        onSuccess: () => setNewTitle(""),
        onFinish: () => {
          setCreating(false);
          newTodoInput.current?.focus();
        },
      },
    );
  };

  const toggleTodo = (todo: Todo) => {
    if (busyIds.includes(todo.id)) return;
    router.patch(
      `/todos/${todo.id}/toggle`,
      {},
      {
        preserveScroll: true,
        onStart: () => markBusy(todo.id, true),
        onFinish: () => markBusy(todo.id, false),
      },
    );
  };

  const beginEditing = (todo: Todo) => {
    setEditingId(todo.id);
    setEditingTitle(todo.title);
  };

  const saveTodo = (todo: Todo) => {
    const title = editingTitle.trim();
    if (!title || title === todo.title) {
      setEditingId(null);
      return;
    }

    router.patch(
      `/todos/${todo.id}`,
      { title },
      {
        preserveScroll: true,
        onStart: () => markBusy(todo.id, true),
        onSuccess: () => setEditingId(null),
        onFinish: () => markBusy(todo.id, false),
      },
    );
  };

  const handleEditKey = (event: KeyboardEvent<HTMLInputElement>, todo: Todo) => {
    if (event.key === "Enter") saveTodo(todo);
    if (event.key === "Escape") setEditingId(null);
  };

  const deleteTodo = (todo: Todo) => {
    if (busyIds.includes(todo.id)) return;
    router.delete(`/todos/${todo.id}`, {
      preserveScroll: true,
      onStart: () => markBusy(todo.id, true),
      onFinish: () => markBusy(todo.id, false),
    });
  };

  const clearCompleted = () => {
    router.delete("/todos/completed", { preserveScroll: true });
  };

  const selectFilter = (nextFilter: Filter) => {
    const query = nextFilter === "all" ? {} : { filter: nextFilter };
    router.get("/", query, {
      preserveScroll: true,
      preserveState: true,
    });
  };

  return (
    <>
      <Head title="My Todo" />
      <main className="app-shell">
        <section className="hero" aria-labelledby="page-title">
          <div className="eyebrow"><span /> DAILY FOCUS</div>
          <h1 id="page-title">Make today, <em>one task at a time.</em></h1>
          <p>Get it out of your head. Build momentum one small step at a time.</p>
        </section>

        <section className="todo-card" aria-label="Todo list">
          <form className="add-form" onSubmit={addTodo}>
            <label className="sr-only" htmlFor="new-todo">New todo</label>
            <span className="add-icon" aria-hidden="true">＋</span>
            <input
              ref={newTodoInput}
              id="new-todo"
              value={newTitle}
              onChange={(event) => setNewTitle(event.target.value)}
              placeholder="What needs to be done?"
              maxLength={120}
              autoComplete="off"
            />
            <button type="submit" disabled={!newTitle.trim() || creating}>
              {creating ? "Adding…" : "Add task"}
            </button>
          </form>

          <div className="toolbar">
            <div className="filters" aria-label="Filter todos">
              {filters.map((item) => (
                <button
                  key={item.value}
                  type="button"
                  className={filter === item.value ? "active" : ""}
                  onClick={() => selectFilter(item.value)}
                  aria-pressed={filter === item.value}
                >
                  {item.label}
                  {item.value === "all" && <span>{todos.length}</span>}
                </button>
              ))}
            </div>
            <p className="remaining">
              <strong>{activeCount}</strong> {activeCount === 1 ? "task" : "tasks"} left
            </p>
          </div>

          <div className="todo-list" aria-live="polite">
            {visibleTodos.length > 0 ? (
              visibleTodos.map((todo) => {
                const busy = busyIds.includes(todo.id);
                return (
                  <article
                    key={todo.id}
                    className={`todo-row ${todo.completed ? "completed" : ""} ${busy ? "busy" : ""}`}
                  >
                    <button
                      type="button"
                      className="check-button"
                      onClick={() => toggleTodo(todo)}
                      aria-label={todo.completed ? `Mark ${todo.title} as active` : `Mark ${todo.title} as completed`}
                      aria-pressed={todo.completed}
                      disabled={busy}
                    >
                      <span aria-hidden="true">✓</span>
                    </button>

                    <div className="todo-content">
                      {editingId === todo.id ? (
                        <input
                          className="edit-input"
                          value={editingTitle}
                          onChange={(event) => setEditingTitle(event.target.value)}
                          onBlur={() => saveTodo(todo)}
                          onKeyDown={(event) => handleEditKey(event, todo)}
                          maxLength={120}
                          autoFocus
                          aria-label="Edit todo title"
                        />
                      ) : (
                        <button
                          type="button"
                          className="todo-title"
                          onDoubleClick={() => beginEditing(todo)}
                          onClick={() => beginEditing(todo)}
                          title="Click to edit"
                        >
                          {todo.title}
                        </button>
                      )}
                      <span className="todo-state">{todo.completed ? "Completed" : "Active"}</span>
                    </div>

                    <button
                      type="button"
                      className="delete-button"
                      onClick={() => deleteTodo(todo)}
                      aria-label={`Delete ${todo.title}`}
                      disabled={busy}
                    >
                      <span aria-hidden="true">×</span>
                    </button>
                  </article>
                );
              })
            ) : (
              <div className="empty-state">
                <div className="empty-check" aria-hidden="true">✓</div>
                <h2>{todos.length === 0 ? "Nothing planned yet" : "No matching todos"}</h2>
                <p>{todos.length === 0 ? "Add your first step using the field above." : "Try selecting a different filter."}</p>
              </div>
            )}
          </div>

          <footer className="card-footer">
            <div className="progress-copy">
              <span>Today's progress</span>
              <strong>{completion}%</strong>
            </div>
            <div className="progress-track" aria-label={`Today's progress: ${completion}%`}>
              <span style={{ width: `${completion}%` }} />
            </div>
            <button
              type="button"
              className="clear-button"
              onClick={clearCompleted}
              disabled={completedCount === 0}
            >
              Clear completed
            </button>
          </footer>
        </section>

        <p className="hint">Tip: Click a todo to edit it</p>
      </main>
    </>
  );
}
