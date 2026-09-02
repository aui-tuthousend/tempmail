import { useMutation } from '@tanstack/react-query'

import { createMailbox } from '../api/client'

export function useCreateMailbox() {
  return useMutation({
    mutationFn: createMailbox,
  })
}
