import { useEffect } from 'react'

import { messageEventsUrl } from '../api/client'
import type { EmailReceivedEvent } from '../api/types'
import { queryClient } from '../queryClient'
import { messagesQueryKey } from './useSession'

export function useInboxEvents(isAuthenticated: boolean) {
  useEffect(() => {
    if (!isAuthenticated) {
      return
    }

    const events = new EventSource(messageEventsUrl(), { withCredentials: true })

    const refreshMessages = () => {
      void queryClient.invalidateQueries({ queryKey: messagesQueryKey })
    }

    events.addEventListener('email.received', (event) => {
      try {
        JSON.parse(event.data) as EmailReceivedEvent
        refreshMessages()
      } catch {
        refreshMessages()
      }
    })

    events.onerror = refreshMessages

    return () => {
      events.close()
    }
  }, [isAuthenticated])
}
