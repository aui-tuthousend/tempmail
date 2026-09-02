import { useQuery } from '@tanstack/react-query'

import { listMessages } from '../api/client'

export function useMessages(mailbox: string | null) {
  return useQuery({
    queryKey: ['mailbox-messages', mailbox],
    queryFn: () => listMessages(mailbox!),
    enabled: Boolean(mailbox),
  })
}
