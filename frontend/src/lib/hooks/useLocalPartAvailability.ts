import { useEffect, useState } from 'react'
import { useQuery } from '@tanstack/react-query'

import { accountAvailability } from '../api/client'

const debounceMs = 450

export function useLocalPartAvailability(localPart: string, username?: string) {
  const [debouncedLocalPart, setDebouncedLocalPart] = useState('')
  const [debouncedUsername, setDebouncedUsername] = useState('')
  const normalizedLocalPart = localPart.trim().toLowerCase().split('@')[0]
  const normalizedUsername = username?.trim() ?? ''

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      setDebouncedLocalPart(normalizedLocalPart)
      setDebouncedUsername(normalizedUsername)
    }, debounceMs)

    return () => window.clearTimeout(timeout)
  }, [normalizedLocalPart, normalizedUsername])

  return useQuery({
    queryKey: ['account-availability', debouncedLocalPart, debouncedUsername] as const,
    queryFn: () =>
      accountAvailability({
        local_part: debouncedLocalPart,
        username: debouncedUsername,
      }),
    enabled: debouncedLocalPart.length > 0 || debouncedUsername.length > 0,
    retry: false,
  })
}
