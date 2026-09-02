import type { Mailbox } from '../lib/api/types'

type MailboxCardProps = {
  mailbox: Mailbox
  address: string
  isGenerating: boolean
  onGenerate: () => void
}

export function MailboxCard({ mailbox, address, isGenerating, onGenerate }: MailboxCardProps) {
  return (
    <section className="card mailbox-card">
      <div>
        <p className="eyebrow">Temporary address</p>
        <h1>{address}</h1>
        <p className="muted">Aktif sampai {formatDate(mailbox.expires_at)}</p>
      </div>
      <button type="button" onClick={onGenerate} disabled={isGenerating}>
        {isGenerating ? 'Generating...' : 'Generate baru'}
      </button>
    </section>
  )
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat('id-ID', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value))
}
