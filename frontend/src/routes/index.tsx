import { createFileRoute, isRedirect, redirect } from '@tanstack/react-router'

import { listSessionAccountsForRoute } from '../lib/api/sessionGuard'

export const Route = createFileRoute('/')({
  beforeLoad: async () => {
    try {
      const accounts = await listSessionAccountsForRoute()
      throw redirect({ to: accounts.length > 0 ? '/mail' : '/login' })
    } catch (error) {
      if (isRedirect(error)) {
        throw error
      }

      throw redirect({ to: '/login' })
    }
  },
  component: () => null,
})

