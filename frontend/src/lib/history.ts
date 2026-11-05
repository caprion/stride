import { addDays, format, isValid, parseISO, startOfWeek } from 'date-fns'

export function validDate(value: string | null): string {
  if (!value || !/^\d{4}-\d{2}-\d{2}$/.test(value)) return ''
  const date = parseISO(value)
  return isValid(date) && format(date, 'yyyy-MM-dd') === value ? value : ''
}

export function shiftDate(date: string, days: number): string {
  return format(addDays(parseISO(date), days), 'yyyy-MM-dd')
}

export function weekDates(date: string): string[] {
  const first = startOfWeek(parseISO(date), { weekStartsOn: 1 })
  return Array.from({ length: 7 }, (_, index) => format(addDays(first, index), 'yyyy-MM-dd'))
}

export function runsPath(from: string, to = from): string {
  return `/runs?${new URLSearchParams({ from, to })}`
}

export function historyReturn(path: string | null): string {
  return path && /^\/runs(?:\?[^#]*)?$/.test(path) ? path : '/runs'
}
