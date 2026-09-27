export const mailboxDomain = import.meta.env.VITE_MAILBOX_DOMAIN ?? ''

export function normalizeLocalPart(value: string) {
  return value.trim().toLowerCase().split('@')[0]
}

export function joinMailboxAddress(localPart: string) {
  const value = normalizeLocalPart(localPart)
  return mailboxDomain ? `${value}@${mailboxDomain}` : value
}
