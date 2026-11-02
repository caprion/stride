/**
 * The brand.
 *
 * One mark, one word. It is in the shell's chrome, and it is on the
 * first-run page — which stands alone, and has to say what it is.
 */

import { Activity } from 'lucide-react'

export function Brand() {
  return (
    <span className="flex items-center gap-2">
      <span className="flex h-7 w-7 items-center justify-center rounded-md bg-brand-700 text-accent-400">
        <Activity size={16} strokeWidth={2.6} />
      </span>
      <span className="text-[18px] font-extrabold uppercase tracking-tight text-brand-600">
        Stride
      </span>
    </span>
  )
}
