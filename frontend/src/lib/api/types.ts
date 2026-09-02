export type EmailAddress = {
  local_part: string
  domain: string
}

export type Mailbox = {
  id: string
  address: EmailAddress
  created_at: string
  expires_at: string
}

export type Attachment   = {
  id: string
  filename: string | null
  content_type: string
  size_bytes: number
  storage_key: string
}

export type EmailMessage = {
  id: string
  mailbox: string
  from: string | null
  to: string[]
  subject: string | null
  text_body: string | null
  html_body: string | null
  attachments: Attachment[]
  received_at: string
  expires_at: string
}

export type GenerateMailboxResponse = {
  mailbox: Mailbox
  address: string
}

export type ListMessagesResponse = {
  mailbox: string
  messages: EmailMessage[]
}

export type EmailReceivedEvent = {
  type: 'email_received'
  message_id: string
  mailbox: string
  subject: string | null
  from: string | null
  received_at: string
}
