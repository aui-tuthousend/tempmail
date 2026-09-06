import { createServerFn } from '@tanstack/react-start'

import { API_BASE_URL, listSessionAccounts } from './client'
import type { SessionAccount } from './types'

const serverListSessionAccounts = createServerFn({ method: 'GET' }).handler(async () => {
  const { getRequestHeader, getRequestUrl } = await import('@tanstack/react-start/server')
  const cookie = getRequestHeader('cookie')
  const apiBaseUrl = resolveServerApiBaseUrl(getRequestUrl())

  const response = await fetch(`${apiBaseUrl}/session/accounts`, {
    headers: cookie ? { cookie } : undefined,
  })

  if (!response.ok) {
    throw new Error(`Request failed with status ${response.status}`)
  }

  return response.json() as Promise<SessionAccount[]>
})

export function listSessionAccountsForRoute() {
  if (typeof window === 'undefined') {
    return serverListSessionAccounts()
  }

  return listSessionAccounts()
}

function resolveServerApiBaseUrl(requestUrl: URL) {
  if (/^https?:\/\//.test(API_BASE_URL)) {
    return API_BASE_URL
  }

  return new URL(API_BASE_URL, requestUrl.origin).toString().replace(/\/$/, '')
}
