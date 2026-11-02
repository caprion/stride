/**
 * The settings' groups and rows.
 *
 * The Settings screen is a list of groups, and a group is a list of rows.
 * Nothing nests deeper than one level: at compact a group collapses, and
 * at wide the groups sit in two columns.
 *
 * The rows are chrome — a label, a value and the app's own words. The
 * values themselves are the core's or the api's, and they arrive
 * rendered.
 */

import { useState, type ReactNode } from 'react'
import { ChevronDown } from 'lucide-react'

/**
 * One group.
 *
 * A group is collapsible, because at compact a screen of six groups is a
 * screen of six walls. It is open by default: the athlete came here to
 * change something, and a closed group hides what they came for.
 */
export function SettingsGroup({
  title,
  children,
  testId,
  defaultOpen = true,
}: {
  title: string
  children: ReactNode
  testId?: string
  defaultOpen?: boolean
}) {
  const [open, setOpen] = useState(defaultOpen)

  return (
    <section className="card overflow-hidden" data-testid={testId}>
      <button
        type="button"
        className="tap flex w-full items-center justify-between gap-3 px-4 py-3 text-left"
        aria-expanded={open}
        onClick={() => setOpen((value) => !value)}
      >
        <span className="text-[13px] font-semibold uppercase tracking-[0.06em] text-ink-muted">
          {title}
        </span>
        <ChevronDown
          size={17}
          strokeWidth={2}
          aria-hidden="true"
          className={`shrink-0 text-ink-faint transition-transform ${open ? 'rotate-180' : ''}`}
        />
      </button>
      {open ? <div className="border-t border-edge px-4 py-3">{children}</div> : null}
    </section>
  )
}

/**
 * One row of a group.
 *
 * A label on the left, a value on the right — or the control the row
 * carries, when the row is something the athlete changes.
 */
export function SettingRow({
  label,
  value,
  children,
  testId,
}: {
  label: string
  value?: ReactNode
  children?: ReactNode
  testId?: string
}) {
  return (
    <div
      className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 border-b border-edge py-2.5 last:border-b-0 last:pb-0"
      data-testid={testId}
    >
      <span className="text-[13px] text-ink-muted">{label}</span>
      {value !== undefined ? (
        <span className="text-[14px] font-medium text-ink">{value}</span>
      ) : null}
      {children}
    </div>
  )
}

/**
 * A line that says what just happened.
 *
 * A fact about the app's state, in the app's voice. It is never a
 * congratulation and never a warning.
 */
export function Notice({ text, testId }: { text: string | null; testId?: string }) {
  if (!text) return null
  return (
    <p className="mt-2 text-[13px] text-ink-muted" data-testid={testId}>
      {text}
    </p>
  )
}

/**
 * A text field.
 *
 * The label sits above the field, and the field is the app's own — the
 * athlete's own words go in it, and nothing else is stored with them.
 */
export function Field({
  label,
  value,
  onChange,
  placeholder,
  type = 'text',
  inputMode,
  testId,
}: {
  label: string
  value: string
  onChange: (value: string) => void
  placeholder?: string
  type?: string
  inputMode?: 'text' | 'numeric' | 'decimal'
  testId?: string
}) {
  return (
    <label className="block">
      <span className="block text-[12px] font-semibold uppercase tracking-[0.06em] text-ink-faint">
        {label}
      </span>
      <input
        type={type}
        inputMode={inputMode}
        value={value}
        placeholder={placeholder}
        onChange={(event) => onChange(event.target.value)}
        data-testid={testId}
        className="tap mt-1 w-full rounded-md border border-edge bg-surface px-3 text-[15px] text-ink outline-none focus:border-brand-400"
      />
    </label>
  )
}

/**
 * A choice, from a closed set.
 *
 * The options are the athlete's own words for the thing — "travel",
 * "sick", "injured" — and the app never writes one of them for them.
 */
export function Choice({
  label,
  value,
  options,
  onChange,
  testId,
}: {
  label: string
  value: string
  options: { value: string; label: string }[]
  onChange: (value: string) => void
  testId?: string
}) {
  return (
    <label className="block">
      <span className="block text-[12px] font-semibold uppercase tracking-[0.06em] text-ink-faint">
        {label}
      </span>
      <select
        value={value}
        onChange={(event) => onChange(event.target.value)}
        data-testid={testId}
        className="tap mt-1 w-full rounded-md border border-edge bg-surface px-3 text-[15px] text-ink outline-none focus:border-brand-400"
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  )
}
