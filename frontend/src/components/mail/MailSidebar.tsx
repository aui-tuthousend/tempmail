import { Link } from '@tanstack/react-router'
import { Archive, Inbox, Mail, Star, Trash2 } from 'lucide-react'
import type { ReactNode } from 'react'

import { cn } from '../../lib/utils'
import { Sidebar, SidebarNav, SidebarNavItem } from '../ui/sidebar'

const navItems = [
  { to: '/mail', label: 'Inbox', icon: <Inbox className="h-4 w-4" /> },
  { to: '/mail/starred', label: 'Starred', icon: <Star className="h-4 w-4" /> },
  { to: '/mail/archived', label: 'Archived', icon: <Archive className="h-4 w-4" /> },
  { to: '/mail/deleted', label: 'Deleted', icon: <Trash2 className="h-4 w-4" /> },
] as const

export function MailSidebar() {
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
      <SidebarNav>
        {navItems.map((item) => (
          <NavItem key={item.to} to={item.to} icon={item.icon} label={item.label} />
        ))}
      </SidebarNav>
    </Sidebar>
  )
}

function NavItem({ to, icon, label }: { to: (typeof navItems)[number]['to']; icon: ReactNode; label: string }) {
  return (
    <Link to={to} activeOptions={{ exact: to === '/mail' }}>
      {({ isActive }) => (
        <SidebarNavItem className={cn('flex items-center gap-3', isActive && 'bg-slate-950 text-white')}>
          {icon}
          {label}
        </SidebarNavItem>
      )}
    </Link>
  )
}
