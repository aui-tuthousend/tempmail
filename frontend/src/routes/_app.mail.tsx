import { createFileRoute, Outlet } from '@tanstack/react-router'
import { RefreshCw } from 'lucide-react'
import { toast } from 'sonner'

import { Inbox } from '../components/Inbox'
import { useMailSearch } from '../components/mail/search-context'
import { Button } from '../components/ui/button'
import type { Message, MessageView } from '../lib/api/types'
import { useInboxEvents } from '../lib/hooks/useInboxEvents'
import { useDeleteMessage, useMessages, useUpdateMessage } from '../lib/hooks/useMessages'

export const Route = createFileRoute('/_app/mail')({
  component: MailLayout,
})

function MailLayout() {
  return <Outlet />
}

export function MailView({ view, title }: { view: MessageView; title: string }) {
  const { search } = useMailSearch()
  const messages = useMessages(true, view)
  const updateMessage = useUpdateMessage()
  const deleteMessage = useDeleteMessage()
  const visibleMessages = filterMessages(messages.data ?? [], search)

  useInboxEvents(view === 'inbox')

  return (
    <section className="flex h-full min-h-0 flex-col gap-5 overflow-hidden">
      <div className="flex flex-col gap-4 rounded-3xl border border-slate-200 bg-white/90 p-5 shadow-xl shadow-slate-900/5 md:flex-row md:items-center md:justify-between">
        <div>
          <p className="mb-2 text-xs font-extrabold uppercase tracking-widest text-indigo-600">Mailbox</p>
          <h2 className="text-3xl font-black tracking-tight text-slate-950">{title}</h2>
          {search && <p className="mt-2 text-sm text-slate-500">Hasil pencarian untuk “{search}”</p>}
        </div>
        <Button
          type="button"
          variant="secondary"
          disabled={messages.isFetching}
          onClick={() => {
            void messages.refetch().then(() => toast.success('Mailbox diperbarui'))
          }}
        >
          <RefreshCw className="h-4 w-4" />
          Refresh
        </Button>
      </div>

      {messages.isError && (
        <section className="rounded-3xl border border-orange-200 bg-orange-50 p-5 text-center font-medium text-orange-700">
          Gagal memuat email. Session mungkin sudah expired.
        </section>
      )}

      <Inbox
        messages={visibleMessages}
        view={view}
        isLoading={messages.isLoading}
        isAuthenticated
        onMarkRead={(message) =>
          updateMessage.mutate(
            { messageId: message.id, payload: { is_read: true } },
            { onError: (error) => toast.error(error.message) },
          )
        }
        onToggleStar={(message) =>
          updateMessage.mutate(
            { messageId: message.id, payload: { is_starred: !message.is_starred } },
            { onError: (error) => toast.error(error.message) },
          )
        }
        onToggleArchive={(message) =>
          updateMessage.mutate(
            { messageId: message.id, payload: { is_archived: !message.is_archived } },
            {
              onSuccess: () => toast.success(message.is_archived ? 'Email dikembalikan ke inbox' : 'Email diarsipkan'),
              onError: (error) => toast.error(error.message),
            },
          )
        }
        onDelete={(message) =>
          updateMessage.mutate(
            { messageId: message.id, payload: { is_deleted: true } },
            { onSuccess: () => toast.success('Email dihapus'), onError: (error) => toast.error(error.message) },
          )
        }
        onRestore={(message) =>
          updateMessage.mutate(
            { messageId: message.id, payload: { is_deleted: false } },
            { onSuccess: () => toast.success('Email dipulihkan'), onError: (error) => toast.error(error.message) },
          )
        }
        onPermanentDelete={(message) =>
          deleteMessage.mutate(message.id, {
            onSuccess: () => toast.success('Email dihapus permanen'),
            onError: (error) => toast.error(error.message),
          })
        }
      />
    </section>
  )
}

function filterMessages(messages: Message[], search: string) {
  const term = search.trim().toLowerCase()

  if (!term) {
    return messages
  }

  return messages.filter((message) =>
    [message.subject, message.from_address, message.from_name, message.text_body]
      .filter(Boolean)
      .some((value) => value!.toLowerCase().includes(term)),
  )
}
