import type { Message } from '../lib/api/types'

type EmailDetailDialogProps = {
  message: Message | null
  onClose: () => void
}

export function EmailDetailDialog({ message, onClose }: EmailDetailDialogProps) {
  if (!message) {
    return null
  }

  const body = message.text_body || message.html_body || 'Email ini tidak memiliki konten teks.'

  return (
    <div className="dialog-backdrop" role="presentation" onClick={onClose}>
      <section
        aria-labelledby="email-detail-title"
        className="dialog-panel"
        role="dialog"
        aria-modal="true"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="dialog-header">
          <div>
            <p className="eyebrow">Email detail</p>
            <h2 id="email-detail-title">{message.subject || '(Tanpa subject)'}</h2>
          </div>
          <button type="button" className="ghost-button" onClick={onClose}>
            Tutup
          </button>
        </div>

        <dl className="email-meta">
          <div>
            <dt>Dari</dt>
            <dd>{formatSender(message)}</dd>
          </div>
          <div>
            <dt>Ke</dt>
            <dd>{formatAddresses(message.to_addresses)}</dd>
          </div>
          <div>
            <dt>CC</dt>
            <dd>{formatAddresses(message.cc_addresses)}</dd>
          </div>
          <div>
            <dt>Diterima</dt>
            <dd>{formatDate(message.received_at)}</dd>
          </div>
        </dl>

        <div className="email-body">
          <h3>Konten</h3>
          <pre>{body}</pre>
        </div>

        {message.has_attachments && (
          <div className="attachments">
            <h3>Attachments</h3>
            <p className="muted">Attachment metadata tersimpan di backend.</p>
          </div>
        )}
      </section>
    </div>
  )
}

function formatSender(message: Message) {
  if (!message.from_name) {
    return message.from_address
  }

  return `${message.from_name} <${message.from_address}>`
}

function formatAddresses(addresses: Message['to_addresses'] | Message['cc_addresses']) {
  if (!addresses || addresses.length === 0) {
    return '-'
  }

  return addresses.map((entry) => (entry.name ? `${entry.name} <${entry.address}>` : entry.address)).join(', ')
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat('id-ID', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value))
}
