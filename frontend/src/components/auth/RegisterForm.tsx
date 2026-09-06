import { Link, useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { toast } from 'sonner'

import { useLocalPartAvailability } from '../../lib/hooks/useLocalPartAvailability'
import { useCreateAccount } from '../../lib/hooks/useSession'
import { Button } from '../ui/button'
import { Input } from '../ui/input'
import { Label } from '../ui/label'
import { isPasswordValid, joinMailboxAddress, mailboxDomain } from './auth-utils'

export function RegisterForm() {
  const navigate = useNavigate()
  const createAccount = useCreateAccount()
  const [apiKey, setApiKey] = useState('')
  const [username, setUsername] = useState('')
  const [localPart, setLocalPart] = useState('')
  const [displayName, setDisplayName] = useState('')
  const [password, setPassword] = useState('')
  const passwordValid = isPasswordValid(password)
  const availability = useLocalPartAvailability(localPart, username)
  const localPartAvailable = availability.data?.local_part_available
  const usernameAvailable = availability.data?.username_available
  const localPartTaken = localPartAvailable === false
  const usernameTaken = usernameAvailable === false
  const canSubmit = passwordValid && !localPartTaken && !usernameTaken

  return (
    <form
      className="grid w-full max-w-md gap-4 rounded-3xl border border-slate-200 bg-white p-6 shadow-xl shadow-slate-900/5"
      onSubmit={(event) => {
        event.preventDefault()

        if (!canSubmit) {
          toast.error('Cek kembali username, email, dan password')
          return
        }

        createAccount.mutate(
          {
            apiKey,
            payload: {
              username,
              email: joinMailboxAddress(localPart),
              password,
              display_name: displayName || null,
            },
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
        <p className="mt-2 text-sm text-slate-500">Buat account baru dengan API key valid.</p>
      </div>

      <div className="grid gap-2">
        <Label htmlFor="api-key">API key</Label>
        <Input id="api-key" value={apiKey} onChange={(event) => setApiKey(event.target.value)} required />
      </div>

      <div className="grid gap-2">
        <Label htmlFor="username">Username</Label>
        <Input id="username" value={username} onChange={(event) => setUsername(event.target.value)} required />
        {availability.isFetching && username.trim() && <p className="text-sm font-medium text-slate-500">Mengecek username...</p>}
        {usernameTaken && <p className="text-sm font-medium text-orange-700">Username sudah terdaftar.</p>}
        {usernameAvailable === true && <p className="text-sm font-medium text-emerald-700">Username tersedia.</p>}
      </div>

      <div className="grid gap-2">
        <Label htmlFor="email-local-part">Email</Label>
        <div className="flex">
          <Input
            id="email-local-part"
            className="rounded-r-none"
            value={localPart}
            onChange={(event) => setLocalPart(event.target.value)}
            placeholder="localpart"
            required
          />
          <Input className="w-[42%] min-w-36 rounded-l-none border-l-0" value={`@${mailboxDomain || 'domain'}`} disabled readOnly />
        </div>
        {availability.isFetching && localPart.trim() && <p className="text-sm font-medium text-slate-500">Mengecek email...</p>}
        {localPartTaken && <p className="text-sm font-medium text-orange-700">Local part email sudah terdaftar.</p>}
        {localPartAvailable === true && <p className="text-sm font-medium text-emerald-700">Local part email tersedia.</p>}
      </div>

      <div className="grid gap-2">
        <Label htmlFor="display-name">Display name</Label>
        <Input id="display-name" value={displayName} onChange={(event) => setDisplayName(event.target.value)} placeholder="Opsional" />
      </div>

      <div className="grid gap-2">
        <Label htmlFor="password">Password</Label>
        <Input id="password" value={password} onChange={(event) => setPassword(event.target.value)} type="password" required />
        <p className={passwordValid ? 'text-sm font-medium text-emerald-700' : 'text-sm font-medium text-orange-700'}>
          Password minimal 8 karakter dan 1 huruf kapital.
        </p>
      </div>

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
