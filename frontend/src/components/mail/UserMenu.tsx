import { Link, useNavigate } from '@tanstack/react-router'
import { LogOut, Plus } from 'lucide-react'
import { toast } from 'sonner'

import type { SessionAccount } from '../../lib/api/types'
import { useActivateSessionAccount, useLogout } from '../../lib/hooks/useSession'
import { Avatar, AvatarFallback } from '../ui/avatar'
import { Button } from '../ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '../ui/dropdown-menu'

type UserMenuProps = {
  accounts: SessionAccount[]
}

export function UserMenu({ accounts }: UserMenuProps) {
  const navigate = useNavigate()
  const activeAccount = accounts.find((account) => account.is_active)
  const activateAccount = useActivateSessionAccount()
  const logout = useLogout()

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button type="button" variant="ghost" className="rounded-2xl px-2">
          <Avatar>
            <AvatarFallback>{accountInitial(activeAccount?.display_name || activeAccount?.username)}</AvatarFallback>
          </Avatar>
          <span className="hidden max-w-40 truncate text-sm md:inline">{activeAccount?.display_name || activeAccount?.username}</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuGroup>
          {accounts.map((account) => (
            <DropdownMenuItem
              key={account.account_id}
              disabled={account.is_active || activateAccount.isPending}
              onSelect={() => {
                activateAccount.mutate(account.account_id, {
                  onSuccess: () => toast.success('Account aktif diganti'),
                  onError: (error) => toast.error(error.message),
                })
              }}
            >
              <span className="min-w-0 flex-1 truncate">{account.address}</span>
              {account.is_active && <span className="text-xs font-semibold text-indigo-600">aktif</span>}
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuItem asChild>
          <Link to="/login">
            <Plus className="h-4 w-4" />
            Tambah account
          </Link>
        </DropdownMenuItem>
        <DropdownMenuItem
          className="text-red-700 focus:bg-red-50"
          disabled={logout.isPending}
          onSelect={() => {
            logout.mutate(undefined, {
              onSuccess: (nextAccounts) => {
                toast.success('Account logout')
                if (nextAccounts.length === 0) {
                  void navigate({ to: '/login' })
                }
              },
              onError: (error) => toast.error(error.message),
            })
          }}
        >
          <LogOut className="h-4 w-4" />
          Logout account
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

function accountInitial(value: string | undefined) {
  return value?.trim().charAt(0).toUpperCase() || 'M'
}
