import { Link, useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { toast } from 'sonner'

import { useLogin } from '../../lib/hooks/useSession'
import { Button } from '../ui/button'
import { Input } from '../ui/input'
import { Label } from '../ui/label'
import { joinMailboxAddress, mailboxDomain } from './auth-utils'

export function LoginForm() {
  const navigate = useNavigate()
  const login = useLogin()
  const [localPart, setLocalPart] = useState('')
  const [password, setPassword] = useState('')

  return (
    <form
      className="grid w-full max-w-md gap-4 rounded-3xl border border-slate-200 bg-white p-6 shadow-xl shadow-slate-900/5"
      onSubmit={(event) => {
        event.preventDefault()
        login.mutate(
          {
            username_or_email: joinMailboxAddress(localPart),
            password,
            device_info: {
              user_agent: navigator.userAgent,
              platform: navigator.platform,
            },
          },
          {
            onSuccess: () => {
              toast.success('Login berhasil')
              void navigate({ to: '/mail' })
            },
            onError: (error) => toast.error(error.message),
          },
        )
      }}
    >
      <div>
        <p className="mb-2 text-xs font-extrabold uppercase tracking-widest text-indigo-600">TempMail</p>
        <h1 className="text-3xl font-black tracking-tight text-slate-950">Login</h1>
        <p className="mt-2 text-sm text-slate-500">Masuk ke account mailbox permanen.</p>
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
      </div>

      <div className="grid gap-2">
        <Label htmlFor="password">Password</Label>
        <Input id="password" value={password} onChange={(event) => setPassword(event.target.value)} type="password" required />
      </div>

      <Button type="submit" disabled={login.isPending}>
        {login.isPending ? 'Login...' : 'Login'}
      </Button>

      <p className="text-center text-sm text-slate-500">
        Belum punya account?{' '}
        <Link className="font-semibold text-indigo-600 hover:text-indigo-500" to="/register">
          Register
        </Link>
      </p>
    </form>
  )
}
