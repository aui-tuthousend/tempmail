import { createContext, useContext, useMemo, useState, type ReactNode } from 'react'

import { ComposeDialog, type ComposeDraft } from './ComposeDialog'

type ComposeContextValue = {
  openCompose: (draft?: ComposeDraft) => void
}

const ComposeContext = createContext<ComposeContextValue | null>(null)

export function ComposeProvider({ children }: { children: ReactNode }) {
  const [open, setOpen] = useState(false)
  const [draft, setDraft] = useState<ComposeDraft | undefined>()

  const value = useMemo(
    () => ({
      openCompose: (nextDraft?: ComposeDraft) => {
        setDraft(nextDraft)
        setOpen(true)
      },
    }),
    [],
  )

  return (
    <ComposeContext.Provider value={value}>
      {children}
      <ComposeDialog draft={draft} open={open} onOpenChange={setOpen} />
    </ComposeContext.Provider>
  )
}

export function useCompose() {
  const context = useContext(ComposeContext)

  if (!context) {
    throw new Error('useCompose must be used within ComposeProvider')
  }

  return context
}
