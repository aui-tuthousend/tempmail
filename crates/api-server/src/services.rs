use chrono::{Duration as ChronoDuration, Utc};
use shared::auth::{PasswordService, TokenService};
use shared::ids::new_uuid_v7;
use shared::models::{Account, EmailAddress, EmailMessage, Mailbox, Session, StoredMessage};
use shared::{Result, TempMailError};
use tracing::{error, warn};
use uuid::Uuid;

use crate::dto::{
    AccountAvailabilityQuery, AccountAvailabilityResponse, CreateAccountRequest, LoginRequest,
    LoginResponse, MessageView, SessionAccountResponse, UpdateMessageRequest,
};
use crate::repositories::{
    AccountRepository, ApiKeyRepository, MailboxRepository, MessageFlagsUpdate, MessageRepository,
    NewAccount, NewSession, SessionRepository,
};

#[derive(Clone)]
pub struct MailboxService {
    repository: MailboxRepository,
    domain: String,
    local_part_length: usize,
    ttl_seconds: u64,
}

impl MailboxService {
    pub fn new(
        repository: MailboxRepository,
        domain: String,
        local_part_length: usize,
        ttl_seconds: u64,
    ) -> Self {
        Self {
            repository,
            domain,
            local_part_length,
            ttl_seconds,
        }
    }

    pub async fn generate_mailbox(&self) -> Mailbox {
        let mailbox = self.new_mailbox();

        if let Err(error) = self
            .repository
            .store_mailbox(&mailbox, self.ttl_seconds)
            .await
        {
            error!(%error, "failed to store generated mailbox");
        }

        mailbox
    }

    pub async fn list_messages(&self, mailbox: &str) -> Vec<EmailMessage> {
        match self.repository.list_messages(mailbox).await {
            Ok(messages) => messages,
            Err(error) => {
                warn!(%error, %mailbox, "failed to load mailbox messages");
                Vec::new()
            }
        }
    }

    fn new_mailbox(&self) -> Mailbox {
        let local_part = Uuid::new_v4()
            .simple()
            .to_string()
            .chars()
            .take(self.local_part_length)
            .collect::<String>();
        let address = EmailAddress::new(local_part, self.domain.clone());
        let created_at = Utc::now();
        let expires_at = created_at + ChronoDuration::seconds(self.ttl_seconds as i64);

        Mailbox {
            id: Uuid::new_v4(),
            address,
            created_at,
            expires_at,
        }
    }
}

#[derive(Clone)]
pub struct AccountService {
    account_repository: AccountRepository,
    api_key_repository: ApiKeyRepository,
    password_service: PasswordService,
    token_service: TokenService,
}

impl AccountService {
    pub fn new(
        account_repository: AccountRepository,
        api_key_repository: ApiKeyRepository,
        password_service: PasswordService,
        token_service: TokenService,
    ) -> Self {
        Self {
            account_repository,
            api_key_repository,
            password_service,
            token_service,
        }
    }

    pub async fn availability(
        &self,
        query: AccountAvailabilityQuery,
        domain: &str,
    ) -> Result<AccountAvailabilityResponse> {
        let local_part = query
            .local_part
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_lowercase);
        let username = query
            .username
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);

        let (local_part_available, username_available) = self
            .account_repository
            .availability(local_part.as_deref(), username.as_deref(), domain)
            .await
            .map_err(|_| TempMailError::AuthorizationFailed)?;

        Ok(AccountAvailabilityResponse {
            local_part_available,
            username_available,
        })
    }

    pub async fn create_account(
        &self,
        api_key: Option<&str>,
        request: CreateAccountRequest,
    ) -> Result<Account> {
        self.validate_api_key(api_key).await?;

        let (local_part, domain) = parse_email(&request.email)?;
        let password_hash = self.password_service.hash_password(&request.password)?;
        let account = NewAccount {
            id: new_uuid_v7(),
            username: request.username,
            local_part,
            domain,
            password_hash,
            display_name: request.display_name,
        };

        self.account_repository
            .create(account)
            .await
            .map_err(map_account_create_error)
    }

    async fn validate_api_key(&self, api_key: Option<&str>) -> Result<()> {
        let api_key = api_key.ok_or(TempMailError::ApiKeyRequired)?;
        let key_hash = self.token_service.hash_token(api_key);
        let api_key_id = self
            .api_key_repository
            .find_active_by_hash(&key_hash)
            .await
            .map_err(|_| TempMailError::InvalidApiKey)?
            .ok_or(TempMailError::InvalidApiKey)?;

        self.api_key_repository
            .touch_last_used(api_key_id)
            .await
            .map_err(|_| TempMailError::InvalidApiKey)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct SessionLoginResult {
    pub token: String,
    pub response: LoginResponse,
}

#[derive(Clone)]
pub struct SessionService {
    repository: SessionRepository,
    password_service: PasswordService,
    token_service: TokenService,
    ttl_seconds: i64,
}

impl SessionService {
    pub fn new(
        repository: SessionRepository,
        password_service: PasswordService,
        token_service: TokenService,
        ttl_seconds: i64,
    ) -> Self {
        Self {
            repository,
            password_service,
            token_service,
            ttl_seconds,
        }
    }

    pub async fn login(
        &self,
        existing_token: Option<&str>,
        request: LoginRequest,
    ) -> Result<SessionLoginResult> {
        let credentials = self
            .repository
            .find_account_credentials(request.username_or_email.trim())
            .await
            .map_err(|_| TempMailError::AuthenticationFailed)?
            .ok_or(TempMailError::AuthenticationFailed)?;

        if !credentials.is_active
            || !self
                .password_service
                .verify_password(&request.password, &credentials.password_hash)?
        {
            return Err(TempMailError::AuthenticationFailed);
        }

        let (session, token) = self
            .get_or_create_session(existing_token, request.device_info)
            .await?;

        self.repository
            .attach_account_and_activate_if_needed(session.id, credentials.id, new_uuid_v7())
            .await
            .map_err(|_| TempMailError::InvalidSession)?;

        let accounts = self.session_accounts_by_id(session.id).await?;
        let active_account_id =
            active_account_id(&accounts).ok_or(TempMailError::InvalidSession)?;

        Ok(SessionLoginResult {
            token,
            response: LoginResponse {
                session_id: session.id,
                active_account_id,
                expires_at: session.expires_at,
                accounts,
            },
        })
    }

    pub async fn session_accounts(
        &self,
        token: Option<&str>,
    ) -> Result<Vec<SessionAccountResponse>> {
        let session = self.require_session(token).await?;
        self.session_accounts_by_id(session.id).await
    }

    pub async fn activate_account(
        &self,
        token: Option<&str>,
        account_id: Uuid,
    ) -> Result<Vec<SessionAccountResponse>> {
        let session = self.require_session(token).await?;
        let activated = self
            .repository
            .activate_account(session.id, account_id)
            .await
            .map_err(|_| TempMailError::InvalidSession)?;

        if !activated {
            return Err(TempMailError::AuthorizationFailed);
        }

        self.session_accounts_by_id(session.id).await
    }

    pub async fn logout_active_account(
        &self,
        token: Option<&str>,
    ) -> Result<Vec<SessionAccountResponse>> {
        let session = self.require_session(token).await?;
        let logged_out = self
            .repository
            .logout_active_account(session.id)
            .await
            .map_err(|_| TempMailError::InvalidSession)?;

        if !logged_out {
            return Err(TempMailError::InvalidSession);
        }

        self.session_accounts_by_id(session.id).await
    }

    pub async fn active_account_id(&self, token: Option<&str>) -> Result<Uuid> {
        let session = self.require_session(token).await?;
        let accounts = self.session_accounts_by_id(session.id).await?;
        active_account_id(&accounts).ok_or(TempMailError::InvalidSession)
    }

    async fn get_or_create_session(
        &self,
        existing_token: Option<&str>,
        device_info: Option<serde_json::Value>,
    ) -> Result<(Session, String)> {
        if let Some(token) = existing_token {
            let token_hash = self.token_service.hash_token(token);
            if let Some(session) = self
                .repository
                .find_valid_session_by_hash(&token_hash)
                .await
                .map_err(|_| TempMailError::InvalidSession)?
            {
                self.repository
                    .touch_session(session.id)
                    .await
                    .map_err(|_| TempMailError::InvalidSession)?;
                return Ok((session, token.to_owned()));
            }
        }

        let token_pair = self.token_service.generate_token_pair();
        let session = self
            .repository
            .create_session(NewSession {
                id: new_uuid_v7(),
                token_hash: token_pair.token_hash,
                device_info,
                expires_at: Utc::now() + ChronoDuration::seconds(self.ttl_seconds),
            })
            .await
            .map_err(|_| TempMailError::InvalidSession)?;

        Ok((session, token_pair.token))
    }

    async fn require_session(&self, token: Option<&str>) -> Result<Session> {
        let token = token.ok_or(TempMailError::InvalidSession)?;
        let token_hash = self.token_service.hash_token(token);
        let session = self
            .repository
            .find_valid_session_by_hash(&token_hash)
            .await
            .map_err(|_| TempMailError::InvalidSession)?
            .ok_or(TempMailError::InvalidSession)?;

        self.repository
            .touch_session(session.id)
            .await
            .map_err(|_| TempMailError::InvalidSession)?;

        Ok(session)
    }

    async fn session_accounts_by_id(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<SessionAccountResponse>> {
        self.repository
            .list_accounts(session_id)
            .await
            .map_err(|_| TempMailError::InvalidSession)
    }
}

#[derive(Clone)]
pub struct MessageService {
    repository: MessageRepository,
    session_service: SessionService,
}

impl MessageService {
    pub fn new(repository: MessageRepository, session_service: SessionService) -> Self {
        Self {
            repository,
            session_service,
        }
    }

    pub async fn list_messages(
        &self,
        token: Option<&str>,
        view: Option<MessageView>,
    ) -> Result<Vec<StoredMessage>> {
        let account_id = self.session_service.active_account_id(token).await?;
        self.repository
            .list_by_view(account_id, view)
            .await
            .map_err(|_| TempMailError::AuthorizationFailed)
    }

    pub async fn get_message(
        &self,
        token: Option<&str>,
        message_id: Uuid,
    ) -> Result<StoredMessage> {
        let account_id = self.session_service.active_account_id(token).await?;
        self.repository
            .find_by_id_for_account(message_id, account_id)
            .await
            .map_err(|_| TempMailError::AuthorizationFailed)?
            .ok_or(TempMailError::AuthorizationFailed)
    }

    pub async fn update_message(
        &self,
        token: Option<&str>,
        message_id: Uuid,
        request: UpdateMessageRequest,
    ) -> Result<StoredMessage> {
        let account_id = self.session_service.active_account_id(token).await?;
        let update = MessageFlagsUpdate {
            is_read: request.is_read,
            is_starred: request.is_starred,
            is_archived: request.is_archived,
            is_deleted: request.is_deleted,
        };

        self.repository
            .update_flags_for_account(message_id, account_id, update)
            .await
            .map_err(|_| TempMailError::AuthorizationFailed)?
            .ok_or(TempMailError::AuthorizationFailed)
    }

    pub async fn delete_message(&self, token: Option<&str>, message_id: Uuid) -> Result<()> {
        let account_id = self.session_service.active_account_id(token).await?;
        let deleted = self
            .repository
            .delete_for_account(message_id, account_id)
            .await
            .map_err(|_| TempMailError::AuthorizationFailed)?;

        if deleted {
            Ok(())
        } else {
            Err(TempMailError::AuthorizationFailed)
        }
    }
}

fn active_account_id(accounts: &[SessionAccountResponse]) -> Option<Uuid> {
    accounts
        .iter()
        .find(|account| account.is_active)
        .map(|account| account.account_id)
}

fn parse_email(email: &str) -> Result<(String, String)> {
    let email = email.trim().to_lowercase();
    let (local_part, domain) = email
        .split_once('@')
        .ok_or_else(|| TempMailError::InvalidEmailAddress(email.clone()))?;

    if local_part.is_empty() || domain.is_empty() || domain.contains('@') {
        return Err(TempMailError::InvalidEmailAddress(email));
    }

    Ok((local_part.to_owned(), domain.to_owned()))
}

fn map_account_create_error(error: sqlx::Error) -> TempMailError {
    if let sqlx::Error::Database(database_error) = &error {
        if database_error.is_unique_violation() {
            return TempMailError::Conflict("username or email already exists".to_owned());
        }
    }

    TempMailError::AuthorizationFailed
}

#[derive(Clone)]
pub struct EventService {
    redis_url: String,
}

impl EventService {
    pub fn new(redis_url: String) -> Self {
        Self { redis_url }
    }

    pub fn redis_url(&self) -> &str {
        &self.redis_url
    }
}
