/** Core UI kit. Every surface in the app is built from these primitives. */

import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'
import type { ReactNode } from 'react'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/* ------------------------------------------------------------------ card */

export function Card({
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        'rounded-card border border-edge bg-surface shadow-card',
        className,
      )}
      {...props}
    >
      {children}
    </div>
  )
}

export function CardHeader({
  title,
  subtitle,
  action,
  className,
}: {
  title: ReactNode
  subtitle?: ReactNode
  action?: ReactNode
  className?: string
}) {
  return (
    <div className={cn('flex items-start justify-between gap-4 px-5 pt-4 pb-3', className)}>
      <div className="min-w-0">
        <h3 className="text-[15px] font-semibold leading-tight text-ink">{title}</h3>
        {subtitle ? <p className="mt-0.5 text-[13px] text-ink-muted">{subtitle}</p> : null}
      </div>
      {action ? <div className="shrink-0">{action}</div> : null}
    </div>
  )
}

/* ------------------------------------------------------------- stat tile */

export function StatTile({
  label,
  value,
  unit,
  hint,
  icon,
  tone = 'default',
  className,
}: {
  label: string
  value: ReactNode
  unit?: string
  hint?: ReactNode
  icon?: ReactNode
  tone?: 'default' | 'brand' | 'positive' | 'negative'
  className?: string
}) {
  return (
    <div className={cn('flex flex-col gap-0.5', className)}>
      <span className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-[0.07em] text-ink-muted">
        {icon}
        {label}
      </span>
      <span
        className={cn(
          'text-[26px] font-semibold leading-none tracking-tight',
          tone === 'brand' && 'text-brand-600',
          tone === 'positive' && 'text-positive',
          tone === 'negative' && 'text-negative',
          tone === 'default' && 'text-ink',
        )}
      >
        {value}
        {unit ? <span className="ml-1 text-[14px] font-medium text-ink-muted">{unit}</span> : null}
      </span>
      {hint ? <span className="text-[12px] text-ink-muted">{hint}</span> : null}
    </div>
  )
}

/* ---------------------------------------------------------------- button */

type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger'

export function Button({
  variant = 'primary',
  size = 'md',
  className,
  children,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ButtonVariant
  size?: 'sm' | 'md' | 'lg'
}) {
  return (
    <button
      className={cn(
        'inline-flex items-center justify-center gap-1.5 rounded-md font-semibold transition-colors',
        'focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-500',
        'disabled:cursor-not-allowed disabled:opacity-50',
        size === 'sm' && 'h-8 px-3 text-[13px]',
        size === 'md' && 'h-9 px-4 text-[14px]',
        size === 'lg' && 'h-11 px-6 text-[15px]',
        variant === 'primary' && 'bg-brand-500 text-white hover:bg-brand-600 active:bg-brand-700',
        variant === 'secondary' &&
          'border border-edge bg-surface text-ink hover:bg-surface-soft active:bg-surface-sunken',
        variant === 'ghost' && 'text-ink-soft hover:bg-surface-soft',
        variant === 'danger' && 'bg-negative text-white hover:brightness-95',
        className,
      )}
      {...props}
    >
      {children}
    </button>
  )
}

/* ----------------------------------------------------------------- badge */

export function Badge({
  children,
  tone = 'neutral',
  className,
}: {
  children: ReactNode
  tone?: 'neutral' | 'brand' | 'positive' | 'warn' | 'negative' | 'info'
  className?: string
}) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-semibold uppercase tracking-wide',
        tone === 'neutral' && 'bg-surface-sunken text-ink-soft',
        tone === 'brand' && 'bg-brand-50 text-brand-700',
        tone === 'positive' && 'bg-emerald-50 text-positive',
        tone === 'warn' && 'bg-amber-50 text-warn',
        tone === 'negative' && 'bg-red-50 text-negative',
        tone === 'info' && 'bg-sky-50 text-sky-700',
        className,
      )}
    >
      {children}
    </span>
  )
}

/* -------------------------------------------------------------- skeleton */

export function Skeleton({ className }: { className?: string }) {
  return (
    <div className={cn('relative overflow-hidden rounded bg-surface-sunken', className)}>
      <div className="absolute inset-0 -translate-x-full animate-shimmer bg-gradient-to-r from-transparent via-white/70 to-transparent" />
    </div>
  )
}

/* ------------------------------------------------------------ empty state */

export function EmptyState({
  icon,
  title,
  description,
  action,
}: {
  icon?: ReactNode
  title: string
  description?: string
  action?: ReactNode
}) {
  return (
    <div className="flex flex-col items-center justify-center gap-2 px-6 py-14 text-center">
      {icon ? <div className="mb-1 text-ink-faint">{icon}</div> : null}
      <p className="text-[15px] font-semibold text-ink">{title}</p>
      {description ? <p className="max-w-sm text-[13px] text-ink-muted">{description}</p> : null}
      {action ? <div className="mt-3">{action}</div> : null}
    </div>
  )
}

/* ------------------------------------------------------------- page head */

export function PageHeader({
  title,
  description,
  actions,
  className,
}: {
  title: ReactNode
  description?: ReactNode
  actions?: ReactNode
  className?: string
}) {
  return (
    <div className={cn('mb-5 flex flex-wrap items-end justify-between gap-3', className)}>
      <div className="min-w-0">
        <h1 className="text-[26px] font-bold leading-tight tracking-tight text-ink">{title}</h1>
        {description ? (
          <p className="mt-1 text-[14px] text-ink-muted">{description}</p>
        ) : null}
      </div>
      {actions ? <div className="flex shrink-0 items-center gap-2">{actions}</div> : null}
    </div>
  )
}

/* --------------------------------------------------------------- toolbar */

export function Toolbar({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <div className={cn('flex flex-wrap items-center gap-2', className)}>{children}</div>
  )
}

export function SegmentedControl<T extends string | number>({
  options,
  value,
  onChange,
  className,
}: {
  options: { label: string; value: T }[]
  value: T
  onChange: (value: T) => void
  className?: string
}) {
  return (
    <div className={cn('inline-flex rounded-md border border-edge bg-surface p-0.5', className)}>
      {options.map((option) => (
        <button
          key={String(option.value)}
          onClick={() => onChange(option.value)}
          className={cn(
            'rounded px-3 py-1 text-[13px] font-medium transition-colors',
            option.value === value
              ? 'bg-ink text-white'
              : 'text-ink-muted hover:bg-surface-soft hover:text-ink',
          )}
        >
          {option.label}
        </button>
      ))}
    </div>
  )
}

export function Toggle({
  checked,
  onChange,
  label,
  description,
}: {
  checked: boolean
  onChange: (value: boolean) => void
  label: string
  description?: string
}) {
  return (
    <label className="flex cursor-pointer items-start justify-between gap-4">
      <span>
        <span className="block text-[14px] font-medium text-ink">{label}</span>
        {description ? <span className="mt-0.5 block text-[13px] text-ink-muted">{description}</span> : null}
      </span>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        onClick={() => onChange(!checked)}
        className={cn(
          'relative mt-0.5 h-6 w-11 shrink-0 rounded-full transition-colors',
          checked ? 'bg-brand-500' : 'bg-surface-sunken',
        )}
      >
        <span
          className={cn(
            'absolute top-0.5 h-5 w-5 rounded-full bg-white shadow transition-transform',
            checked ? 'translate-x-[22px]' : 'translate-x-0.5',
          )}
        />
      </button>
    </label>
  )
}

export function Field({
  label,
  hint,
  children,
  className,
}: {
  label: string
  hint?: ReactNode
  children: ReactNode
  className?: string
}) {
  return (
    <label className={cn('block', className)}>
      <span className="mb-1.5 block text-[13px] font-semibold text-ink-soft">{label}</span>
      {children}
      {hint ? <span className="mt-1.5 block text-[12px] text-ink-muted">{hint}</span> : null}
    </label>
  )
}

export const inputClass =
  'w-full rounded-md border border-edge bg-surface px-3 py-2 text-[14px] text-ink outline-none transition-colors placeholder:text-ink-faint focus:border-brand-400 focus:ring-2 focus:ring-brand-100'

export function Divider({ className }: { className?: string }) {
  return <div className={cn('h-px w-full bg-edge', className)} />
}
