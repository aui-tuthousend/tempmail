use axum::http::HeaderName;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

use crate::handlers::{
    account_availability, activate_session_account, create_account, delete_message,
    generate_mailbox, get_message, health, inbox_events, list_mailbox_messages, list_messages,
    login, logout, message_events, session_accounts, update_message,
};
use crate::state::AppState;

const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

pub fn router(state: AppState, cors: CorsLayer) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/accounts", post(create_account))
        .route("/accounts/availability", get(account_availability))
        .route("/sessions/login", post(login))
        .route("/sessions/logout", post(logout))
        .route("/session/accounts", get(session_accounts))
        .route(
            "/session/accounts/:account_id/activate",
            post(activate_session_account),
        )
        .route("/messages", get(list_messages))
        .route("/messages/events", get(message_events))
        .route(
            "/messages/:message_id",
            get(get_message)
                .patch(update_message)
                .delete(delete_message),
        )
        .route("/mailboxes", post(generate_mailbox))
        .route("/mailboxes/:mailbox/messages", get(list_mailbox_messages))
        .route("/mailboxes/:mailbox/events", get(inbox_events))
        .layer(PropagateRequestIdLayer::new(X_REQUEST_ID.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true)),
        )
        .layer(SetRequestIdLayer::new(X_REQUEST_ID, MakeRequestUuid))
        .layer(cors)
        .with_state(state)
}
