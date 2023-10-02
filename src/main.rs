use std::sync::{Arc, Mutex};

use anyhow::Context;
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    todos: Mutex<Vec<String>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "with_axum_htmx_askama=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("hello, web server!");
    let app_state = Arc::new(AppState {
        todos: Mutex::new(vec![]),
    });
    let assets_path = std::env::current_dir().unwrap();

    let api_router = Router::new()
        .route("/hello", get(hello_from_the_server))
        // .route("/todos", post(add_todo))
        .with_state(app_state);

    let router = Router::new()
        .nest("/api", api_router)
        .route("/", get(hello))
        .route("/another-page", get(another_page))
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", assets_path.to_str().unwrap())),
        );
    let port = 8000_u16;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    info!("router intitialised, now listening on port {}", port);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .context("error while starting server")?;

    Ok(())
}

async fn hello() -> impl IntoResponse {
    let template = HelloTemplate {};
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate;

async fn another_page() -> impl IntoResponse {
    let template = AnotherPageTemplate {};
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "another-page.html")]
struct AnotherPageTemplate;

async fn hello_from_the_server() -> &'static str {
    "Hello!"
}

#[derive(Template)]
#[template(path = "todo-list.html")]
struct TodoList {
    todos: Vec<String>,
}

// async fn add_todo(
//     State(state): State<Arc<AppState>>,
//     Form(todo): Form<TodoList>,
// ) -> impl IntoResponse {
//     let mut lock = state.todos.lock().unwrap();
//     lock.push(todo.todos);
//
//     let template = TodoList {
//         todos: lock.clone(),
//     };
//
//     HtmlTemplate(template)
// }

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
