import { useEffect } from 'react'
import { toast } from 'sonner'

import { messageEventsUrl } from '../api/client'
import type { EmailReceivedEvent, EmailSentEvent } from '../api/types'
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

    events.addEventListener('email.sent', (event) => {
      try {
        const payload = JSON.parse(event.data) as EmailSentEvent
        if (payload.status === 'sent') {
          toast.success('Email berhasil dikirim')
        } else if (payload.status === 'failed') {
          toast.error('Email gagal dikirim')
        }
      } catch {
        toast.success('Status pengiriman email diperbarui')
      }
    })

    events.onerror = refreshMessages

    return () => {
      events.close()
    }
  }, [isAuthenticated])
}
