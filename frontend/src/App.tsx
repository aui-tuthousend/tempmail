import { AccountSwitcher } from './components/AccountSwitcher'
import { AuthPanel } from './components/AuthPanel'
import { Inbox } from './components/Inbox'
import { useInboxEvents } from './lib/hooks/useInboxEvents'
import { useMessages, useUpdateMessage } from './lib/hooks/useMessages'
import { useActivateSessionAccount, useCreateAccount, useLogin, useSessionAccounts } from './lib/hooks/useSession'

export function App() {
  const sessionAccounts = useSessionAccounts()
  const accounts = sessionAccounts.data ?? []
  const isAuthenticated = accounts.length > 0
  const messages = useMessages(isAuthenticated)
  const createAccount = useCreateAccount()
  const login = useLogin()
  const activateAccount = useActivateSessionAccount()
  const updateMessage = useUpdateMessage()

  useInboxEvents(isAuthenticated)

  const authExpired = sessionAccounts.isError && !sessionAccounts.isFetching

  return (
    <main className="page-shell">
      <section className="hero">
        <p className="eyebrow">TempMail</p>
        <h1>Persistent email inbox</h1>
        <p>Login ke account email permanen, switch antar-account dalam satu browser, dan terima inbox realtime via SSE.</p>
      </section>

      <AuthPanel
        isRegistering={createAccount.isPending}
        isLoggingIn={login.isPending}
        registerError={createAccount.error?.message ?? null}
        loginError={login.error?.message ?? null}
        onRegister={(apiKey, payload) => createAccount.mutate({ apiKey, payload })}
        onLogin={(payload) => login.mutate(payload)}
      />

      {authExpired && (
        <section className="card error-state">Session belum ada atau sudah expired. Silakan login kembali.</section>
      )}

      <AccountSwitcher
        accounts={accounts}
        isSwitching={activateAccount.isPending}
        onActivate={(accountId) => activateAccount.mutate(accountId)}
      />

      {activateAccount.isError && <section className="card error-state">Gagal switch account.</section>}
      {messages.isError && isAuthenticated && (
        <section className="card error-state">Gagal memuat inbox. Session mungkin sudah expired.</section>
      )}

      <Inbox
        messages={messages.data ?? []}
        isLoading={messages.isLoading}
        isAuthenticated={isAuthenticated}
        onToggleRead={(message) =>
          updateMessage.mutate({ messageId: message.id, payload: { is_read: !message.is_read } })
        }
        onToggleStar={(message) =>
          updateMessage.mutate({ messageId: message.id, payload: { is_starred: !message.is_starred } })
        }
        onArchive={(message) => updateMessage.mutate({ messageId: message.id, payload: { is_archived: true } })}
        onDelete={(message) => updateMessage.mutate({ messageId: message.id, payload: { is_deleted: true } })}
      />
    </main>
  )
}
