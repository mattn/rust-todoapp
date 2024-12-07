use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tower_http::services::ServeDir;

async fn do_list(State(state): State<MyState>) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Task>("SELECT * FROM tasks ORDER BY id")
        .fetch_all(&state.pool)
        .await
    {
        Ok(tasks) => Ok((StatusCode::OK, Json(tasks))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn do_get(
    Path(id): Path<i32>,
    State(state): State<MyState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
    {
        Ok(task) => Ok((StatusCode::OK, Json(task))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn do_create(
    State(state): State<MyState>,
    Json(data): Json<TaskNew>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Task>(
        "INSERT INTO tasks (text, completed) VALUES ($1, false) RETURNING id, text, completed",
    )
    .bind(&data.text)
    .fetch_one(&state.pool)
    .await
    {
        Ok(task) => Ok((StatusCode::CREATED, Json(task))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn do_complete(
    Path(id): Path<i32>,
    State(state): State<MyState>,
    Json(data): Json<TaskComplete>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Task>(
        "UPDATE tasks SET completed =$1 WHERE id = $2 RETURNING id, text, completed",
    )
    .bind(data.completed)
    .bind(id)
    .fetch_one(&state.pool)
    .await
    {
        Ok(task) => Ok((StatusCode::OK, Json(task))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn do_delete(
    Path(id): Path<i32>,
    State(state): State<MyState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Task>("DELETE FROM tasks WHERE id = $1 RETURNING id, text, completed")
        .bind(id)
        .fetch_one(&state.pool)
        .await
    {
        Ok(task) => Ok((StatusCode::OK, Json(task))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

#[derive(Clone)]
struct MyState {
    pool: PgPool,
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let state = MyState { pool };
    let router = Router::new()
        .route("/tasks", get(do_list))
        .route("/tasks", post(do_create))
        .route("/tasks/:id", get(do_get))
        .route("/tasks/:id", post(do_complete))
        .route("/tasks/:id", delete(do_delete))
        .nest_service("/", ServeDir::new("assets"))
        .with_state(state);

    Ok(router.into())
}

#[derive(Deserialize)]
struct TaskNew {
    pub text: String,
}

#[derive(Deserialize)]
struct TaskComplete {
    pub completed: bool,
}

#[derive(Serialize, FromRow)]
struct Task {
    pub id: i32,
    pub text: String,
    pub completed: bool,
}
