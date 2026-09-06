import type { Message } from '../lib/api/types'
import { Button } from './ui/button'
import { Dialog, DialogClose, DialogContent, DialogDescription, DialogTitle } from './ui/dialog'
import { ScrollArea } from './ui/scroll-area'

type EmailDetailDialogProps = {
  message: Message | null
  onClose: () => void
}

export function EmailDetailDialog({ message, onClose }: EmailDetailDialogProps) {
  const body = message?.text_body || message?.html_body || 'Email ini tidak memiliki konten teks.'

  return (
    <Dialog open={Boolean(message)} onOpenChange={(open) => !open && onClose()}>
      {message && (
        <DialogContent>
          <div className="flex items-start justify-between gap-4 border-b border-slate-100 pb-4">
            <div>
              <p className="mb-2 text-xs font-extrabold uppercase tracking-widest text-indigo-600">Email detail</p>
              <DialogTitle className="text-2xl font-bold tracking-tight text-slate-950">
                {message.subject || '(Tanpa subject)'}
              </DialogTitle>
              <DialogDescription className="mt-2 text-sm text-slate-500">{formatSender(message)}</DialogDescription>
            </div>
            <DialogClose asChild>
              <Button type="button" variant="secondary" size="sm">
                Tutup
              </Button>
            </DialogClose>
          </div>

          <ScrollArea className="max-h-[70vh] pr-3">
            <dl className="my-4 grid gap-3 md:grid-cols-2">
              <MetaItem label="Dari" value={formatSender(message)} />
              <MetaItem label="Ke" value={formatAddresses(message.to_addresses)} />
              <MetaItem label="CC" value={formatAddresses(message.cc_addresses)} />
              <MetaItem label="Diterima" value={formatDate(message.received_at)} />
            </dl>

            <div className="grid gap-3">
              <h3 className="text-lg font-bold text-slate-950">Konten</h3>
              <pre className="whitespace-pre-wrap break-words rounded-3xl bg-slate-950 p-4 font-mono text-sm leading-6 text-slate-100">
                {body}
              </pre>
            </div>

            {message.has_attachments && (
              <div className="mt-4 rounded-2xl bg-slate-50 p-4">
                <h3 className="font-bold text-slate-950">Attachments</h3>
                <p className="mt-1 text-sm text-slate-500">Attachment metadata tersimpan di backend.</p>
              </div>
            )}
          </ScrollArea>
        </DialogContent>
      )}
    </Dialog>
  )
}

function MetaItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-2xl bg-slate-50 p-3">
      <dt className="mb-1 text-xs font-extrabold uppercase tracking-wider text-slate-500">{label}</dt>
      <dd className="break-words text-sm text-slate-900">{value}</dd>
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
