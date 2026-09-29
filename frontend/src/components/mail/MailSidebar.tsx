import { Link } from '@tanstack/react-router'
import { useEffect, type ReactNode } from 'react'
import { Archive, Inbox, Mail, PenLine, Star, Trash2 } from 'lucide-react'

import { listMessages } from '../../lib/api/client'
import type { MessageView } from '../../lib/api/types'
import { messageQueryKey } from '../../lib/hooks/useMessages'
import { queryClient } from '../../lib/queryClient'

import { cn } from '../../lib/utils'
import { Button } from '../ui/button'
import { Sidebar, SidebarNav, SidebarNavItem } from '../ui/sidebar'
import { useCompose } from './compose-context'

const navItems = [
  { to: '/mail', label: 'Inbox', view: 'inbox', icon: <Inbox className="h-4 w-4" /> },
  { to: '/mail/starred', label: 'Starred', view: 'starred', icon: <Star className="h-4 w-4" /> },
  { to: '/mail/archived', label: 'Archived', view: 'archived', icon: <Archive className="h-4 w-4" /> },
  { to: '/mail/deleted', label: 'Deleted', view: 'deleted', icon: <Trash2 className="h-4 w-4" /> },
] as const

export function MailSidebar() {
  const { openCompose } = useCompose()

  useEffect(() => {
    for (const item of navItems) {
      void queryClient.prefetchQuery({
        queryKey: messageQueryKey(item.view),
        queryFn: () => listMessages(item.view),
        staleTime: 30_000,
      })
    }
  }, [])

  return (
    <Sidebar className="hidden md:flex">
      <div className="mb-8 flex items-center gap-3 px-2">
        <div className="flex h-10 w-10 items-center justify-center rounded-2xl bg-slate-950 text-white">
          <Mail className="h-5 w-5" />
        </div>
        <div>
          <p className="text-xs font-extrabold uppercase tracking-widest text-indigo-600">TempMail</p>
          <p className="text-lg font-black tracking-tight">Mail</p>
        </div>
      </div>
      <Button type="button" className="mb-6 w-full" onClick={() => openCompose()}>
        <PenLine className="h-4 w-4" />
        Compose
      </Button>
      <SidebarNav>
        {navItems.map((item) => (
          <NavItem key={item.to} to={item.to} view={item.view} icon={item.icon} label={item.label} />
        ))}
      </SidebarNav>
    </Sidebar>
  )
}

function NavItem({ to, view, icon, label }: { to: (typeof navItems)[number]['to']; view: MessageView; icon: ReactNode; label: string }) {
  return (
    <Link
      to={to}
      activeOptions={{ exact: to === '/mail' }}
      onMouseEnter={() => {
        void queryClient.prefetchQuery({ queryKey: messageQueryKey(view), queryFn: () => listMessages(view), staleTime: 30_000 })
      }}
    >
      {({ isActive }) => (
        <SidebarNavItem className={cn('flex items-center gap-3', isActive && 'bg-slate-950 text-white')}>
          {icon}
          {label}
        </SidebarNavItem>
      )}
    </Link>
  )
}
