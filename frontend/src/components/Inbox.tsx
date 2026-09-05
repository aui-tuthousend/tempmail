import { useState } from 'react'

import type { Message } from '../lib/api/types'
import { EmailDetailDialog } from './EmailDetailDialog'

type InboxProps = {
  messages: Message[]
  isLoading: boolean
  isAuthenticated: boolean
  onToggleRead: (message: Message) => void
  onToggleStar: (message: Message) => void
  onArchive: (message: Message) => void
  onDelete: (message: Message) => void
}

export function Inbox({
  messages,
  isLoading,
  isAuthenticated,
  onToggleRead,
  onToggleStar,
  onArchive,
  onDelete,
}: InboxProps) {
  const [selectedMessage, setSelectedMessage] = useState<Message | null>(null)

  if (!isAuthenticated) {
    return (
      <section className="card empty-state">
        <h2>Login diperlukan</h2>
        <p>Login terlebih dahulu untuk melihat inbox persistent account.</p>
      </section>
    )
  }

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
            <MessageItem
              key={message.id}
              message={message}
              onOpen={() => setSelectedMessage(message)}
              onArchive={() => onArchive(message)}
              onDelete={() => onDelete(message)}
              onToggleRead={() => onToggleRead(message)}
              onToggleStar={() => onToggleStar(message)}
            />
          ))}
        </ul>
      </section>

      <EmailDetailDialog message={selectedMessage} onClose={() => setSelectedMessage(null)} />
    </>
  )
}

function MessageItem({
  message,
  onOpen,
  onToggleRead,
  onToggleStar,
  onArchive,
  onDelete,
}: {
  message: Message
  onOpen: () => void
  onToggleRead: () => void
  onToggleStar: () => void
  onArchive: () => void
  onDelete: () => void
}) {
  return (
    <li className={message.is_read ? 'message-item read' : 'message-item unread'}>
      <button type="button" className="message-button" onClick={onOpen}>
        <div>
          <h3>{message.subject || '(Tanpa subject)'}</h3>
          <p>{message.from_name || message.from_address || 'Unknown sender'}</p>
          <small>{formatDate(message.received_at)}</small>
        </div>
        {message.has_attachments && <span>attachment</span>}
      </button>
      <div className="message-actions">
        <button type="button" className="ghost-button" onClick={onToggleRead}>
          {message.is_read ? 'Unread' : 'Read'}
        </button>
        <button type="button" className="ghost-button" onClick={onToggleStar}>
          {message.is_starred ? 'Unstar' : 'Star'}
        </button>
        <button type="button" className="ghost-button" onClick={onArchive}>
          Archive
        </button>
        <button type="button" className="ghost-button danger" onClick={onDelete}>
          Delete
        </button>
      </div>
    </li>
  )
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat('id-ID', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value))
}
