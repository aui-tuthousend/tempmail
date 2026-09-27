import { useState } from 'react'

import type { CreateAccountRequest, LoginRequest } from '../lib/api/types'
import { Button } from './ui/button'
import { Input } from './ui/input'

const mailboxDomain = import.meta.env.VITE_MAILBOX_DOMAIN ?? ''
const DEFAULT_REGISTER_PASSWORD = 'Walawe123!'

function normalizeLocalPart(value: string) {
  return value.trim().toLowerCase().split('@')[0]
}

function joinMailboxAddress(localPart: string) {
  const value = normalizeLocalPart(localPart)

  if (mailboxDomain.length === 0) {
    return value
  }

  return `${value}@${mailboxDomain}`
}

function randomLocalPart() {
  const random = new Uint32Array(1)
  crypto.getRandomValues(random)
  return `user${random[0].toString(36)}`
}

type AuthPanelProps = {
  isRegistering: boolean
  isLoggingIn: boolean
  registerError: string | null
  loginError: string | null
  onRegister: (payload: CreateAccountRequest) => void
  onLogin: (payload: LoginRequest) => void
}

export function AuthPanel({
  isRegistering,
  isLoggingIn,
  registerError,
  loginError,
  onRegister,
  onLogin,
}: AuthPanelProps) {
  const [emailLocalPart, setEmailLocalPart] = useState('')
  const [loginLocalPart, setLoginLocalPart] = useState('')
  const [loginPassword, setLoginPassword] = useState('')

  return (
    <section className="grid items-stretch gap-4 md:grid-cols-2">
      <form
        className="grid gap-3 rounded-3xl border border-slate-200 bg-white/90 p-5 shadow-xl shadow-slate-900/5"
        onSubmit={(event) => {
          event.preventDefault()
          const localPart = normalizeLocalPart(emailLocalPart)
          onRegister({
            username: localPart,
            email: joinMailboxAddress(localPart),
            password: DEFAULT_REGISTER_PASSWORD,
            display_name: null,
          })
        }}
      >
        <div>
          <p className="mb-2 text-xs font-extrabold uppercase tracking-widest text-indigo-600">Register</p>
          <h2 className="text-2xl font-bold tracking-tight text-slate-950">Buat account email</h2>
          <p className="mt-2 text-sm text-slate-500">Cukup isi localpart. Username sama dengan localpart, password awal: {DEFAULT_REGISTER_PASSWORD}</p>
        </div>
        <div className="flex">
          <Input
            className="rounded-r-none"
            value={emailLocalPart}
            onChange={(event) => setEmailLocalPart(normalizeLocalPart(event.target.value))}
            placeholder="localpart email"
            required
          />
          <Input className="w-[38%] min-w-36 rounded-l-none border-l-0" value={`@${mailboxDomain || 'domain'}`} disabled readOnly />
        </div>
        <Button type="button" variant="default" onClick={() => setEmailLocalPart(randomLocalPart())}>
          Generate random localpart
        </Button>
        <Button type="submit" disabled={isRegistering}>
          {isRegistering ? 'Membuat...' : 'Buat account'}
        </Button>
        {registerError && <p className="text-sm font-medium text-orange-700">{registerError}</p>}
      </form>

      <form
        className="grid gap-3 rounded-3xl border border-slate-200 bg-white/90 p-5 shadow-xl shadow-slate-900/5"
        onSubmit={(event) => {
          event.preventDefault()
          onLogin({
            username_or_email: joinMailboxAddress(loginLocalPart),
            password: loginPassword,
            device_info: {
              user_agent: navigator.userAgent,
              platform: navigator.platform,
            },
          })
        }}
      >
        <div>
          <p className="mb-2 text-xs font-extrabold uppercase tracking-widest text-indigo-600">Login</p>
          <h2 className="text-2xl font-bold tracking-tight text-slate-950">Masuk ke mailbox</h2>
          <p className="mt-2 text-sm text-slate-500">Login account lain di browser yang sama akan menambah account ke session ini.</p>
        </div>
        <div className="flex">
          <Input
            className="rounded-r-none"
            value={loginLocalPart}
            onChange={(event) => setLoginLocalPart(event.target.value)}
            placeholder="localpart email"
            required
          />
          <Input className="w-[38%] min-w-36 rounded-l-none border-l-0" value={`@${mailboxDomain || 'domain'}`} disabled readOnly />
        </div>
        <Input
          value={loginPassword}
          onChange={(event) => setLoginPassword(event.target.value)}
          placeholder="Password"
          type="password"
          required
        />
        <Button type="submit" disabled={isLoggingIn}>
          {isLoggingIn ? 'Login...' : 'Login'}
        </Button>
        {loginError && <p className="text-sm font-medium text-orange-700">{loginError}</p>}
      </form>
    </section>
  )
}
