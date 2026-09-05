# Project: TempMail - Persistent Email Service (Rust)

## 1. Goal

Membangun email service berbasis Rust yang awalnya temp mail, lalu berkembang menjadi persistent mailbox service dengan fitur:

- Account email permanen: username, email, password.
- Create account hanya boleh memakai API key valid.
- Satu account memiliki satu email address unik.
- Satu browser/device session bisa login ke beberapa account.
- User bisa switch active account seperti Gmail.
- Inbox fetch berdasarkan active account.
- Receive email via SMTP port 25.
- Realtime inbox update via SSE.
- Message detail, read status, starred, archive, soft delete.
- Attachment metadata + object storage.
- PostgreSQL/Neon sebagai source of truth.
- Redis/Dragonfly sebagai queue + pub/sub, bukan primary storage.

Dokumen ini menggantikan flow temporary-mail lama. Semua prompt lanjutan harus mengikuti flow baru ini.

## 2. Architecture Decision

### Style

Tetap pakai **Microservices + Event-Driven Architecture**.

Core idea:

```text
SMTP Receiver → Redis Streams → Email Processor → PostgreSQL
                                      ↓
                                Redis Pub/Sub
                                      ↓
                                  API SSE
                                      ↓
                                  Frontend
```

### Source of Truth

- PostgreSQL/Neon = source of truth untuk accounts, sessions, messages, attachments, API keys.
- Redis/Dragonfly = queue (`Redis Streams`) + realtime pub/sub (`email.received`).
- R2/S3 = attachment object storage.
- Frontend local state/cache tidak boleh menjadi source of truth.

### Services

| Service | Role | Notes |
|---|---|---|
| `smtp-receiver` | Receive raw SMTP email | Fast path only. No parsing. Push raw email to Redis Streams. |
| `email-processor` | Consume stream, parse email, persist to DB | Resolve recipient account from DB, store message/attachments, publish event. |
| `api-server` | Auth, account/session, message API, SSE | Main business API. Uses Postgres + Redis Pub/Sub. |
| `cleanup-worker` | Periodic cleanup | Expired sessions/API keys, old soft-deleted messages, orphan attachments. |
| `frontend` | TanStack Start UI | SSR/BFF-friendly. Account switcher, inbox, message detail. |

## 3. Non-Negotiable Engineering Rules

1. Clean code first. No handler bloat.
2. Manual DI only. No DI framework.
3. Constructor injection + Axum `State`.
4. Raw SQL/manual query allowed and preferred for this project phase.
5. UUIDv7 generated from Rust app, not DB default.
6. No hardcoded secrets.
7. `api-server` business logic must live in service layer.
8. SQL must live in repository layer.
9. `smtp-receiver` must not parse email body.
10. `email-processor` owns parsing and persistence.
11. Redis queue processing must stay at-least-once.
12. Frontend must not call private internal service URLs directly in production; use same-origin `/api` through nginx/BFF.

## 4. Data Model

Schema source:

- `docs/schema.sql`

Main tables:

- `accounts`
- `sessions`
- `session_accounts`
- `api_keys`
- `messages`
- `attachments`

### UUIDv7 Rule

Every table primary key uses:

```sql
id UUID PRIMARY KEY
```

Rust app must generate UUIDv7 before insert.

No `gen_random_uuid()` default for app-created rows.

### Session Meaning

`sessions` = one browser/device session.

Example:

```text
Browser A login account1 + account2
→ one sessions row
→ two session_accounts rows
→ one row has is_active = true

Browser B login account1
→ another sessions row
```

### API Key Meaning

`api_keys` is global gatekeeper for account creation.

It is not linked to `accounts` for now.

Use case:

```text
Client wants create account
→ must send valid API key
→ api-server checks api_keys
→ account created if username/email not used
```

## 5. Rust Architecture Pattern

### Module Layout per Complex Service

For `api-server` and `email-processor`, use this pattern:

```text
src/
  main.rs
  config.rs
  state.rs
  routes.rs
  handlers/
  services/
  repositories/
  models/
  dto/
```

Keep smaller service layout simple if no benefit from over-splitting.

### Dependency Direction

```text
handlers → services → repositories → database
```

Handlers must not contain SQL.
Repositories must not contain HTTP logic.
Services contain business rules.

### Manual DI Shape

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisClient,
    pub auth_service: AuthService,
    pub account_service: AccountService,
    pub message_service: MessageService,
}
```

Services receive dependencies via constructors:

```rust
impl AccountService {
    pub fn new(repo: AccountRepository, api_keys: ApiKeyRepository, password: PasswordService) -> Self {
        Self { repo, api_keys, password }
    }
}
```

## 6. Tech Stack

### Backend

- Rust Edition 2021 or newer workspace setting.
- Tokio.
- Axum.
- Tower middleware.
- tracing + tracing-subscriber.
- dotenvy/environment config.
- PostgreSQL/Neon.
- Raw SQL via `tokio-postgres` + pool, or `sqlx` without macros if preferred.
- Redis/Dragonfly via `redis-rs` async.
- `mail-parser` for email parsing.
- AWS SDK/S3-compatible client for R2 attachment storage.
- Password hashing: Argon2id preferred.
- Token hashing: SHA-256/HMAC-SHA256 preferred for session/API key lookup.
- UUIDv7 crate for IDs.

### Frontend

- TanStack Start.
- TanStack Query.
- Bun package manager/runtime.
- Same-origin API path `/api` in production.
- SSE via `EventSource`.
- Modal/dialog for message detail.

## 7. API Design Target

### Account Creation

```text
POST /accounts
Headers:
  X-API-Key: <raw-api-key>
Body:
  username
  local_part
  domain? optional/server default
  password
  display_name?
```

Rules:

- API key required.
- API key hash must match active, non-expired `api_keys` row.
- Username must be unique.
- `(local_part, domain)` must be unique.
- Password stored as hash only.
- ID generated as UUIDv7 in app.

### Login / Session

```text
POST /sessions/login
Body:
  username_or_email
  password
```

Rules:

- Create session if no valid session cookie exists.
- Reuse session if valid session cookie exists.
- Attach account to session via `session_accounts`.
- If no active account in session, set login account active.
- Return safe session/account data.
- Set secure HTTP-only cookie in production.

### Session Account Switch

```text
GET /session/accounts
POST /session/accounts/:account_id/activate
```

Rules:

- Only accounts attached to current session can be activated.
- Use transaction:
  - set all `is_active=false` for session
  - set chosen account `is_active=true`
- DB partial unique index enforces one active account per session.

### Messages

```text
GET /messages
GET /messages/:message_id
PATCH /messages/:message_id
```

`GET /messages` reads active account from session.

Default inbox filter:

```sql
WHERE account_id = $1
  AND is_deleted = false
  AND is_archived = false
ORDER BY received_at DESC
```

Patch supports:

- `is_read`
- `is_starred`
- `is_archived`
- `is_deleted`

### SSE

```text
GET /messages/events
```

Rules:

- Auth required.
- Stream events only for current active account/session.
- Redis Pub/Sub event must include `account_id` and `message_id`.
- API filters event by account/session before sending to browser.

## 8. Service Responsibilities

### smtp-receiver

Keep fast and dumb.

Responsibilities:

- Listen SMTP port 25.
- Validate domain accepted by config.
- Enforce max message size.
- Deny open relay.
- Optional STARTTLS.
- Push raw email to Redis Streams.

Not responsibilities:

- No MIME parsing.
- No DB message insert.
- No attachment upload.

Future improvement:

- Recipient validation through cached account address set in Redis to reject unknown users at SMTP RCPT stage.
- Do this later, not phase 1.

### email-processor

Responsibilities:

- Consume Redis Streams with consumer group.
- Parse email with `mail-parser`.
- Extract from/to/cc/bcc/subject/text/html/message-id/attachments.
- Find recipient account by `accounts.address`.
- Insert `messages` into PostgreSQL.
- Upload attachments to R2 if configured.
- Insert `attachments` metadata.
- Publish `email.received` event.
- Ack stream message only after DB/object storage success.

Unknown recipient rule:

- Log and ack/drop for now, since SMTP may already accept message.
- Later: SMTP recipient validation avoids this.

### api-server

Responsibilities:

- Auth/session/account API.
- Message query/update API.
- SSE endpoint.
- API key validation for account creation.
- Manual DI orchestration.
- Transaction boundaries for session switch and account creation where needed.

### cleanup-worker

Responsibilities:

- Delete expired sessions.
- Disable/delete expired API keys if desired.
- Purge soft-deleted messages older than configured retention.
- Delete orphan/expired object storage attachments.

Not now:

- No temp mailbox TTL deletion as core flow.

## 9. Frontend Flow

### Initial Load

```text
Open app
→ SSR/BFF/API checks session cookie
→ fetch session accounts
→ select active account
→ fetch messages for active account
```

### Register Account

```text
User fills username/email/password + API key
→ POST /api/accounts
→ success: account created
→ user can login
```

### Login Multiple Accounts

```text
User login another account in same browser
→ POST /api/sessions/login
→ backend attaches account to existing session
→ account switcher shows both accounts
```

### Switch Account

```text
User chooses account
→ POST /api/session/accounts/:id/activate
→ invalidate messages query
→ fetch messages for active account
```

### Inbox

- List messages for active account.
- Click message opens dialog/detail view.
- Star/archive/read/delete actions update backend then invalidate query.
- SSE invalidates messages query on `email.received`.

## 10. Environment Variables

### Global

```env
RUST_LOG=info
APP_ENV=production
```

### PostgreSQL/Neon

```env
DATABASE_URL=postgresql://...
```

### Redis/Dragonfly

```env
REDIS_URL=redis://dragonfly:6379
QUEUE_RAW_EMAIL_STREAM=email_raw
REDIS_CONSUMER_GROUP=email-processor
REDIS_CONSUMER_NAME=email-processor-1
REDIS_STREAM_BLOCK_MS=5000
```

### Mail Domain

```env
MAILBOX_DOMAIN=intotheheap.net
```

### API Server

```env
API_BIND_ADDR=0.0.0.0:8080
CORS_ALLOWED_ORIGINS=https://tempmail.intotheheap.net
SESSION_COOKIE_NAME=tempmail_session
SESSION_TTL_SECONDS=2592000
SESSION_COOKIE_SECURE=true
```

### SMTP Receiver

```env
SMTP_LISTEN_ADDR=0.0.0.0:25
SMTP_HOSTNAME=mail.intotheheap.net
SMTP_MAX_MESSAGE_BYTES=10485760
SMTP_MAX_MESSAGES_PER_CONNECTION=20
SMTP_ENABLE_STARTTLS=false
TLS_CERT_PATH=
TLS_KEY_PATH=
```

### Object Storage

```env
R2_ENDPOINT=
R2_BUCKET=
R2_ACCESS_KEY_ID=
R2_SECRET_ACCESS_KEY=
R2_REGION=auto
R2_PREFIX=attachments/
```

### Cleanup

```env
CLEANUP_INTERVAL_SECONDS=60
CLEANUP_BATCH_SIZE=100
SOFT_DELETE_RETENTION_DAYS=30
```

### Frontend

Local:

```env
VITE_API_BASE_URL=http://localhost:8080
```

Production build:

```env
VITE_API_BASE_URL=/api
```

## 11. Implementation Roadmap

### Phase 0 — Planning & DB Schema

- Update `docs/schema.sql`.
- Use UUIDv7 IDs from Rust app.
- Decide DB pool crate.
- Add `DATABASE_URL` env docs.

Done when:

- Schema supports accounts/sessions/messages/API keys.
- Planning doc matches new persistent flow.

### Phase 1 — Shared Foundation

Tasks:

1. Add Postgres config to `crates/shared`.
2. Add UUIDv7 helper.
3. Add password hashing helper.
4. Add token generation/hash helper.
5. Add shared DB/domain models.
6. Add common auth/session error types.

Done when:

- Workspace compiles.
- Helpers unit-testable.
- No DB logic leaked to handlers.

### Phase 2 — api-server DI Refactor

Tasks:

1. Split `api-server` into handlers/services/repositories/models/dto.
2. Add `PgPool` to `AppState`.
3. Wire repositories/services in `main.rs`.
4. Keep Redis Pub/Sub/SSE dependency explicit.
5. Add health endpoint checking app + optional DB ping.

Done when:

- Existing health and routes still compile.
- `AppState` becomes explicit dependency container.
- No SQL in handlers.

### Phase 3 — API Key Gate + Account Creation

Tasks:

1. Implement `ApiKeyRepository`.
2. Implement API key hash lookup.
3. Implement `AccountRepository`.
4. Implement `AccountService::create_account`.
5. Endpoint `POST /accounts`.
6. Duplicate username/email handling.
7. Password hashing with Argon2id.

Done when:

- Account cannot be created without valid API key.
- Duplicate username/email returns clean error.
- Account IDs are UUIDv7.

### Phase 4 — Sessions + Multi-Account Login

Tasks:

1. Implement `SessionRepository`.
2. Implement login with password verify.
3. Create/reuse browser session.
4. Attach account to session.
5. Set active account if none active.
6. HTTP-only session cookie.
7. `GET /session/accounts`.
8. `POST /session/accounts/:account_id/activate`.

Done when:

- One browser can hold multiple logged-in accounts.
- Only one active account per session.
- Switch account changes message source.

### Phase 5 — email-processor PostgreSQL Persistence

Tasks:

1. Add DB pool/config to `email-processor`.
2. Add account lookup by email address.
3. Replace Redis message storage with DB insert.
4. Keep R2 upload for attachments.
5. Insert attachment metadata in DB.
6. Publish `email.received` with account/message IDs.
7. Ack queue only after DB/storage success.

Done when:

- Incoming email stored in PostgreSQL.
- Unknown recipient handled safely.
- SSE event still emitted.

### Phase 6 — Message API

Tasks:

1. Implement `MessageRepository`.
2. `GET /messages` for active account.
3. `GET /messages/:id` detail for active account only.
4. `PATCH /messages/:id` flags.
5. Enforce account ownership on every message access.

Done when:

- Inbox reads from PostgreSQL.
- Message detail works.
- Star/archive/read/delete work.

### Phase 7 — SSE Auth & Account Filtering

Tasks:

1. Make SSE endpoint auth-aware.
2. Subscribe to Redis Pub/Sub.
3. Filter events by active account/session.
4. Frontend invalidates message query on event.

Done when:

- User only receives events for active account.
- Switch account updates event filtering.

### Phase 8 — Frontend Auth + Account Switcher

Tasks:

1. Add register page/form with API key.
2. Add login form.
3. Add session query.
4. Add account switcher.
5. Update inbox query to `/api/messages`.
6. Add message actions: star/archive/read/delete.
7. Keep detail dialog.
8. Handle expired session cleanly.

Done when:

- User can create account, login, switch accounts, view messages.
- UI no longer depends on generate-temp-mail flow.

### Phase 9 — Cleanup Worker Refactor

Tasks:

1. Add DB pool.
2. Delete expired sessions.
3. Handle expired API keys if configured.
4. Purge soft-deleted messages older than retention.
5. Delete R2 objects for purged attachments.

Done when:

- Permanent mailbox data remains unless soft-deleted/purged.
- No temp TTL cleanup used for main messages.

### Phase 10 — Hardening

Tasks:

1. Rate limit account creation/login.
2. Rate limit SMTP by IP/domain.
3. Add request tracing IDs.
4. Sanitize HTML email body before render, or render as text only.
5. Add secure cookie config.
6. Add DB transaction tests for session switch.
7. Add indexes based on real query patterns.
8. Add deployment env docs.

Done when:

- Auth/session safe enough for public deploy.
- No open relay.
- No cross-account data leak.

## 12. Deployment Notes

Current production target:

- GHCR images.
- Portainer stack.
- Dragonfly in same stack network.
- Nginx on VPS host.
- `tempmail.intotheheap.net` proxies frontend and `/api`.
- `mail.intotheheap.net` DNS-only A record points to VPS.
- MX `intotheheap.net` points to `mail.intotheheap.net`.

Nginx pattern:

```text
/      → 127.0.0.1:3000
/api/  → 127.0.0.1:8080/  (strip /api prefix)
```

SMTP:

```text
public :25 → smtp-receiver container :25
```

## 13. Definition of Done

Per phase:

- `cargo check --workspace` passes.
- Relevant service runs independently.
- Config via env.
- No secrets in repo.
- SQL only in repositories.
- Handler remains thin.
- Errors are typed and mapped to proper HTTP responses.
- Logs use tracing.
- No plaintext passwords/tokens stored.
- Session/account ownership enforced.

## 14. Coding Agent Instruction

Use this document as single source of truth.

If user asks for code change, preserve these decisions:

- Persistent Postgres-first mailbox.
- Redis only queue/pubsub.
- Manual DI.
- UUIDv7 from Rust app.
- API key required for account creation.
- Multi-account browser session via `sessions` + `session_accounts`.
- SSE, not WebSocket.
- Clean architecture: handlers → services → repositories.

If request conflicts with this doc, ask before coding.
