import type { SessionAccount } from '../lib/api/types'

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
    <section className="card account-switcher">
      <div>
        <p className="eyebrow">Active account</p>
        <h2>{activeAccount?.address ?? 'Pilih account'}</h2>
      </div>
      <div className="account-list">
        {accounts.map((account) => (
          <button
            key={account.account_id}
            type="button"
            className={account.is_active ? 'account-chip active' : 'account-chip'}
            disabled={isSwitching || account.is_active}
            onClick={() => onActivate(account.account_id)}
          >
            {account.display_name || account.username}
          </button>
        ))}
      </div>
    </section>
  )
}
