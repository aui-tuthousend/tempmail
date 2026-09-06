import type { Mailbox } from '../lib/api/types'
import { Button } from './ui/button'

type MailboxCardProps = {
  mailbox: Mailbox
  address: string
  isGenerating: boolean
  onGenerate: () => void
}

export function MailboxCard({ mailbox, address, isGenerating, onGenerate }: MailboxCardProps) {
  return (
    <section className="flex flex-col gap-4 rounded-3xl border border-slate-200 bg-white/90 p-5 shadow-xl shadow-slate-900/5 md:flex-row md:items-center md:justify-between">
      <div>
        <p className="mb-2 text-xs font-extrabold uppercase tracking-widest text-indigo-600">Temporary address</p>
        <h1 className="break-all text-3xl font-bold tracking-tight text-slate-950">{address}</h1>
        <p className="mt-2 text-sm text-slate-500">Aktif sampai {formatDate(mailbox.expires_at)}</p>
      </div>
      <Button type="button" onClick={onGenerate} disabled={isGenerating}>
        {isGenerating ? 'Generating...' : 'Generate baru'}
      </Button>
    </section>
  )
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat('id-ID', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value))
}
