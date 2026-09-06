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

export type Attachment = {
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

export type ListMailboxMessagesResponse = {
  mailbox: string
  messages: EmailMessage[]
}

export type ApiErrorBody = {
  error: string
}

export type Account = {
  id: string
  username: string
  address: string
  display_name: string | null
  is_active: boolean
  created_at: string
  updated_at: string
}

export type CreateAccountRequest = {
  username: string
  email: string
  password: string
  display_name?: string | null
}

export type LoginRequest = {
  username_or_email: string
  password: string
  device_info?: Record<string, unknown>
}

export type SessionAccount = {
  account_id: string
  username: string
  address: string
  display_name: string | null
  is_active: boolean
  logged_in_at: string
}

export type LoginResponse = {
  session_id: string
  active_account_id: string
  expires_at: string
  accounts: SessionAccount[]
}

export type AccountAvailabilityResponse = {
  local_part_available: boolean | null
  username_available: boolean | null
}

export type MessageView = 'inbox' | 'starred' | 'archived' | 'deleted'

export type AddressEntry = {
  address: string
  name: string | null
}

export type Message = {
  id: string
  message_id: string | null
  in_reply_to: string | null
  from_address: string
  from_name: string | null
  to_addresses: AddressEntry[]
  cc_addresses: AddressEntry[] | null
  bcc_addresses: AddressEntry[] | null
  subject: string | null
  text_body: string | null
  html_body: string | null
  is_read: boolean
  is_starred: boolean
  is_archived: boolean
  is_deleted: boolean
  has_attachments: boolean
  size_bytes: number
  received_at: string
  created_at: string
  updated_at: string
}

export type UpdateMessageRequest = {
  is_read?: boolean
  is_starred?: boolean
  is_archived?: boolean
  is_deleted?: boolean
}

export type EmailReceivedEvent = {
  type: 'email_received'
  account_id: string
  message_id: string
  mailbox: string
  subject: string | null
  from: string | null
  received_at: string
}
