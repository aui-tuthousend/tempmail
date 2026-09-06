import { createFileRoute } from '@tanstack/react-router'

import { RegisterForm } from '../components/auth/RegisterForm'

export const Route = createFileRoute('/register')({
  component: RegisterPage,
})

function RegisterPage() {
  return (
    <main className="flex min-h-screen items-center justify-center bg-slate-100 px-4 py-10">
      <RegisterForm />
    </main>
  )
}
