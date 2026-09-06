import { createFileRoute } from '@tanstack/react-router'

import { MailView } from './_app.mail'

export const Route = createFileRoute('/_app/mail/archived')({
  component: ArchivedPage,
})

function ArchivedPage() {
  return <MailView view="archived" title="Archived" />
}
