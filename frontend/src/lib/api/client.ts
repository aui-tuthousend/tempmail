import type { GenerateMailboxResponse, ListMessagesResponse } from './types'

export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? '/api'

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...init,
    headers: {
      'Content-Type': 'application/json',
      ...init?.headers,
    },
  })

  if (!response.ok) {
    const body = await response.text()
    throw new Error(body || `Request failed with status ${response.status}`)
  }

  return response.json() as Promise<T>
}

export function createMailbox() {
  return request<GenerateMailboxResponse>('/mailboxes', { method: 'POST' })
}

export function listMessages(mailbox: string) {
  return request<ListMessagesResponse>(`/mailboxes/${encodeURIComponent(mailbox)}/messages`)
}

export function mailboxEventsUrl(mailbox: string) {
  return `${API_BASE_URL}/mailboxes/${encodeURIComponent(mailbox)}/events`
}
