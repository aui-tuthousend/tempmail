export const mailboxDomain = import.meta.env.VITE_MAILBOX_DOMAIN ?? ''

export function joinMailboxAddress(localPart: string) {
  const value = localPart.trim().toLowerCase().split('@')[0]
  return mailboxDomain ? `${value}@${mailboxDomain}` : value
}

export function isPasswordValid(password: string) {
  return password.length >= 8 && /[A-Z]/.test(password)
}
