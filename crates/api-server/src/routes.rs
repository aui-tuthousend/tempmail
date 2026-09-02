use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::handlers::{generate_mailbox, inbox_events, list_messages};
use crate::state::AppState;

pub fn router(state: AppState, cors: CorsLayer) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/mailboxes", post(generate_mailbox))
        .route("/mailboxes/:mailbox/messages", get(list_messages))
        .route("/mailboxes/:mailbox/events", get(inbox_events))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
