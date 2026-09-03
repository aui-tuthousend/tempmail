import { useState } from 'react'

import type { EmailMessage } from '../lib/api/types'
import { EmailDetailDialog } from './EmailDetailDialog'

type InboxProps = {
  messages: EmailMessage[]
  isLoading: boolean
}

export function Inbox({ messages, isLoading }: InboxProps) {
  const [selectedMessage, setSelectedMessage] = useState<EmailMessage | null>(null)

  if (isLoading) {
    return <section className="card empty-state">Memuat inbox...</section>
  }

  if (messages.length === 0) {
    return (
      <section className="card empty-state">
        <h2>Inbox kosong</h2>
        <p>Email baru akan muncul otomatis lewat SSE setelah diterima SMTP.</p>
      </section>
    )
  }

  return (
    <>
      <section className="card inbox">
        <div className="section-header">
          <p className="eyebrow">Inbox</p>
          <span>{messages.length} email</span>
        </div>
        <ul>
          {messages.map((message) => (
            <MessageItem key={message.id} message={message} onOpen={() => setSelectedMessage(message)} />
          ))}
        </ul>
      </section>

      <EmailDetailDialog message={selectedMessage} onClose={() => setSelectedMessage(null)} />
    </>
  )
}

function MessageItem({ message, onOpen }: { message: EmailMessage; onOpen: () => void }) {
  return (
    <li className="message-item">
      <button type="button" className="message-button" onClick={onOpen}>
        <div>
          <h3>{message.subject || '(Tanpa subject)'}</h3>
          <p>{message.from || 'Unknown sender'}</p>
          <small>{formatDate(message.received_at)}</small>
        </div>
        {message.attachments.length > 0 && <span>{message.attachments.length} attachment</span>}
      </button>
    </li>
  )
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat('id-ID', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value))
}
