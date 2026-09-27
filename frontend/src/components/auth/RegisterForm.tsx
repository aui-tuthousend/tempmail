import { Link, useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { toast } from 'sonner'

import { useLocalPartAvailability } from '../../lib/hooks/useLocalPartAvailability'
import { useCreateAccount } from '../../lib/hooks/useSession'
import { Button } from '../ui/button'
import { Input } from '../ui/input'
import { Label } from '../ui/label'
import { joinMailboxAddress, mailboxDomain, normalizeLocalPart } from './auth-utils'

const DEFAULT_REGISTER_PASSWORD = 'Walawe123!'

function randomLocalPart() {
  const random = new Uint32Array(1)
  crypto.getRandomValues(random)
  return `user${random[0].toString(36)}`
}

export function RegisterForm() {
  const navigate = useNavigate()
  const createAccount = useCreateAccount()
  const [localPart, setLocalPart] = useState('')
  const normalizedLocalPart = normalizeLocalPart(localPart)
  const availability = useLocalPartAvailability(normalizedLocalPart, normalizedLocalPart)
  const localPartAvailable = availability.data?.local_part_available
  const usernameAvailable = availability.data?.username_available
  const localPartTaken = localPartAvailable === false
  const usernameTaken = usernameAvailable === false
  const canSubmit = Boolean(normalizedLocalPart) && !localPartTaken && !usernameTaken

  return (
    <form
      className="grid w-full max-w-md gap-4 rounded-3xl border border-slate-200 bg-white p-6 shadow-xl shadow-slate-900/5"
      onSubmit={(event) => {
        event.preventDefault()

        if (!canSubmit) {
          toast.error('Localpart belum tersedia')
          return
        }

        createAccount.mutate(
          {
            username: normalizedLocalPart,
            email: joinMailboxAddress(normalizedLocalPart),
            password: DEFAULT_REGISTER_PASSWORD,
            display_name: null,
          },
          {
            onSuccess: () => {
              toast.success('Account berhasil dibuat')
              void navigate({ to: '/login' })
            },
            onError: (error) => toast.error(error.message),
          },
        )
      }}
    >
      <div>
        <p className="mb-2 text-xs font-extrabold uppercase tracking-widest text-indigo-600">TempMail</p>
        <h1 className="text-3xl font-black tracking-tight text-slate-950">Register</h1>
        <p className="mt-2 text-sm text-slate-500">
          Cukup isi localpart email. Username akan sama dengan localpart, password awal: {DEFAULT_REGISTER_PASSWORD}
        </p>
      </div>

      <div className="grid gap-2">
        <Label htmlFor="email-local-part">Localpart email</Label>
        <div className="flex">
          <Input
            id="email-local-part"
            className="rounded-r-none"
            value={localPart}
            onChange={(event) => setLocalPart(normalizeLocalPart(event.target.value))}
            placeholder="localpart"
            required
          />
          <Input className="w-[42%] min-w-36 rounded-l-none border-l-0" value={`@${mailboxDomain || 'domain'}`} disabled readOnly />
        </div>
        {availability.isFetching && normalizedLocalPart && <p className="text-sm font-medium text-slate-500">Mengecek localpart...</p>}
        {(localPartTaken || usernameTaken) && <p className="text-sm font-medium text-orange-700">Localpart sudah terdaftar.</p>}
        {localPartAvailable === true && usernameAvailable === true && (
          <p className="text-sm font-medium text-emerald-700">Localpart tersedia.</p>
        )}
      </div>

      <Button type="button" variant="default" onClick={() => setLocalPart(randomLocalPart())}>
        Generate random localpart
      </Button>

      <Button type="submit" disabled={createAccount.isPending || !canSubmit}>
        {createAccount.isPending ? 'Membuat...' : 'Buat account'}
      </Button>

      <p className="text-center text-sm text-slate-500">
        Sudah punya account?{' '}
        <Link className="font-semibold text-indigo-600 hover:text-indigo-500" to="/login">
          Login
        </Link>
      </p>
    </form>
  )
}
