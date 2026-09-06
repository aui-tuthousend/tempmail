import { useState } from 'react'

import type { Message } from '../lib/api/types'
import { Badge } from './ui/badge'
import { Button } from './ui/button'
import { ScrollArea } from './ui/scroll-area'
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
      <section className="rounded-3xl border border-slate-200 bg-white/90 p-8 text-center text-slate-500 shadow-xl shadow-slate-900/5">
        <h2 className="text-2xl font-bold text-slate-950">Login diperlukan</h2>
        <p className="mt-2">Login terlebih dahulu untuk melihat inbox persistent account.</p>
      </section>
    )
  }

  if (isLoading) {
    return <section className="rounded-3xl border border-slate-200 bg-white/90 p-8 text-center text-slate-500 shadow-xl shadow-slate-900/5">Memuat inbox...</section>
  }

  if (messages.length === 0) {
    return (
      <section className="rounded-3xl border border-slate-200 bg-white/90 p-8 text-center text-slate-500 shadow-xl shadow-slate-900/5">
        <h2 className="text-2xl font-bold text-slate-950">Inbox kosong</h2>
        <p className="mt-2">Email baru akan muncul otomatis lewat SSE setelah diterima SMTP.</p>
      </section>
    )
  }

  return (
    <>
      <section className="rounded-3xl border border-slate-200 bg-white/90 p-5 shadow-xl shadow-slate-900/5">
        <div className="flex items-center justify-between gap-4">
          <p className="text-xs font-extrabold uppercase tracking-widest text-indigo-600">Inbox</p>
          <Badge>{messages.length} email</Badge>
        </div>
        <ScrollArea className="mt-4 max-h-[calc(100vh-18rem)] pr-3">
          <ul className="grid list-none gap-3 p-0">
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
        </ScrollArea>
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
    <li className="overflow-hidden rounded-3xl border border-slate-200 bg-white">
      <button
        type="button"
        className="flex w-full items-center justify-between gap-4 bg-transparent p-4 text-left text-slate-950 transition hover:bg-slate-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500"
        onClick={onOpen}
      >
        <div className="min-w-0">
          <h3 className={message.is_read ? 'truncate text-base font-semibold text-slate-600' : 'truncate text-base font-bold text-slate-950'}>
            {message.subject || '(Tanpa subject)'}
          </h3>
          <p className="mt-1 truncate text-sm text-slate-500">{message.from_name || message.from_address || 'Unknown sender'}</p>
          <small className="mt-1 block text-xs text-slate-400">{formatDate(message.received_at)}</small>
        </div>
        {message.has_attachments && <Badge>attachment</Badge>}
      </button>
      <div className="flex flex-wrap gap-2 border-t border-slate-100 p-3">
        <Button type="button" variant="secondary" size="sm" onClick={onToggleRead}>
          {message.is_read ? 'Unread' : 'Read'}
        </Button>
        <Button type="button" variant="secondary" size="sm" onClick={onToggleStar}>
          {message.is_starred ? 'Unstar' : 'Star'}
        </Button>
        <Button type="button" variant="secondary" size="sm" onClick={onArchive}>
          Archive
        </Button>
        <Button type="button" variant="destructive" size="sm" onClick={onDelete}>
          Delete
        </Button>
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
