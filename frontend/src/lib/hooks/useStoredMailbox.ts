import { useEffect, useState } from 'react'

import type { GenerateMailboxResponse } from '../api/types'

const STORAGE_KEY = 'tempmail.mailbox'

export function useStoredMailbox() {
  const [mailbox, setMailbox] = useState<GenerateMailboxResponse | null>(null)

  useEffect(() => {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (!stored) {
      return
    }

    try {
      setMailbox(JSON.parse(stored) as GenerateMailboxResponse)
    } catch {
      localStorage.removeItem(STORAGE_KEY)
    }
  }, [])

  useEffect(() => {
    if (!mailbox) {
      localStorage.removeItem(STORAGE_KEY)
      return
    }

    localStorage.setItem(STORAGE_KEY, JSON.stringify(mailbox))
  }, [mailbox])

  return { mailbox, setMailbox }
}
