import type { SessionAccount } from '../../lib/api/types'
import { SearchInput } from './SearchInput'
import { UserMenu } from './UserMenu'

const mailboxDomain = import.meta.env.VITE_MAILBOX_DOMAIN ?? 'mailbox'

type MailHeaderProps = {
  accounts: SessionAccount[]
  search: string
  onSearchChange: (value: string) => void
}

export function MailHeader({ accounts, search, onSearchChange }: MailHeaderProps) {
  return (
    <header className="sticky top-0 z-20 grid gap-3 border-b border-slate-200 bg-white/90 px-4 py-3 backdrop-blur md:grid-cols-[minmax(0,1fr)_minmax(18rem,42rem)_auto] md:items-center md:px-6">
      <div className="min-w-0">
        <p className="text-xs font-extrabold uppercase tracking-widest text-indigo-600">Domain</p>
        <h1 className="truncate text-lg font-bold text-slate-950">{mailboxDomain}</h1>
      </div>
      <SearchInput value={search} onChange={onSearchChange} />
      <div className="justify-self-end">
        <UserMenu accounts={accounts} />
      </div>
    </header>
  )
}
