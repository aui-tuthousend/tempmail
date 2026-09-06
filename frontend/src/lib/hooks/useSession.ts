import { useMutation, useQuery } from '@tanstack/react-query'

import { activateSessionAccount, createAccount, listSessionAccounts, login, logout } from '../api/client'
import type { CreateAccountRequest, LoginRequest } from '../api/types'
import { queryClient } from '../queryClient'

export const sessionAccountsQueryKey = ['session-accounts'] as const
export const messagesQueryKey = ['messages'] as const

export function useSessionAccounts() {
  return useQuery({
    queryKey: sessionAccountsQueryKey,
    queryFn: listSessionAccounts,
    retry: false,
  })
}

export function useCreateAccount() {
  return useMutation({
    mutationFn: ({ apiKey, payload }: { apiKey: string; payload: CreateAccountRequest }) =>
      createAccount(apiKey, payload),
  })
}

export function useLogin() {
  return useMutation({
    mutationFn: (payload: LoginRequest) => login(payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: sessionAccountsQueryKey })
      void queryClient.invalidateQueries({ queryKey: messagesQueryKey })
    },
  })
}

export function useActivateSessionAccount() {
  return useMutation({
    mutationFn: activateSessionAccount,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: sessionAccountsQueryKey })
      void queryClient.invalidateQueries({ queryKey: messagesQueryKey })
    },
  })
}

export function useLogout() {
  return useMutation({
    mutationFn: logout,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: sessionAccountsQueryKey })
      void queryClient.invalidateQueries({ queryKey: messagesQueryKey })
    },
  })
}
