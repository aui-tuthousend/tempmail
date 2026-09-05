import { useMutation, useQuery } from '@tanstack/react-query'

import { listMessages, updateMessage } from '../api/client'
import type { UpdateMessageRequest } from '../api/types'
import { queryClient } from '../queryClient'
import { messagesQueryKey } from './useSession'

export function useMessages(isAuthenticated: boolean) {
  return useQuery({
    queryKey: messagesQueryKey,
    queryFn: listMessages,
    enabled: isAuthenticated,
    retry: false,
  })
}

export function useUpdateMessage() {
  return useMutation({
    mutationFn: ({ messageId, payload }: { messageId: string; payload: UpdateMessageRequest }) =>
      updateMessage(messageId, payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: messagesQueryKey })
    },
  })
}
