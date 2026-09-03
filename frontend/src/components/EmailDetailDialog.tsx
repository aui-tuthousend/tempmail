import type { EmailMessage } from '../lib/api/types'

type EmailDetailDialogProps = {
  message: EmailMessage | null
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
            <dd>{message.from || 'Unknown sender'}</dd>
          </div>
          <div>
            <dt>Ke</dt>
            <dd>{message.to.length > 0 ? message.to.join(', ') : '-'}</dd>
          </div>
          <div>
            <dt>Diterima</dt>
            <dd>{formatDate(message.received_at)}</dd>
          </div>
          <div>
            <dt>Kedaluwarsa</dt>
            <dd>{formatDate(message.expires_at)}</dd>
          </div>
        </dl>

        <div className="email-body">
          <h3>Konten</h3>
          <pre>{body}</pre>
        </div>

        {message.attachments.length > 0 && (
          <div className="attachments">
            <h3>Attachments</h3>
            <ul>
              {message.attachments.map((attachment) => (
                <li key={attachment.id}>
                  <span>{attachment.filename || 'Unnamed attachment'}</span>
                  <small>
                    {attachment.content_type} · {formatBytes(attachment.size_bytes)}
                  </small>
                </li>
              ))}
            </ul>
          </div>
        )}
      </section>
    </div>
  )
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat('id-ID', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value))
}

function formatBytes(value: number) {
  if (value < 1024) {
    return `${value} B`
  }

  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`
  }

  return `${(value / (1024 * 1024)).toFixed(1)} MB`
}
