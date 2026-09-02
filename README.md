# TempMail

TempMail adalah layanan disposable email berbasis Rust, Redis Streams, dan TanStack Start. Sistem ini menerima email via SMTP, memproses raw email secara async, menyimpan inbox sementara di Redis, dan mengirim update realtime ke frontend via SSE.

## Komponen

- `smtp-receiver`: menerima SMTP, validasi dasar, lalu push raw email ke Redis Streams.
- `email-processor`: consume Redis Streams, parse email, simpan pesan ke Redis TTL, upload attachment opsional ke R2, lalu publish event `email.received`.
- `api-server`: menyediakan endpoint generate mailbox, list messages, dan SSE events.
- `cleanup-worker`: menghapus pesan/attachment yang expired.
- `frontend`: UI inbox dengan TanStack Start, TanStack Query, dan Bun.
- `dragonfly`: Redis-compatible storage/queue via Docker Compose.

## Prasyarat

- Rust stable
- Bun
- Docker + Docker Compose

## Setup

Salin konfigurasi environment:

```bash
cp .env.example .env
```

Minimal konfigurasi yang perlu diisi:

```env
MAILBOX_DOMAIN=example.com
REDIS_URL=redis://127.0.0.1:6379
API_BIND_ADDR=0.0.0.0:8080
SMTP_LISTEN_ADDR=0.0.0.0:2525
```

Install dependency frontend:

```bash
cd frontend
bun install
```

## Menjalankan development

Dari root project:

```bash
make dev
```

Command ini akan:

1. Menjalankan Dragonfly/Redis dengan Docker Compose.
2. Menjalankan `api-server`.
3. Menjalankan `smtp-receiver`.
4. Menjalankan `email-processor`.
5. Menjalankan `cleanup-worker`.
6. Menjalankan frontend dengan Bun.

Untuk menghentikan semua process development, tekan `Ctrl+C`.

## Command tambahan

```bash
make redis-up
make redis-down
make api
make smtp
make processor
make cleanup
make frontend
make check
```

## Endpoint utama

- `GET /health`
- `POST /mailboxes`
- `GET /mailboxes/:mailbox/messages`
- `GET /mailboxes/:mailbox/events`

Default API URL frontend:

```env
VITE_API_BASE_URL=/api
```

## Deployment Portainer

File [docker-compose.prod.yml](docker-compose.prod.yml) berisi satu stack untuk:

- DragonflyDB
- `api-server`
- `smtp-receiver`
- `email-processor`
- `cleanup-worker`
- `frontend`

App image dipublish ke GitHub Container Registry (GHCR):

- `ghcr.io/aui-tuthousend/tempmail-api-server:latest`
- `ghcr.io/aui-tuthousend/tempmail-smtp-receiver:latest`
- `ghcr.io/aui-tuthousend/tempmail-email-processor:latest`
- `ghcr.io/aui-tuthousend/tempmail-cleanup-worker:latest`
- `ghcr.io/aui-tuthousend/tempmail-frontend:latest`

Workflow [publish-images.yml](.github/workflows/publish-images.yml) otomatis build dan push image ke GHCR saat push ke `main`/`master`, atau bisa dijalankan manual dari tab Actions.

Di Portainer, buat Stack baru lalu gunakan isi `docker-compose.prod.yml`. Semua service memakai network internal yang sama, sehingga app services mengakses Dragonfly dengan:

```env
REDIS_URL=redis://dragonfly:6379
```

Frontend production memakai `VITE_API_BASE_URL=/api`, jadi nginx perlu proxy path `/api` ke host `127.0.0.1:8080` dan proxy domain frontend ke host `127.0.0.1:3000`.

## DNS untuk domain email

Untuk menerima email publik ke domain sendiri, siapkan DNS:

- `A mail -> IP_VPS`
- `MX @ -> mail.intotheheap.net` priority `10`
- `A tempmail -> IP_VPS`

Untuk Cloudflare, record `mail` sebaiknya **DNS only** / tidak di-proxy.

Di VPS, pastikan port SMTP publik `25` terbuka dan mengarah ke `smtp-receiver`. Pada stack production, service `smtp-receiver` listen langsung di port `25`.

## CI

GitHub Actions menjalankan:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bun install --frozen-lockfile`
- `bun run typecheck`
- `bun run build`
