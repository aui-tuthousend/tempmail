use std::sync::Arc;

use redis::aio::ConnectionManager;
use sqlx::PgPool;

use crate::config::ApiConfig;
use crate::services::{
    AccountService, EventService, MailboxService, MessageService, SessionService,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ApiConfig>,
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub mailbox_service: MailboxService,
    pub event_service: EventService,
    pub account_service: AccountService,
    pub session_service: SessionService,
    pub message_service: MessageService,
}

impl AppState {
    pub fn new(
        config: ApiConfig,
        db: PgPool,
        redis: ConnectionManager,
        mailbox_service: MailboxService,
        event_service: EventService,
        account_service: AccountService,
        session_service: SessionService,
        message_service: MessageService,
    ) -> Self {
        Self {
            config: Arc::new(config),
            db,
            redis,
            mailbox_service,
            event_service,
            account_service,
            session_service,
            message_service,
        }
    }
}
