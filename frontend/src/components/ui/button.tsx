import { Slot } from '@radix-ui/react-slot'
import type { ButtonHTMLAttributes } from 'react'

import { cn } from '../../lib/utils'

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  asChild?: boolean
  variant?: 'default' | 'secondary' | 'ghost' | 'destructive'
  size?: 'default' | 'sm' | 'icon'
}

const variants = {
  default: 'bg-slate-950 text-white hover:bg-slate-800',
  secondary: 'bg-indigo-50 text-indigo-700 hover:bg-indigo-100',
  ghost: 'bg-transparent text-slate-700 hover:bg-slate-100',
  destructive: 'bg-red-50 text-red-700 hover:bg-red-100',
}

const sizes = {
  default: 'h-11 px-5 py-2',
  sm: 'h-9 px-3 text-sm',
  icon: 'h-10 w-10',
}

export function Button({ asChild, className, variant = 'default', size = 'default', ...props }: ButtonProps) {
  const Comp = asChild ? Slot : 'button'

  return (
    <Comp
      className={cn(
        'inline-flex items-center justify-center gap-2 rounded-full font-semibold transition disabled:pointer-events-none disabled:opacity-60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500',
        variants[variant],
        sizes[size],
        className,
      )}
      {...props}
    />
  )
}
