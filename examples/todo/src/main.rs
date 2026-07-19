use std::{
    env,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use axum::{
    Json, Router,
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, patch, post},
};
use axum_inertia::{Inertia, InertiaConfig, vite};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;

const CLIENT_ENTRY: &str = "src/main.tsx";
const CLIENT_DIST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/client/dist");
const MANIFEST_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/client/dist/.vite/manifest.json"
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct Todo {
    id: u64,
    title: String,
    completed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TodoError {
    InvalidTitle,
    NotFound,
}

#[derive(Clone, Default)]
struct TodoStore {
    items: Arc<RwLock<Vec<Todo>>>,
    next_id: Arc<AtomicU64>,
}

impl TodoStore {
    fn with_examples() -> Self {
        Self {
            items: Arc::new(RwLock::new(vec![
                Todo {
                    id: 1,
                    title: "Review the Axum and Inertia setup".into(),
                    completed: true,
                },
                Todo {
                    id: 2,
                    title: "Add your first todo".into(),
                    completed: false,
                },
            ])),
            next_id: Arc::new(AtomicU64::new(3)),
        }
    }

    async fn all(&self) -> Vec<Todo> {
        self.items.read().await.clone()
    }

    async fn create(&self, title: String) -> Result<(), TodoError> {
        let todo = Todo {
            id: self.next_id.fetch_add(1, Ordering::Relaxed),
            title: normalize_title(title)?,
            completed: false,
        };
        self.items.write().await.push(todo);
        Ok(())
    }

    async fn rename(&self, id: u64, title: String) -> Result<(), TodoError> {
        let title = normalize_title(title)?;
        let mut items = self.items.write().await;
        let todo = items
            .iter_mut()
            .find(|todo| todo.id == id)
            .ok_or(TodoError::NotFound)?;
        todo.title = title;
        Ok(())
    }

    async fn toggle(&self, id: u64) -> Result<(), TodoError> {
        let mut items = self.items.write().await;
        let todo = items
            .iter_mut()
            .find(|todo| todo.id == id)
            .ok_or(TodoError::NotFound)?;
        todo.completed = !todo.completed;
        Ok(())
    }

    async fn delete(&self, id: u64) -> Result<(), TodoError> {
        let mut items = self.items.write().await;
        let original_len = items.len();
        items.retain(|todo| todo.id != id);
        if items.len() == original_len {
            return Err(TodoError::NotFound);
        }
        Ok(())
    }

    async fn clear_completed(&self) {
        self.items.write().await.retain(|todo| !todo.completed);
    }
}

fn normalize_title(title: String) -> Result<String, TodoError> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 120 {
        return Err(TodoError::InvalidTitle);
    }
    Ok(title.to_owned())
}

#[derive(Clone)]
struct AppState {
    inertia: InertiaConfig,
    todos: TodoStore,
}

impl FromRef<AppState> for InertiaConfig {
    fn from_ref(state: &AppState) -> Self {
        state.inertia.clone()
    }
}

#[derive(Deserialize)]
struct CreateTodo {
    title: String,
}

#[derive(Deserialize)]
struct UpdateTodo {
    title: String,
}

#[derive(Default, Deserialize)]
struct TodoQuery {
    filter: Option<String>,
}

struct ApiError(TodoError);

impl From<TodoError> for ApiError {
    fn from(error: TodoError) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0 {
            TodoError::InvalidTitle => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Todo title must contain between 1 and 120 characters",
            ),
            TodoError::NotFound => (StatusCode::NOT_FOUND, "Todo not found"),
        };

        (status, Json(json!({ "message": message }))).into_response()
    }
}

fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/todos", post(create_todo))
        .route("/todos/completed", delete(clear_completed))
        .route("/todos/{id}", patch(update_todo).delete(delete_todo))
        .route("/todos/{id}/toggle", patch(toggle_todo))
}

async fn index(
    inertia: Inertia,
    Query(query): Query<TodoQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let filter = normalize_filter(query.filter.as_deref());
    inertia.render(
        "Todos/Index",
        json!({ "todos": state.todos.all().await, "filter": filter }),
    )
}

fn normalize_filter(filter: Option<&str>) -> &'static str {
    match filter {
        Some("active") => "active",
        Some("completed") => "completed",
        _ => "all",
    }
}

async fn create_todo(
    State(state): State<AppState>,
    Json(input): Json<CreateTodo>,
) -> Result<Redirect, ApiError> {
    state.todos.create(input.title).await?;
    Ok(Redirect::to("/"))
}

async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<UpdateTodo>,
) -> Result<Redirect, ApiError> {
    state.todos.rename(id, input.title).await?;
    Ok(Redirect::to("/"))
}

async fn toggle_todo(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Redirect, ApiError> {
    state.todos.toggle(id).await?;
    Ok(Redirect::to("/"))
}

async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Redirect, ApiError> {
    state.todos.delete(id).await?;
    Ok(Redirect::to("/"))
}

async fn clear_completed(State(state): State<AppState>) -> Redirect {
    state.todos.clear_completed().await;
    Redirect::to("/")
}

fn inertia_config() -> InertiaConfig {
    if env::var("APP_ENV").is_ok_and(|value| value == "production") {
        vite::Production::new(MANIFEST_PATH, CLIENT_ENTRY)
            .unwrap_or_else(|error| {
                panic!("failed to read the Vite manifest at {MANIFEST_PATH}: {error}")
            })
            .lang("en")
            .title("Todo — Axum × Inertia")
            .into_config()
    } else {
        vite::Development::default()
            .port(5173)
            .main(CLIENT_ENTRY)
            .lang("en")
            .title("Todo — Axum × Inertia")
            .react()
            .into_config()
    }
}

fn app(state: AppState) -> Router {
    routes()
        .fallback_service(ServeDir::new(CLIENT_DIST))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "todo=debug,tower_http=debug".into()),
        )
        .init();

    let state = AppState {
        inertia: inertia_config(),
        todos: TodoStore::with_examples(),
    };
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("port 3000 is unavailable");

    info!("Todo app listening on http://localhost:3000");
    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Ctrl+C handler could not be installed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_must_contain_between_one_and_120_characters() {
        assert_eq!(
            normalize_title("  Write tests  ".into()).unwrap(),
            "Write tests"
        );
        assert_eq!(normalize_title("   ".into()), Err(TodoError::InvalidTitle));
        assert_eq!(
            normalize_title("a".repeat(121)),
            Err(TodoError::InvalidTitle)
        );
    }

    #[test]
    fn filter_query_is_normalized() {
        assert_eq!(normalize_filter(None), "all");
        assert_eq!(normalize_filter(Some("active")), "active");
        assert_eq!(normalize_filter(Some("completed")), "completed");
        assert_eq!(normalize_filter(Some("unknown")), "all");
    }

    #[tokio::test]
    async fn store_supports_the_todo_lifecycle() {
        let store = TodoStore::default();

        store.create("  Write tests  ".into()).await.unwrap();
        let created = store.all().await;
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].title, "Write tests");
        assert!(!created[0].completed);

        let id = created[0].id;
        store
            .rename(id, "Make the tests pass".into())
            .await
            .unwrap();
        store.toggle(id).await.unwrap();
        let updated = store.all().await;
        assert_eq!(updated[0].title, "Make the tests pass");
        assert!(updated[0].completed);

        store.clear_completed().await;
        assert!(store.all().await.is_empty());
    }

    #[tokio::test]
    async fn store_rejects_invalid_titles_and_unknown_ids() {
        let store = TodoStore::default();

        assert_eq!(
            store.create("   ".into()).await,
            Err(TodoError::InvalidTitle)
        );
        assert_eq!(store.toggle(999).await, Err(TodoError::NotFound));
        assert_eq!(store.delete(999).await, Err(TodoError::NotFound));
    }
}
