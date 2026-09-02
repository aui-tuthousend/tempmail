import { useEffect } from 'react'

import { mailboxEventsUrl } from '../api/client'
import type { EmailReceivedEvent } from '../api/types'
import { queryClient } from '../queryClient'

export function useInboxEvents(mailbox: string | null) {
  useEffect(() => {
    if (!mailbox) {
      return
    }

    const events = new EventSource(mailboxEventsUrl(mailbox))

    const refreshMessages = () => {
      void queryClient.invalidateQueries({ queryKey: ['mailbox-messages', mailbox] })
    }

    events.addEventListener('email.received', (event) => {
      try {
        const payload = JSON.parse(event.data) as EmailReceivedEvent
        if (payload.mailbox === mailbox) {
          refreshMessages()
        }
      } catch {
        refreshMessages()
      }
    })

    events.onerror = refreshMessages

    return () => {
      events.close()
    }
  }, [mailbox])
}
