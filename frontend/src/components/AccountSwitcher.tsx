import type { SessionAccount } from '../lib/api/types'
import { Badge } from './ui/badge'
import { Button } from './ui/button'

type AccountSwitcherProps = {
  accounts: SessionAccount[]
  isSwitching: boolean
  onActivate: (accountId: string) => void
}

export function AccountSwitcher({ accounts, isSwitching, onActivate }: AccountSwitcherProps) {
  const activeAccount = accounts.find((account) => account.is_active)

  if (accounts.length === 0) {
    return null
  }

  return (
    <section className="flex flex-col gap-4 rounded-3xl border border-slate-200 bg-white/90 p-5 shadow-xl shadow-slate-900/5 md:flex-row md:items-center md:justify-between">
      <div>
        <p className="mb-2 text-xs font-extrabold uppercase tracking-widest text-indigo-600">Active account</p>
        <h2 className="break-all text-2xl font-bold tracking-tight text-slate-950">{activeAccount?.address ?? 'Pilih account'}</h2>
      </div>
      <div className="flex flex-wrap gap-2">
        {accounts.map((account) => (
          <Button
            key={account.account_id}
            type="button"
            variant={account.is_active ? 'default' : 'secondary'}
            size="sm"
            disabled={isSwitching || account.is_active}
            onClick={() => onActivate(account.account_id)}
          >
            {account.display_name || account.username}
            {account.is_active && <Badge className="bg-white/15 text-white">aktif</Badge>}
          </Button>
        ))}
      </div>
    </section>
  )
}
