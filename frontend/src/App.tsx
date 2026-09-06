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
    <main className="mx-auto grid max-w-5xl gap-5 px-4 py-12">
      <section className="py-4">
        <p className="mb-3 text-xs font-extrabold uppercase tracking-widest text-indigo-600">TempMail</p>
        <h1 className="max-w-4xl text-5xl font-black leading-none tracking-[-0.06em] text-slate-950 md:text-7xl">
          Persistent email inbox
        </h1>
        <p className="mt-5 max-w-2xl text-base leading-7 text-slate-500">
          Login ke account email permanen, switch antar-account dalam satu browser, dan terima inbox realtime via SSE.
        </p>
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
        <section className="rounded-3xl border border-orange-200 bg-orange-50 p-5 text-center font-medium text-orange-700">
          Session belum ada atau sudah expired. Silakan login kembali.
        </section>
      )}

      <AccountSwitcher
        accounts={accounts}
        isSwitching={activateAccount.isPending}
        onActivate={(accountId) => activateAccount.mutate(accountId)}
      />

      {activateAccount.isError && (
        <section className="rounded-3xl border border-orange-200 bg-orange-50 p-5 text-center font-medium text-orange-700">
          Gagal switch account.
        </section>
      )}
      {messages.isError && isAuthenticated && (
        <section className="rounded-3xl border border-orange-200 bg-orange-50 p-5 text-center font-medium text-orange-700">
          Gagal memuat inbox. Session mungkin sudah expired.
        </section>
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
