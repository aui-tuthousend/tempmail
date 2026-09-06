import { useState } from 'react'
import { Archive, Paperclip, RotateCcw, Star, Trash2 } from 'lucide-react'

import type { Message, MessageView } from '../lib/api/types'
import { Badge } from './ui/badge'
import { Button } from './ui/button'
import { ScrollArea } from './ui/scroll-area'
import { EmailDetailDialog } from './EmailDetailDialog'

const emptySnippet = 'Tidak ada cuplikan konten.'

type InboxProps = {
  messages: Message[]
  view?: MessageView
  isLoading: boolean
  isAuthenticated: boolean
  onMarkRead: (message: Message) => void
  onToggleStar: (message: Message) => void
  onToggleArchive: (message: Message) => void
  onDelete: (message: Message) => void
  onRestore?: (message: Message) => void
  onPermanentDelete?: (message: Message) => void
}

export function Inbox({
  messages,
  view = 'inbox',
  isLoading,
  isAuthenticated,
  onMarkRead,
  onToggleStar,
  onToggleArchive,
  onDelete,
  onRestore,
  onPermanentDelete,
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
      <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-3xl border border-slate-200 bg-white/90 p-5 shadow-xl shadow-slate-900/5">
        <div className="flex items-center justify-between gap-4">
          <p className="text-xs font-extrabold uppercase tracking-widest text-indigo-600">Inbox</p>
          <Badge>{messages.length} email</Badge>
        </div>
        <ScrollArea className="mt-4 min-h-0 flex-1 pr-2">
          <ul className="grid list-none gap-2 p-0">
            {messages.map((message) => (
              <MessageItem
                key={message.id}
                message={message}
                view={view}
                onOpen={() => {
                  if (!message.is_read) {
                    onMarkRead(message)
                  }

                  setSelectedMessage({ ...message, is_read: true })
                }}
                onDelete={() => onDelete(message)}
                onPermanentDelete={() => onPermanentDelete?.(message)}
                onRestore={() => onRestore?.(message)}
                onToggleArchive={() => onToggleArchive(message)}
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
  view,
  onOpen,
  onToggleStar,
  onToggleArchive,
  onDelete,
  onRestore,
  onPermanentDelete,
}: {
  message: Message
  view: MessageView
  onOpen: () => void
  onToggleStar: () => void
  onToggleArchive: () => void
  onDelete: () => void
  onRestore: () => void
  onPermanentDelete: () => void
}) {
  const sender = message.from_name || message.from_address || 'Unknown sender'
  const snippet = messageSnippet(message)

  return (
    <li className={message.is_read ? 'overflow-hidden rounded-2xl border border-slate-200 bg-slate-100' : 'overflow-hidden rounded-2xl border border-indigo-100 bg-white'}>
      <div className="flex items-center gap-3 p-3 transition hover:bg-slate-50">
        <button
          type="button"
          className="min-w-0 flex-1 bg-transparent text-left text-slate-950 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500"
          onClick={onOpen}
        >
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <h3 className={message.is_read ? 'truncate text-base font-semibold text-slate-600' : 'truncate text-base font-bold text-slate-950'}>
                {message.subject || '(Tanpa subject)'}
              </h3>
              {message.has_attachments && <Paperclip className="h-4 w-4 shrink-0 text-slate-400" />}
            </div>
            <p className="mt-1 truncate text-sm text-slate-500">
              <span className="font-semibold text-slate-700">{sender}</span>
              <span className="px-1 text-slate-300">-</span>
              <span>{snippet}</span>
            </p>
            <small className="mt-1 block text-xs text-slate-400">{formatDate(message.received_at)}</small>
          </div>
        </button>

        <MessageActions
          isArchived={message.is_archived}
          isDeletedView={view === 'deleted'}
          isStarred={message.is_starred}
          onDelete={onDelete}
          onPermanentDelete={onPermanentDelete}
          onRestore={onRestore}
          onToggleArchive={onToggleArchive}
          onToggleStar={onToggleStar}
        />
      </div>
    </li>
  )
}

function MessageActions({
  isArchived,
  isDeletedView,
  isStarred,
  onDelete,
  onPermanentDelete,
  onRestore,
  onToggleArchive,
  onToggleStar,
}: {
  isArchived: boolean
  isDeletedView: boolean
  isStarred: boolean
  onDelete: () => void
  onPermanentDelete: () => void
  onRestore: () => void
  onToggleArchive: () => void
  onToggleStar: () => void
}) {
  if (isDeletedView) {
    return (
      <div className="flex shrink-0 items-center gap-1 rounded-full border border-slate-200 bg-white/80 p-1 shadow-sm">
        <Button type="button" variant="ghost" size="icon" title="Restore message" onClick={onRestore}>
          <RotateCcw className="h-4 w-4 text-emerald-600" />
        </Button>
        <Button type="button" variant="ghost" size="icon" title="Delete permanent" onClick={onPermanentDelete}>
          <Trash2 className="h-4 w-4 text-red-600" />
        </Button>
      </div>
    )
  }

  return (
    <div className="flex shrink-0 items-center gap-1 rounded-full border border-slate-200 bg-white/80 p-1 shadow-sm">
      <Button type="button" variant="ghost" size="icon" title={isStarred ? 'Hapus star' : 'Beri star'} onClick={onToggleStar}>
        <Star className={isStarred ? 'h-4 w-4 fill-yellow-400 text-yellow-500' : 'h-4 w-4'} />
      </Button>
      <Button type="button" variant="ghost" size="icon" title={isArchived ? 'Unarchive' : 'Archive'} onClick={onToggleArchive}>
        <Archive className={isArchived ? 'h-4 w-4 text-indigo-600' : 'h-4 w-4'} />
      </Button>
      <Button type="button" variant="ghost" size="icon" title="Delete" onClick={onDelete}>
        <Trash2 className="h-4 w-4 text-red-600" />
      </Button>
    </div>
  )
}

function messageSnippet(message: Message) {
  const source = message.text_body || stripHtml(message.html_body || '')
  const normalized = source.replace(/\s+/g, ' ').trim()

  return normalized || emptySnippet
}

function stripHtml(value: string) {
  return value.replace(/<[^>]*>/g, ' ')
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat('id-ID', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value))
}
