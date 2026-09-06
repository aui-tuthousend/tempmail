import type { HTMLAttributes } from 'react'

import { cn } from '../../lib/utils'

export function Sidebar({ className, ...props }: HTMLAttributes<HTMLElement>) {
  return <aside className={cn('flex w-64 shrink-0 flex-col border-r border-slate-200 bg-white/80 p-4', className)} {...props} />
}

export function SidebarNav({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return <nav className={cn('grid gap-1', className)} {...props} />
}

export function SidebarNavItem({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return <div className={cn('rounded-2xl px-3 py-2 text-sm font-medium text-slate-600', className)} {...props} />
}
