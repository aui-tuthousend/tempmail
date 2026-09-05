use std::convert::Infallible;
use std::time::Duration;

use async_stream::stream;
use axum::extract::{Path, State};
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures_util::Stream;
use futures_util::StreamExt;
use shared::events::{DomainEvent, EMAIL_RECEIVED_CHANNEL};
use shared::TempMailError;
use tracing::{error, warn};

use crate::dto::{
    AccountIdPath, AccountResponse, CreateAccountRequest, ErrorResponse, GenerateMailboxResponse,
    ListMessagesResponse, LoginRequest, LoginResponse, MailboxPath, MessageIdPath, MessageResponse,
    SessionAccountResponse, UpdateMessageRequest,
};
use crate::state::AppState;

const API_KEY_HEADER: &str = "x-api-key";

pub async fn health() -> &'static str {
    "ok"
}

pub async fn create_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<AccountResponse>), (StatusCode, Json<ErrorResponse>)> {
    let api_key = headers
        .get(API_KEY_HEADER)
        .and_then(|value| value.to_str().ok());

    state
        .account_service
        .create_account(api_key, request)
        .await
        .map(|account| (StatusCode::CREATED, Json(account.into())))
        .map_err(error_response)
}

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> Result<(StatusCode, HeaderMap, Json<LoginResponse>), (StatusCode, Json<ErrorResponse>)> {
    let cookie_name = &state.config.session_cookie_name;
    let existing_token = session_token_from_headers(&headers, cookie_name);
    let login = state
        .session_service
        .login(existing_token.as_deref(), request)
        .await
        .map_err(error_response)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        session_cookie_header(
            cookie_name,
            &login.token,
            state.config.session_ttl_seconds,
            state.config.session_cookie_secure,
        )
        .map_err(|error| {
            error_response(TempMailError::InvalidEnv {
                name: "SESSION_COOKIE_NAME",
                message: error.to_string(),
            })
        })?,
    );

    Ok((StatusCode::OK, headers, Json(login.response)))
}

pub async fn session_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<SessionAccountResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let token = session_token_from_headers(&headers, &state.config.session_cookie_name);
    state
        .session_service
        .session_accounts(token.as_deref())
        .await
        .map(Json)
        .map_err(error_response)
}

pub async fn activate_session_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(path): Path<AccountIdPath>,
) -> Result<Json<Vec<SessionAccountResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let token = session_token_from_headers(&headers, &state.config.session_cookie_name);
    state
        .session_service
        .activate_account(token.as_deref(), path.account_id)
        .await
        .map(Json)
        .map_err(error_response)
}

pub async fn list_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<MessageResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let token = session_token_from_headers(&headers, &state.config.session_cookie_name);
    state
        .message_service
        .list_messages(token.as_deref())
        .await
        .map(|messages| Json(messages.into_iter().map(MessageResponse::from).collect()))
        .map_err(error_response)
}

pub async fn get_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(path): Path<MessageIdPath>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token = session_token_from_headers(&headers, &state.config.session_cookie_name);
    state
        .message_service
        .get_message(token.as_deref(), path.message_id)
        .await
        .map(MessageResponse::from)
        .map(Json)
        .map_err(error_response)
}

pub async fn update_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(path): Path<MessageIdPath>,
    Json(request): Json<UpdateMessageRequest>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token = session_token_from_headers(&headers, &state.config.session_cookie_name);
    state
        .message_service
        .update_message(token.as_deref(), path.message_id, request)
        .await
        .map(MessageResponse::from)
        .map(Json)
        .map_err(error_response)
}

pub async fn generate_mailbox(State(state): State<AppState>) -> Json<GenerateMailboxResponse> {
    let mailbox = state.mailbox_service.generate_mailbox().await;

    Json(GenerateMailboxResponse {
        address: mailbox.address.as_string(),
        mailbox,
    })
}

pub async fn list_mailbox_messages(
    State(state): State<AppState>,
    Path(path): Path<MailboxPath>,
) -> Json<ListMessagesResponse> {
    let mailbox = path.mailbox;
    let messages = state.mailbox_service.list_messages(&mailbox).await;

    Json(ListMessagesResponse { mailbox, messages })
}

pub async fn message_events(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, Json<ErrorResponse>)> {
    let token = session_token_from_headers(&headers, &state.config.session_cookie_name);
    let token = token.ok_or_else(|| error_response(TempMailError::InvalidSession))?;
    state
        .session_service
        .active_account_id(Some(&token))
        .await
        .map_err(error_response)?;
    let redis_url = state.event_service.redis_url().to_owned();
    let session_service = state.session_service.clone();

    let events = stream! {
        match redis::Client::open(redis_url) {
            Ok(client) => match client.get_async_pubsub().await {
                Ok(mut pubsub) => {
                    if let Err(error) = pubsub.subscribe(EMAIL_RECEIVED_CHANNEL).await {
                        error!(%error, "failed to subscribe to Redis Pub/Sub");
                        yield Ok(Event::default().event("error").data("subscription_failed"));
                        return;
                    }

                    let mut messages = pubsub.on_message();
                    while let Some(message) = messages.next().await {
                        let payload = match message.get_payload::<String>() {
                            Ok(payload) => payload,
                            Err(error) => {
                                warn!(%error, "failed to read Pub/Sub payload");
                                continue;
                            }
                        };

                        let event = match serde_json::from_str::<DomainEvent>(&payload) {
                            Ok(event) => event,
                            Err(error) => {
                                warn!(%error, "failed to decode domain event");
                                continue;
                            }
                        };

                        let DomainEvent::EmailReceived(email_received) = event;
                        let current_account_id = match session_service.active_account_id(Some(&token)).await {
                            Ok(account_id) => account_id,
                            Err(error) => {
                                warn!(%error, "session became invalid during SSE stream");
                                yield Ok(Event::default().event("error").data("invalid_session"));
                                return;
                            }
                        };

                        if email_received.account_id != current_account_id {
                            continue;
                        }

                        yield Ok(Event::default()
                            .event("email.received")
                            .id(email_received.message_id.to_string())
                            .data(payload));
                    }
                }
                Err(error) => {
                    error!(%error, "failed to create Redis Pub/Sub connection");
                    yield Ok(Event::default().event("error").data("pubsub_connection_failed"));
                }
            },
            Err(error) => {
                error!(%error, "failed to create Redis client");
                yield Ok(Event::default().event("error").data("redis_client_failed"));
            }
        }
    };

    Ok(Sse::new(events).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

pub async fn inbox_events(
    State(state): State<AppState>,
    Path(path): Path<MailboxPath>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mailbox = path.mailbox;
    let redis_url = state.event_service.redis_url().to_owned();

    let events = stream! {
        match redis::Client::open(redis_url) {
            Ok(client) => match client.get_async_pubsub().await {
                Ok(mut pubsub) => {
                    if let Err(error) = pubsub.subscribe(EMAIL_RECEIVED_CHANNEL).await {
                        error!(%error, "failed to subscribe to Redis Pub/Sub");
                        yield Ok(Event::default().event("error").data("subscription_failed"));
                        return;
                    }

                    let mut messages = pubsub.on_message();
                    while let Some(message) = messages.next().await {
                        let payload = match message.get_payload::<String>() {
                            Ok(payload) => payload,
                            Err(error) => {
                                warn!(%error, "failed to read Pub/Sub payload");
                                continue;
                            }
                        };

                        let event = match serde_json::from_str::<DomainEvent>(&payload) {
                            Ok(event) => event,
                            Err(error) => {
                                warn!(%error, "failed to decode domain event");
                                continue;
                            }
                        };

                        let DomainEvent::EmailReceived(email_received) = event;
                        if email_received.mailbox != mailbox {
                            continue;
                        }

                        yield Ok(Event::default()
                            .event("email.received")
                            .id(email_received.message_id.to_string())
                            .data(payload));
                    }
                }
                Err(error) => {
                    error!(%error, "failed to create Redis Pub/Sub connection");
                    yield Ok(Event::default().event("error").data("pubsub_connection_failed"));
                }
            },
            Err(error) => {
                error!(%error, "failed to create Redis client");
                yield Ok(Event::default().event("error").data("redis_client_failed"));
            }
        }
    };

    Sse::new(events).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

fn session_token_from_headers(headers: &HeaderMap, cookie_name: &str) -> Option<String> {
    let cookie = headers.get(COOKIE)?.to_str().ok()?;

    cookie.split(';').find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        (name == cookie_name).then(|| value.to_owned())
    })
}

fn session_cookie_header(
    cookie_name: &str,
    token: &str,
    ttl_seconds: i64,
    secure: bool,
) -> Result<HeaderValue, axum::http::header::InvalidHeaderValue> {
    let mut cookie =
        format!("{cookie_name}={token}; Max-Age={ttl_seconds}; Path=/; HttpOnly; SameSite=Lax");

    if secure {
        cookie.push_str("; Secure");
    }

    HeaderValue::from_str(&cookie)
}

fn error_response(error: TempMailError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match error {
        TempMailError::ApiKeyRequired => StatusCode::UNAUTHORIZED,
        TempMailError::InvalidApiKey => StatusCode::FORBIDDEN,
        TempMailError::AuthenticationFailed => StatusCode::UNAUTHORIZED,
        TempMailError::InvalidSession | TempMailError::SessionExpired => StatusCode::UNAUTHORIZED,
        TempMailError::AuthorizationFailed => StatusCode::FORBIDDEN,
        TempMailError::InvalidEmailAddress(_) => StatusCode::BAD_REQUEST,
        TempMailError::Conflict(_) => StatusCode::CONFLICT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (
        status,
        Json(ErrorResponse {
            error: error.to_string(),
        }),
    )
}
