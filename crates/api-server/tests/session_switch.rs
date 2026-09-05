//! Integration test for session account switch transaction.
//!
//! Requires a live PostgreSQL database via `DATABASE_URL`. Marked `#[ignore]`
//! so `cargo test` does not fail in CI without a database; run explicitly with:
//!
//! ```text
//! DATABASE_URL=postgresql://... cargo test -p api-server --test session_switch -- --ignored
//! ```

use shared::ids::new_uuid_v7;
use sqlx::PgPool;
use uuid::Uuid;

fn database_url() -> String {
    std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for this test")
}

async fn pool() -> PgPool {
    PgPool::connect(&database_url()).await.expect("connect")
}

async fn insert_account(db: &PgPool, username: &str) -> Uuid {
    let id = new_uuid_v7();
    sqlx::query(
        r#"
        INSERT INTO accounts (id, username, local_part, domain, address, password_hash, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
    )
    .bind(id)
    .bind(username)
    .bind(username.to_lowercase())
    .bind("example.test")
    .bind(format!("{username}@example.test"))
    .bind("unused-hash")
    .execute(db)
    .await
    .expect("insert account");

    id
}

async fn insert_session(db: &PgPool) -> Uuid {
    let id = new_uuid_v7();
    sqlx::query(
        r#"
        INSERT INTO sessions (id, token_hash, expires_at)
        VALUES ($1, $2, NOW() + INTERVAL '1 day')
        "#,
    )
    .bind(id)
    .bind(format!("tok-{id}"))
    .execute(db)
    .await
    .expect("insert session");

    id
}

async fn attach_account(db: &PgPool, session_id: Uuid, account_id: Uuid) -> Uuid {
    let id = new_uuid_v7();
    sqlx::query(
        r#"
        INSERT INTO session_accounts (id, session_id, account_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(id)
    .bind(session_id)
    .bind(account_id)
    .execute(db)
    .await
    .expect("attach account");

    id
}

async fn active_account_id(db: &PgPool, session_id: Uuid) -> Option<Uuid> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT account_id FROM session_accounts WHERE session_id = $1 AND is_active = true",
    )
    .bind(session_id)
    .fetch_optional(db)
    .await
    .expect("query active account")
}

#[tokio::test]
#[ignore]
async fn switching_sets_exactly_one_active_account() {
    let db = pool().await;
    let session_id = insert_session(&db).await;

    let first = insert_account(&db, "first").await;
    let second = insert_account(&db, "second").await;

    attach_account(&db, session_id, first).await;
    attach_account(&db, session_id, second).await;

    // Seed: first is active.
    sqlx::query("UPDATE session_accounts SET is_active = false WHERE session_id = $1")
        .bind(session_id)
        .execute(&db)
        .await
        .expect("reset active");
    sqlx::query(
        "UPDATE session_accounts SET is_active = true WHERE session_id = $1 AND account_id = $2",
    )
    .bind(session_id)
    .bind(first)
    .execute(&db)
    .await
    .expect("set first active");

    // Rebuild repository and call the real transaction path.
    let repo = api_server::repositories::SessionRepository::new(db.clone());
    let activated = repo
        .activate_account(session_id, second)
        .await
        .expect("activate");

    assert!(activated, "attached account must activate");
    assert_eq!(active_account_id(&db, session_id).await, Some(second));
}

#[tokio::test]
#[ignore]
async fn activating_unattached_account_returns_false() {
    let db = pool().await;
    let session_id = insert_session(&db).await;
    let first = insert_account(&db, "owner").await;
    let outsider = insert_account(&db, "outsider").await;

    attach_account(&db, session_id, first).await;

    // Make first active so we can confirm outsider is rejected and active is preserved.
    sqlx::query(
        "UPDATE session_accounts SET is_active = true WHERE session_id = $1 AND account_id = $2",
    )
    .bind(session_id)
    .bind(first)
    .execute(&db)
    .await
    .expect("set first active");

    let repo = api_server::repositories::SessionRepository::new(db.clone());
    let activated = repo
        .activate_account(session_id, outsider)
        .await
        .expect("activate");

    assert!(!activated, "unattached account must not activate");
    assert_eq!(active_account_id(&db, session_id).await, Some(first));
}
