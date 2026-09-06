import { createFileRoute, isRedirect, Outlet, redirect } from '@tanstack/react-router'

import { MailHeader } from '../components/mail/MailHeader'
import { MailSidebar } from '../components/mail/MailSidebar'
import { MailSearchProvider, useMailSearch } from '../components/mail/search-context'
import { listSessionAccountsForRoute } from '../lib/api/sessionGuard'
import { useSessionAccounts } from '../lib/hooks/useSession'

export const Route = createFileRoute('/_app')({
  beforeLoad: async () => {
    try {
      const accounts = await listSessionAccountsForRoute()

      if (accounts.length === 0) {
        throw redirect({ to: '/login' })
      }
    } catch (error) {
      if (isRedirect(error)) {
        throw error
      }

      throw redirect({ to: '/login' })
    }
  },
  component: AppShell,
})

function AppShell() {
  return (
    <MailSearchProvider>
      <ShellContent />
    </MailSearchProvider>
  )
}

function ShellContent() {
  const sessionAccounts = useSessionAccounts()
  const { search, setSearch } = useMailSearch()

  return (
    <div className="min-h-screen bg-slate-100 text-slate-950">
      <div className="flex min-h-screen">
        <MailSidebar />
        <main className="flex min-w-0 flex-1 flex-col">
          <MailHeader accounts={sessionAccounts.data ?? []} search={search} onSearchChange={setSearch} />
          <div className="p-4 md:p-6">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  )
}

