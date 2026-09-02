# Project: TempMail - Disposable Email Service (Rust)

## 1. Goal
Membangun web application Temporary Email (Disposable Email) dari scratch menggunakan Rust, dengan kemampuan:
- Generate temporary email address
- Menerima email via SMTP (port 25)
- Menampilkan inbox secara realtime
- Auto-delete email setelah TTL tertentu
- Arsitektur advanced, scalable, dan maintainable

## 2. Architecture Decision

### Style
**Microservices + Event-Driven Architecture** (bukan monolith, bukan pure REST antar service)

### Services yang harus dibuat (4 service + frontend)

| Service              | Tanggung Jawab Utama                                                                 | Port / Role          |
|----------------------|---------------------------------------------------------------------------------------|----------------------|
| `smtp-receiver`      | Menerima email mentah via SMTP, validasi dasar, push ke Message Queue                | 25 (SMTP)           |
| `email-processor`    | Consume queue → parse email → simpan ke storage → publish event                      | Worker              |
| `api-server`         | REST API + WebSocket untuk frontend (generate email, list inbox, realtime update)   | 8080 (HTTP)         |
| `cleanup-worker`     | Membersihkan data expired (Redis, Postgres, S3)                                      | Background worker   |
| `frontend`           | UI pengguna                                                                          | -                   |

### Communication Pattern (Sangat Penting)
- **Jangan** membuat service saling call via REST/gRPC sebagai jalur utama.
- Gunakan **Event-Driven**:
  - `smtp-receiver` → **Message Queue** → `email-processor`
  - `email-processor` → **Pub/Sub** → `api-server` (untuk realtime)
- Shared Storage (Redis + PostgreSQL + Object Storage) diakses langsung oleh service yang membutuhkan.
- REST/gRPC hanya boleh digunakan untuk keperluan admin/internal yang jarang.

**Queue Recommendation (urut prioritas):**
1. Redis Streams (paling mudah di awal)
2. NATS
3. Kafka (jika volume sangat besar)

### Dependency Injection
- **Tidak menggunakan** DI framework (shaku, injector, dll).
- Gunakan **Manual Constructor Injection** + **Axum State**.
- Semua dependency di-wire secara explicit di `main.rs`.

## 3. Monorepo Structure (Cargo Workspace)

Gunakan **monorepo** dengan struktur berikut:
tempmail/
├── Cargo.toml                          # Workspace root
├── crates/
│   ├── shared/                         # Shared library (models, queue message, events, config, error)
│   ├── smtp-receiver/
│   ├── email-processor/
│   ├── api-server/
│   └── cleanup-worker/
├── frontend/
├── deploy/
│   ├── docker/
│   └── docker-compose.yml
├── scripts/
└── docs/

### Shared Crate (`crates/shared`) harus berisi:
- Models (Email, Envelope, Mailbox, Attachment, dll)
- Queue message format
- Event definitions (EmailReceived, dll)
- Config loading
- Common error types
- Redis helper

## 4. Tech Stack (Wajib diikuti)

- **Language**: Rust (Edition 2021)
- **Async Runtime**: Tokio
- **Web Framework**: Axum + Tower Custom Middleware
- **Email Parsing**: `mail-parser`
- **SMTP Server**: Custom state machine atau library minimal (`minismtp` / `smtpd` / `rs-smtp`) — prioritaskan kontrol penuh
- **Serialization**: serde + serde_json
- **Redis**: redis-rs (tokio)
- **Database**: PostgreSQL via sqlx
- **Object Storage**: Cloudflare R2
- **Logging**: tracing + tracing-subscriber
- **Config**: dotenvy + config crate atau pure environment variable

## 5. Core Design Rules (Best Practices)

### smtp-receiver
- Harus sangat cepat dan ringan.
- Hanya melakukan validasi dasar (domain, size limit, rate limit per IP).
- **Jangan** parse email di sini.
- Langsung push raw data ke queue setelah DATA selesai.
- Support STARTTLS.
- Tolak open relay.

### email-processor
- Consume dari queue secara reliable (at-least-once).
- Parse menggunakan `mail-parser`.
- Simpan:
  - Metadata + body ke Redis (dengan TTL)
  - Attachment ke Object Storage
  - Optional: metadata ke PostgreSQL
- Setelah berhasil menyimpan → publish event `email.received` ke Pub/Sub.

### api-server
- Stateless.
- Baca data utama dari Redis.
- Sediakan WebSocket untuk realtime inbox update.
- Endpoint minimal:
  - Generate mailbox
  - List messages
  - Get message detail
  - Delete mailbox / message

### cleanup-worker
- Berjalan periodik.
- Menghapus data yang sudah melewati TTL di Redis, Postgres, dan Object Storage.

### Security & Reliability
- Setiap inbox punya TTL (contoh: 1 jam default, configurable).
- Rate limiting ketat di SMTP dan API.
- Sanitize HTML body sebelum dikirim ke frontend.
- Domain harus dikonfigurasi dengan SPF, DKIM, dan DMARC.
- Jangan pernah menjadi open relay.
- Minimal logging data sensitif.

## 6. Development Guidelines untuk Coding Agent

1. Selalu buat perubahan di dalam Cargo Workspace yang sudah ditentukan.
2. Kode yang bisa dipakai lebih dari 1 service **harus** diletakkan di `crates/shared`.
3. Setiap service harus punya `config.rs` dan di-load di `main.rs`.
4. Gunakan `thiserror` + `anyhow` untuk error handling.
5. Semua async code harus menggunakan Tokio.
6. Jangan hardcode credential — selalu dari environment variable.
7. Tulis kode yang explicit dan mudah dibaca (hindari magic yang berlebihan).
8. Setiap service harus bisa dijalankan secara independen (`cargo run -p <service-name>`).
9. Prioritaskan correctness dan clarity daripada premature optimization.
10. Saat menambahkan fitur baru, selalu pertimbangkan dampaknya terhadap arsitektur event-driven.

## 7. Urutan Implementasi yang Disarankan

1. Setup Cargo Workspace + `shared` crate (models + queue message)
2. `smtp-receiver` (bisa terima email dan push ke Redis Streams)
3. `email-processor` (consume → parse → simpan ke Redis)
4. `api-server` (generate email + list messages)
5. WebSocket realtime
6. `cleanup-worker`
7. Frontend (Tanstack Start + Tanstack Query)
8. Hardening (rate limit, STARTTLS, monitoring, dll)

## 8. Definition of Done (per service)
- Bisa di-compile tanpa error
- Bisa dijalankan secara independen
- Config via environment variable
- Logging menggunakan tracing
- Error handling yang jelas
- Tidak melanggar communication pattern (event-driven)

---

**Instruksi untuk Coding Agent:**
Gunakan dokumen ini sebagai single source of truth. Setiap kali akan membuat atau mengubah kode, pastikan keputusan arsitektur di atas tetap dijaga. Jika ada konflik antara permintaan user dengan dokumen ini, tanyakan dulu sebelum melanjutkan.