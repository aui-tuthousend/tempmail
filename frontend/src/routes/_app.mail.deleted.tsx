import { createFileRoute } from '@tanstack/react-router'

import { MailView } from './_app.mail'

export const Route = createFileRoute('/_app/mail/deleted')({
  component: DeletedPage,
})

function DeletedPage() {
  return <MailView view="deleted" title="Deleted" />
}
