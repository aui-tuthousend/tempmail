import { useMutation, useQuery } from '@tanstack/react-query'

import { deleteMessage, listMessages, updateMessage } from '../api/client'
import type { Message, MessageView, UpdateMessageRequest } from '../api/types'
import { queryClient } from '../queryClient'
import { messagesQueryKey } from './useSession'

export function messageQueryKey(view: MessageView = 'inbox') {
  return [...messagesQueryKey, view] as const
}

export function useMessages(isAuthenticated: boolean, view: MessageView = 'inbox') {
  return useQuery({
    queryKey: messageQueryKey(view),
    queryFn: () => listMessages(view),
    enabled: isAuthenticated,
    retry: false,
    staleTime: 30_000,
    placeholderData: (previousData) => previousData,
  })
}

function patchMessageInCache(message: Message) {
  const views: MessageView[] = ['inbox', 'starred', 'archived', 'deleted']

  for (const view of views) {
    queryClient.setQueryData<Message[]>(messageQueryKey(view), (messages) =>
      messages?.map((item) => (item.id === message.id ? message : item)),
    )
  }
}

export function useUpdateMessage() {
  return useMutation({
    mutationFn: ({ messageId, payload }: { messageId: string; payload: UpdateMessageRequest }) =>
      updateMessage(messageId, payload),
    onSuccess: (message) => {
      patchMessageInCache(message)
      void queryClient.invalidateQueries({ queryKey: messagesQueryKey, refetchType: 'none' })
    },
  })
}

export function useDeleteMessage() {
  return useMutation({
    mutationFn: deleteMessage,
    onSuccess: (_, messageId) => {
      const views: MessageView[] = ['inbox', 'starred', 'archived', 'deleted']

      for (const view of views) {
        queryClient.setQueryData<Message[]>(messageQueryKey(view), (messages) => messages?.filter((message) => message.id !== messageId))
      }

      void queryClient.invalidateQueries({ queryKey: messagesQueryKey, refetchType: 'none' })
    },
  })
}
