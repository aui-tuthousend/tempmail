import { createFileRoute } from '@tanstack/react-router'

import { MailView } from './_app.mail'

export const Route = createFileRoute('/_app/mail/starred')({
  component: StarredPage,
})

function StarredPage() {
  return <MailView view="starred" title="Starred" />
}
