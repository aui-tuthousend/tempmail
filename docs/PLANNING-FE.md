# Frontend UI Planning — TempMail

Dokumen ini jadi single source of truth untuk pembangunan UI frontend (halaman, layout, routing, interaksi, komponen). Backend API tetap mengacu ke `docs/PLANNING.md` dan endpoint yang sudah ada.

## 1. Tech & Library

- TanStack Start + TanStack Router (file-based routing).
- TanStack Query (data fetch + mutate).
- Bun runtime.
- **Shadcn/ui** component (Button, Input, Label, DropdownMenu, Avatar, Sidebar, Tooltip, Badge, ScrollArea, Dialog).
- **Sonner** toast (atau shadcn `Toast`/`use-toast`), posisi **top-center** (`position="top-center"`).
- Tailwind CSS (prasyarat shadcn) — migrasi penuh dari `styles/app.css` manual ke utility classes (keputusan final).
- Pakai **Bun** untuk semua instal & command.
- Ikon: `lucide-react`.

## 1b. Keputusan Implementasi (final)

- Migrasi penuh ke Tailwind + shadcn/ui + sonner.
- Backend backfill dikerjakan juga:
  1. `GET /accounts/availability` (local_part & username)
  2. `POST /sessions/logout` (logout akun dari session)
  - (opsional, V1 filter client-side) `GET /messages?view=`

## 2. Routing & Protected Route

File-based routes di `src/routes/`:

| Route | Auth | Perilaku |
|---|---|---|
| `/` (root) | public | redirect: ada session → `/mail`, tidak ada → `/login` |
| `/login` | public | bisa diakses walau sudah login (tidak dipaksa redirect) |
| `/register` | public | bisa diakses walau sudah login |
| `/mail` | protected | wajib session; belum login → redirect `/login` |

Implementasi:

- Buat `src/routes/login.tsx`, `src/routes/register.tsx`, `src/routes/mail.tsx`.
- Buat layout group `src/routes/_app.tsx` (atau `_mail.tsx`) untuk shell /mail.
- `src/routes/index.tsx` jadi redirect only (cek session → `router.navigate`).
- Protected route `/mail` pakai `beforeLoad` / `redirect` di TanStack Router:
  - fetch session accounts (atau reuse cached query)
  - kalau error/empty → `redirect({ to: '/login' })`
  - kalau ok → continue.
- Root `/`:
  - session ada → `redirect({ to: '/mail' })`
  - else → `redirect({ to: '/login' })`

Session check memakai `useSessionAccounts()` (query `GET /session/accounts`). Karena ada `retry:false`, saat request 401 maka dianggap belum login.

## 3. Layout Shell `/mail` (beranda)

Struktur 3 zona:

### Header (top bar)

Kiri → tengah → kanan:

1. **Kiri**: nama domain (statis dari `VITE_MAILBOX_DOMAIN`), tampil sebagai brand/badge.
2. **Tengah**: search inbox (input full-text search). V1 filter client-side terhadap `subject`/`from_address`/`text_body`; bisa dioptimalkan jadi query param `?q=` setelah ada endpoint search backend.
3. **Kanan**: ikon profil / avatar account aktif → `DropdownMenu` berisi:
   - daftar akun lain yang sudah login (selain aktif): klik → switch account (`POST /session/accounts/:id/activate`)
   - opsi "Akun lain" (tambah akun) → arahkan `/login` (tetap di session yang sama)
   - "Logout akun ini" → (endpoint logout belum ada; V1 clear cookie / hapus account dari session — catat sebagai backfill)

### Sidebar (kiri)

Menu statis, satu depth:

- Inbox (`/mail` default)
- Starred
- Archived
- Deleted

Inbox filter berasal dari backend `GET /messages` (default `is_deleted=false AND is_archived=false`). Starred/Archived/Deleted:
- V1 filter client-side dari data messages berdasarkan flag `is_starred` / `is_archived` / `is_deleted`.
- Backfill: query param `?view=starred|archived|deleted` atau endpoint terpisah di `GET /messages`.

Sidebar pakai shadcn `Sidebar` (atau `Sheet`+`aside`) dengan `NavLink` active state berdasarkan route/segment `/mail`, `/mail/starred`, dst.

### Main content

- Toolbar: filter/aksi massal (opsional), tombol refresh.
- List pesan (reuse `Inbox` yang sudah ada, adaptasi ke shadcn `ScrollArea` + `Card`/`List`).
- Klik pesan → `EmailDetailDialog` (shadcn `Dialog`) — pertahankan yang sudah ada.

## 4. Halaman Login `/login`

- Field: local part email (input kiri) + domain disabled (kanan), password.
- Submit → `useLogin()` → sukses → toast → `navigate('/mail')`.
- Error → toast error / inline.
- Link ke `/register`.

## 5. Halaman Register `/register`

Field:
- API key
- Username
- Local part email (input kiri) + domain disabled (kanan)
- Display name (opsional)
- Password (dengan validasi requirement)

### Cek ketersediaan local part (debounce)

- Saat local part diisi, **debounce ~400–500ms** panggil endpoint availability.
- Belum ada endpoint backend → backfill `GET /accounts/availability?local_part=...` (juga bisa `?username=`).
- Kalau sudah terdaftar → peringatan inline + **disable** tombol submit.
- Kalau belum / loading → enable submit (dengan syarat password valid).

### Password requirement

Validasi client-side:
- minimal 8 karakter
- minimal 1 huruf kapital (A–Z)

Tampilkan indikator/help text (`is valid` vs `belum memenuhi`) dan disable submit bila tidak valid.

### Submit

- `useCreateAccount()` → sukses → toast + redirect `/login`.
- Error → toast.

## 6. Notifikasi (Toast)

- Pakai Sonner `<Toaster position="top-center" richColors />` di `__root.tsx`.
- Semua aksi: login sukses/gagal, register sukses/gagal, switch account, logout, update pesan (star/archive/read/delete).

## 7. State & Hooks

- `useSessionAccounts()` — list akun + status auth (sudah ada).
- `useLogin()`, `useCreateAccount()`, `useActivateSessionAccount()` (sudah ada).
- `useMessages(view)` — fetch pesan aktif (sudah ada `useMessages`), tambah param view client filter.
- `useUpdateMessage()` (sudah ada).
- `useInboxEvents()` SSE (sudah ada).
- Tambah `useLocalPartAvailability()` — hook debounce untuk cek ketersediaan.

## 8. Backfill backend yang dibutuhkan

1. `GET /accounts/availability?local_part=...&username=...` → cek ketersediaan (tanpa API key).
2. `POST /sessions/logout` (logout akun dari session) — untuk opsi "Logout akun ini".
3. (opsional) query param pada `GET /messages` untuk view starred/archived/deleted.

Semua backfill di atas dikerjakan di api-server sebagai bagian dari task ini.

## 9. Struktur file target

```text
src/
  components/
    ui/                    # shadcn generated components
    mail/
      MailHeader.tsx
      UserMenu.tsx
      MailSidebar.tsx
      SearchInput.tsx
      MessageList.tsx      # adaptasi Inbox
      EmailDetailDialog.tsx
    auth/
      LoginForm.tsx
      RegisterForm.tsx
      PasswordStrength.tsx
      LocalPartAvailability.tsx
  routes/
    __root.tsx
    index.tsx              # redirect only
    login.tsx
    register.tsx
    _app.tsx               # shell layout /mail
    _app.mail.tsx          # /mail
    _app.mail.starred.tsx
    _app.mail.archived.tsx
    _app.mail.deleted.tsx
  lib/
    api/client.ts
    api/types.ts
    hooks/useSession.ts
    hooks/useMessages.ts
    hooks/useInboxEvents.ts
    hooks/useLocalPartAvailability.ts
    utils.ts               # cn() dari shadcn
```

## 10. Done When

- Shadcn + Tailwind aktif, Sonner terpasang di root (top-center).
- Routing: `/` redirect berdasarkan session, `/mail` protected, `/login` & `/register` publik.
- Shell `/mail` punya header (domain | search | profil dropdown) + sidebar (inbox/starred/archived/deleted).
- Profil dropdown: switch account, tambah akun, logout akun ini.
- Register: debounce cek local part + disable submit jika terdaftar, password min 8 + 1 kapital.
- Semua aksi memakai toast top-center.
- `tsc --noEmit` + `vite build` lulus.