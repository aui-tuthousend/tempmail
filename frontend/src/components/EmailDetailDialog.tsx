import DOMPurify from 'dompurify'
import { Reply } from 'lucide-react'

import { useCompose } from './mail/compose-context'
import type { Message } from '../lib/api/types'
import { Button } from './ui/button'
import { Dialog, DialogClose, DialogContent, DialogDescription, DialogTitle } from './ui/dialog'

const emptyBody = 'Email ini tidak memiliki konten.'

type EmailDetailDialogProps = {
  message: Message | null
  onClose: () => void
}

export function EmailDetailDialog({ message, onClose }: EmailDetailDialogProps) {
  const { openCompose } = useCompose()
  const hasHtmlBody = Boolean(message?.html_body)
  const textBody = message?.text_body || emptyBody
  const htmlBody = message?.html_body ? DOMPurify.sanitize(message.html_body) : ''

  return (
    <Dialog open={Boolean(message)} onOpenChange={(open) => !open && onClose()}>
      {message && (
        <DialogContent className="flex h-[82vh] max-h-[82vh] w-full max-w-4xl flex-col overflow-hidden p-5 lg:max-w-6xl">
          <div className="flex shrink-0 items-start justify-between gap-4 border-b border-slate-100 pb-3">
            <div className="min-w-0">
              <p className="mb-1.5 text-[10px] font-extrabold uppercase tracking-widest text-indigo-600">Email detail</p>
              <DialogTitle className="truncate text-xl font-bold tracking-tight text-slate-950">
                {message.subject || '(Tanpa subject)'}
              </DialogTitle>
              <DialogDescription className="mt-1 truncate text-xs text-slate-500">{formatSender(message)}</DialogDescription>
            </div>
            <div className="flex shrink-0 gap-2">
              <Button
                type="button"
                variant="secondary"
                size="sm"
                onClick={() => {
                  openCompose(replyDraft(message))
                  onClose()
                }}
              >
                <Reply className="h-4 w-4" />
                Reply
              </Button>
              <DialogClose asChild>
                <Button type="button" variant="secondary" size="sm">
                  Tutup
                </Button>
              </DialogClose>
            </div>
          </div>

          <div className="min-h-0 flex-1 overflow-y-auto pr-2 [scrollbar-width:thin]">
            <dl className="my-3 grid gap-2 md:grid-cols-2">
              <MetaItem label="Dari" value={formatSender(message)} />
              <MetaItem label="Ke" value={formatAddresses(message.to_addresses)} />
              <MetaItem label="CC" value={formatAddresses(message.cc_addresses)} />
              <MetaItem label="Diterima" value={formatDate(message.received_at)} />
            </dl>

            <div className="grid gap-2">
              <h3 className="text-base font-bold text-slate-950">Konten</h3>
              <div className="overflow-hidden rounded-2xl border border-slate-200 bg-white p-4 text-sm leading-6 text-slate-800">
                {hasHtmlBody ? (
                  <div className="prose prose-slate max-w-none break-words" dangerouslySetInnerHTML={{ __html: htmlBody }} />
                ) : (
                  <div className="whitespace-pre-wrap break-words">{textBody}</div>
                )}
              </div>
            </div>

            {message.has_attachments && (
              <div className="mt-3 rounded-2xl bg-slate-50 p-3">
                <h3 className="font-bold text-slate-950">Attachments</h3>
                <p className="mt-1 text-sm text-slate-500">Attachment metadata tersimpan di backend.</p>
              </div>
            )}
          </div>
        </DialogContent>
      )}
    </Dialog>
  )
}

function MetaItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-xl bg-slate-50 px-3 py-2">
      <dt className="mb-0.5 text-[10px] font-extrabold uppercase tracking-wider text-slate-500">{label}</dt>
      <dd className="break-words text-xs text-slate-900">{value}</dd>
    </div>
  )
}

function replyDraft(message: Message) {
  const subject = message.subject?.startsWith('Re:') ? message.subject : `Re: ${message.subject || '(Tanpa subject)'}`
  const sourceBody = message.text_body || stripHtml(message.html_body || '') || emptyBody

  return {
    to: message.from_address,
    subject,
    in_reply_to: message.message_id,
    text_body: `\n\n--- Original message ---\nDari: ${formatSender(message)}\nDiterima: ${formatDate(message.received_at)}\n\n${sourceBody}`,
  }
}

function formatSender(message: Message) {
  if (!message.from_name) {
    return message.from_address
  }

  return `${message.from_name} <${message.from_address}>`
}

function stripHtml(value: string) {
  return value.replace(/<[^>]*>/g, ' ')
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
