import { createContext, useContext, useMemo, useState } from 'react'
import type { ReactNode } from 'react'

type MailSearchContextValue = {
  search: string
  setSearch: (value: string) => void
}

const MailSearchContext = createContext<MailSearchContextValue | null>(null)

export function MailSearchProvider({ children }: { children: ReactNode }) {
  const [search, setSearch] = useState('')
  const value = useMemo(() => ({ search, setSearch }), [search])

  return <MailSearchContext.Provider value={value}>{children}</MailSearchContext.Provider>
}

export function useMailSearch() {
  const context = useContext(MailSearchContext)

  if (!context) {
    throw new Error('useMailSearch must be used inside MailSearchProvider')
  }

  return context
}
