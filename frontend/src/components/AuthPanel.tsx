import { useState } from 'react'

import type { CreateAccountRequest, LoginRequest } from '../lib/api/types'

const mailboxDomain = import.meta.env.VITE_MAILBOX_DOMAIN ?? ''

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

type AuthPanelProps = {
  isRegistering: boolean
  isLoggingIn: boolean
  registerError: string | null
  loginError: string | null
  onRegister: (apiKey: string, payload: CreateAccountRequest) => void
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
  const [apiKey, setApiKey] = useState('')
  const [username, setUsername] = useState('')
  const [emailLocalPart, setEmailLocalPart] = useState('')
  const [displayName, setDisplayName] = useState('')
  const [registerPassword, setRegisterPassword] = useState('')
  const [loginLocalPart, setLoginLocalPart] = useState('')
  const [loginPassword, setLoginPassword] = useState('')

  return (
    <section className="auth-grid">
      <form
        className="card auth-card"
        onSubmit={(event) => {
          event.preventDefault()
          onRegister(apiKey, {
            username,
            email: joinMailboxAddress(emailLocalPart),
            password: registerPassword,
            display_name: displayName || null,
          })
        }}
      >
        <div>
          <p className="eyebrow">Register</p>
          <h2>Buat account email</h2>
          <p className="muted">Account baru membutuhkan API key valid. Domain email diambil dari env frontend.</p>
        </div>
        <input value={apiKey} onChange={(event) => setApiKey(event.target.value)} placeholder="API key" required />
        <input value={username} onChange={(event) => setUsername(event.target.value)} placeholder="Username" required />
        <div className="email-input-group">
          <input
            value={emailLocalPart}
            onChange={(event) => setEmailLocalPart(event.target.value)}
            placeholder="localpart email"
            required
          />
          <input className="email-domain-input" value={`@${mailboxDomain || 'domain'}`} disabled readOnly />
        </div>
        <input value={displayName} onChange={(event) => setDisplayName(event.target.value)} placeholder="Display name opsional" />
        <input
          value={registerPassword}
          onChange={(event) => setRegisterPassword(event.target.value)}
          placeholder="Password"
          type="password"
          required
        />
        <button type="submit" disabled={isRegistering}>
          {isRegistering ? 'Membuat...' : 'Buat account'}
        </button>
        {registerError && <p className="form-error">{registerError}</p>}
      </form>

      <form
        className="card auth-card"
        onSubmit={(event) => {
          event.preventDefault()
          onLogin({ username_or_email: joinMailboxAddress(loginLocalPart), password: loginPassword })
        }}
      >
        <div>
          <p className="eyebrow">Login</p>
          <h2>Masuk ke mailbox</h2>
          <p className="muted">Login account lain di browser yang sama akan menambah account ke session ini.</p>
        </div>
        <div className="email-input-group">
          <input
            value={loginLocalPart}
            onChange={(event) => setLoginLocalPart(event.target.value)}
            placeholder="localpart email"
            required
          />
          <input className="email-domain-input" value={`@${mailboxDomain || 'domain'}`} disabled readOnly />
        </div>
        <input
          value={loginPassword}
          onChange={(event) => setLoginPassword(event.target.value)}
          placeholder="Password"
          type="password"
          required
        />
        <button type="submit" disabled={isLoggingIn}>
          {isLoggingIn ? 'Login...' : 'Login'}
        </button>
        {loginError && <p className="form-error">{loginError}</p>}
      </form>
    </section>
  )
}
