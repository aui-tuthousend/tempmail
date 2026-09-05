use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::handlers::{
    activate_session_account, create_account, generate_mailbox, get_message, health, inbox_events,
    list_mailbox_messages, list_messages, login, message_events, session_accounts, update_message,
};
use crate::state::AppState;

pub fn router(state: AppState, cors: CorsLayer) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/accounts", post(create_account))
        .route("/sessions/login", post(login))
        .route("/session/accounts", get(session_accounts))
        .route(
            "/session/accounts/:account_id/activate",
            post(activate_session_account),
        )
        .route("/messages", get(list_messages))
        .route("/messages/events", get(message_events))
        .route(
            "/messages/:message_id",
            get(get_message).patch(update_message),
        )
        .route("/mailboxes", post(generate_mailbox))
        .route("/mailboxes/:mailbox/messages", get(list_mailbox_messages))
        .route("/mailboxes/:mailbox/events", get(inbox_events))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
