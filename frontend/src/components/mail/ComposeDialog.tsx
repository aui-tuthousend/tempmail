import { useEffect, useMemo, useRef, useState, type ChangeEvent, type FormEvent } from 'react'
import { useMutation } from '@tanstack/react-query'
import { Paperclip, Send, X } from 'lucide-react'
import { toast } from 'sonner'

import { sendMessage } from '../../lib/api/client'
import type { SendEmailAttachmentRequest, SendEmailRequest } from '../../lib/api/types'
import { Button } from '../ui/button'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '../ui/dialog'
import { Input } from '../ui/input'

const maxAttachmentBytes = 10 * 1024 * 1024

export type ComposeDraft = {
  to?: string
  subject?: string
  text_body?: string
  in_reply_to?: string | null
}

type ComposeAttachment = SendEmailAttachmentRequest & {
  size_bytes: number
}

type ComposeDialogProps = {
  draft?: ComposeDraft
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function ComposeDialog({ draft, open, onOpenChange }: ComposeDialogProps) {
  const [to, setTo] = useState('')
  const [cc, setCc] = useState('')
  const [bcc, setBcc] = useState('')
  const [subject, setSubject] = useState('')
  const [textBody, setTextBody] = useState('')
  const [htmlBody, setHtmlBody] = useState('')
  const [attachments, setAttachments] = useState<ComposeAttachment[]>([])
  const fileInputRef = useRef<HTMLInputElement | null>(null)

  const mutation = useMutation({
    mutationFn: sendMessage,
    onSuccess: () => {
      toast.success('Email masuk antrean pengiriman')
      onOpenChange(false)
      resetForm()
    },
    onError: (error) => toast.error(error.message),
  })

  useEffect(() => {
    if (!open) {
      return
    }

    setTo(draft?.to ?? '')
    setCc('')
    setBcc('')
    setSubject(draft?.subject ?? '')
    setTextBody(draft?.text_body ?? '')
    setHtmlBody('')
    setAttachments([])
  }, [draft, open])

  const totalAttachmentBytes = useMemo(() => attachments.reduce((sum, attachment) => sum + attachment.size_bytes, 0), [attachments])

  function resetForm() {
    setTo('')
    setCc('')
    setBcc('')
    setSubject('')
    setTextBody('')
    setHtmlBody('')
    setAttachments([])
  }

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()

    const payload: SendEmailRequest = {
      to: parseRecipients(to),
      cc: parseRecipients(cc),
      bcc: parseRecipients(bcc),
      subject: subject.trim() || null,
      text_body: textBody || null,
      html_body: htmlBody || null,
      in_reply_to: draft?.in_reply_to || null,
      attachments: attachments.map(({ size_bytes: _sizeBytes, ...attachment }) => attachment),
    }

    if (payload.to.length === 0) {
      toast.error('Minimal isi 1 penerima')
      return
    }

    mutation.mutate(payload)
  }

  async function handleFiles(event: ChangeEvent<HTMLInputElement>) {
    const files = Array.from(event.target.files ?? [])
    event.target.value = ''

    for (const file of files) {
      if (file.size > maxAttachmentBytes) {
        toast.error(`${file.name} lebih dari 10MB`)
        continue
      }

      const dataBase64 = await fileToBase64(file)
      setAttachments((current) => [
        ...current,
        {
          filename: file.name,
          content_type: file.type || null,
          data_base64: dataBase64,
          size_bytes: file.size,
        },
      ])
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex max-h-[88vh] max-w-3xl flex-col overflow-hidden p-0">
        <div className="shrink-0 border-b border-slate-100 px-6 py-4">
          <DialogTitle className="text-xl font-black tracking-tight text-slate-950">Compose email</DialogTitle>
          <DialogDescription className="mt-1 text-sm text-slate-500">Kirim email dari active account saat ini.</DialogDescription>
        </div>

        <form className="flex min-h-0 flex-1 flex-col" onSubmit={handleSubmit}>
          <div className="min-h-0 flex-1 space-y-3 overflow-y-auto px-6 py-4 [scrollbar-width:thin]">
            <Input value={to} onChange={(event) => setTo(event.target.value)} placeholder="To: user@example.com, another@example.com" />
            <div className="grid gap-3 md:grid-cols-2">
              <Input value={cc} onChange={(event) => setCc(event.target.value)} placeholder="CC (opsional)" />
              <Input value={bcc} onChange={(event) => setBcc(event.target.value)} placeholder="BCC (opsional)" />
            </div>
            <Input value={subject} onChange={(event) => setSubject(event.target.value)} placeholder="Subject" />
            <textarea
              className="min-h-52 w-full resize-y rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm leading-6 text-slate-950 outline-none transition placeholder:text-slate-400 focus:border-indigo-500 focus:ring-2 focus:ring-indigo-100"
              value={textBody}
              onChange={(event) => setTextBody(event.target.value)}
              placeholder="Tulis pesan..."
            />
            <textarea
              className="min-h-28 w-full resize-y rounded-2xl border border-slate-200 bg-white px-4 py-3 font-mono text-xs leading-5 text-slate-950 outline-none transition placeholder:text-slate-400 focus:border-indigo-500 focus:ring-2 focus:ring-indigo-100"
              value={htmlBody}
              onChange={(event) => setHtmlBody(event.target.value)}
              placeholder="HTML body opsional"
            />

            {attachments.length > 0 && (
              <div className="rounded-2xl bg-slate-50 p-3">
                <div className="mb-2 flex items-center justify-between gap-3 text-xs font-bold text-slate-500">
                  <span>Attachments</span>
                  <span>{formatBytes(totalAttachmentBytes)}</span>
                </div>
                <ul className="space-y-2">
                  {attachments.map((attachment, index) => (
                    <li key={`${attachment.filename}-${index}`} className="flex items-center justify-between gap-3 rounded-xl bg-white px-3 py-2 text-sm text-slate-700">
                      <span className="min-w-0 truncate">{attachment.filename}</span>
                      <div className="flex shrink-0 items-center gap-2">
                        <span className="text-xs text-slate-400">{formatBytes(attachment.size_bytes)}</span>
                        <Button type="button" variant="ghost" size="icon" className="h-8 w-8" onClick={() => setAttachments((current) => current.filter((_, itemIndex) => itemIndex !== index))}>
                          <X className="h-4 w-4" />
                        </Button>
                      </div>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>

          <div className="flex shrink-0 flex-col gap-3 border-t border-slate-100 px-6 py-4 md:flex-row md:items-center md:justify-between">
            <div>
              <input ref={fileInputRef} type="file" multiple className="hidden" onChange={handleFiles} />
              <Button type="button" variant="secondary" onClick={() => fileInputRef.current?.click()}>
                <Paperclip className="h-4 w-4" />
                Attach file
              </Button>
              <p className="mt-1 text-xs text-slate-400">Maksimal 10MB per file.</p>
            </div>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
                Batal
              </Button>
              <Button type="submit" disabled={mutation.isPending}>
                <Send className="h-4 w-4" />
                {mutation.isPending ? 'Mengirim...' : 'Send'}
              </Button>
            </div>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  )
}

function parseRecipients(value: string) {
  return value
    .split(/[;,\n]/)
    .map((entry) => entry.trim())
    .filter(Boolean)
}

function fileToBase64(file: File) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const result = String(reader.result ?? '')
      resolve(result.includes(',') ? result.split(',')[1] : result)
    }
    reader.onerror = () => reject(reader.error)
    reader.readAsDataURL(file)
  })
}

function formatBytes(value: number) {
  if (value < 1024) {
    return `${value} B`
  }

  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`
  }

  return `${(value / 1024 / 1024).toFixed(1)} MB`
}
