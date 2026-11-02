/**
 * Handing a file to the host.
 *
 * The app makes the file; the host delivers it. On the desktop that is a
 * download; on the phone it is the native share sheet, which the bridge
 * does not have yet — so the app only offers a file where it can deliver
 * one, and a control that promises a thing and does nothing is worse
 * than an absent one.
 */

import { currentHost } from './transport'

/**
 * Whether this host can take a file from the app.
 *
 * The desktop downloads it. The phone hands it to the native share
 * sheet, and that sheet is not there yet.
 */
export function canDeliverFile(): boolean {
  return currentHost() === 'http'
}

/** The name of a downloaded file. */
export function fileName(name: string, extension: string): string {
  return `${name}.${extension}`
}

/**
 * Hand a file to the host: a download, here.
 *
 * The file is made in the app's own memory and goes straight to the
 * athlete's machine — it is never uploaded, and it never leaves the
 * device.
 */
export function downloadFile(name: string, content: BlobPart, type: string): void {
  const blob = new Blob([content], { type })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = name
  anchor.rel = 'noopener'
  document.body.appendChild(anchor)
  anchor.click()
  anchor.remove()
  URL.revokeObjectURL(url)
}
