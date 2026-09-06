import { createFileRoute } from '@tanstack/react-router'

import { MailView } from './_app.mail'

export const Route = createFileRoute('/_app/mail/')({
  component: MailIndexPage,
})

function MailIndexPage() {
  return <MailView view="inbox" title="Inbox" />
}
