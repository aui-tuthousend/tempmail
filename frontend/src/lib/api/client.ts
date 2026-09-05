import type {
  Account,
  CreateAccountRequest,
  GenerateMailboxResponse,
  ListMailboxMessagesResponse,
  LoginRequest,
  LoginResponse,
  Message,
  SessionAccount,
  UpdateMessageRequest,
} from './types'

export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? '/api'

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...init,
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
      ...init?.headers,
    },
  })

  if (!response.ok) {
    const body = await response.text()
    throw new Error(errorMessage(body, response.status))
  }

  return response.json() as Promise<T>
}

function errorMessage(body: string, status: number) {
  if (!body) {
    return `Request failed with status ${status}`
  }

  try {
    const parsed = JSON.parse(body) as { error?: string }
    return parsed.error || body
  } catch {
    return body
  }
}

export function createMailbox() {
  return request<GenerateMailboxResponse>('/mailboxes', { method: 'POST' })
}

export function listMailboxMessages(mailbox: string) {
  return request<ListMailboxMessagesResponse>(`/mailboxes/${encodeURIComponent(mailbox)}/messages`)
}

export function mailboxEventsUrl(mailbox: string) {
  return `${API_BASE_URL}/mailboxes/${encodeURIComponent(mailbox)}/events`
}

export function createAccount(apiKey: string, payload: CreateAccountRequest) {
  return request<Account>('/accounts', {
    method: 'POST',
    headers: {
      'X-API-Key': apiKey,
    },
    body: JSON.stringify(payload),
  })
}

export function login(payload: LoginRequest) {
  return request<LoginResponse>('/sessions/login', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export function listSessionAccounts() {
  return request<SessionAccount[]>('/session/accounts')
}

export function activateSessionAccount(accountId: string) {
  return request<SessionAccount[]>(`/session/accounts/${accountId}/activate`, {
    method: 'POST',
  })
}

export function listMessages() {
  return request<Message[]>('/messages')
}

export function getMessage(messageId: string) {
  return request<Message>(`/messages/${messageId}`)
}

export function updateMessage(messageId: string, payload: UpdateMessageRequest) {
  return request<Message>(`/messages/${messageId}`, {
    method: 'PATCH',
    body: JSON.stringify(payload),
  })
}

export function messageEventsUrl() {
  return `${API_BASE_URL}/messages/events`
}
