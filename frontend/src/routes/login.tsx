import { createFileRoute } from '@tanstack/react-router'

import { LoginForm } from '../components/auth/LoginForm'

export const Route = createFileRoute('/login')({
  component: LoginPage,
})

function LoginPage() {
  return (
    <main className="flex min-h-screen items-center justify-center bg-slate-100 px-4 py-10">
      <LoginForm />
    </main>
  )
}
