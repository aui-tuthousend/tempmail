import { useMutation, useQuery } from '@tanstack/react-query'

import { deleteMessage, listMessages, updateMessage } from '../api/client'
import type { MessageView, UpdateMessageRequest } from '../api/types'
import { queryClient } from '../queryClient'
import { messagesQueryKey } from './useSession'

export function useMessages(isAuthenticated: boolean, view: MessageView = 'inbox') {
  return useQuery({
    queryKey: [...messagesQueryKey, view] as const,
    queryFn: () => listMessages(view),
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

export function useDeleteMessage() {
  return useMutation({
    mutationFn: deleteMessage,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: messagesQueryKey })
    },
  })
}
