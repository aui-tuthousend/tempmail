import { Inbox } from './components/Inbox'
import { MailboxCard } from './components/MailboxCard'
import { useInboxEvents } from './lib/hooks/useInboxEvents'
import { useCreateMailbox } from './lib/hooks/useMailbox'
import { useMessages } from './lib/hooks/useMessages'
import { useStoredMailbox } from './lib/hooks/useStoredMailbox'

export function App() {
  const { mailbox, setMailbox } = useStoredMailbox()
  const createMailbox = useCreateMailbox()
  const address = mailbox?.address ?? null
  const messages = useMessages(address)

  useInboxEvents(address)

  const handleGenerate = () => {
    createMailbox.mutate(undefined, {
      onSuccess: setMailbox,
    })
  }

  return (
    <main className="page-shell">
      <section className="hero">
        <p className="eyebrow">TempMail</p>
        <h1>Disposable email inbox</h1>
        <p>
          Generate alamat sementara, terima email via SMTP, lalu lihat inbox terupdate otomatis lewat SSE.
        </p>
      </section>

      {mailbox ? (
        <MailboxCard
          mailbox={mailbox.mailbox}
          address={mailbox.address}
          isGenerating={createMailbox.isPending}
          onGenerate={handleGenerate}
        />
      ) : (
        <section className="card start-card">
          <div>
            <p className="eyebrow">Mulai</p>
            <h2>Buat temporary mailbox baru</h2>
            <p className="muted">Mailbox akan aktif sesuai TTL yang dikonfigurasi di API server.</p>
          </div>
          <button type="button" onClick={handleGenerate} disabled={createMailbox.isPending}>
            {createMailbox.isPending ? 'Generating...' : 'Generate mailbox'}
          </button>
        </section>
      )}

      {createMailbox.isError && (
        <section className="card error-state">Gagal membuat mailbox. Pastikan api-server berjalan.</section>
      )}

      <Inbox messages={messages.data?.messages ?? []} isLoading={messages.isLoading} />
    </main>
  )
}
