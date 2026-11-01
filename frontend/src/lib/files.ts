/**
 * Files, on their way to the api.
 *
 * A FIT file and a plan file both travel as base64 in the body of their
 * route. The encoding is the app's business — it is plumbing, not a
 * domain value — and the file itself is never written down anywhere.
 */

/**
 * A file, as base64.
 *
 * The bytes go through in one go, so a large file does not blow the
 * argument limit of `String.fromCharCode`.
 */
export async function fileAsBase64(file: File): Promise<string> {
  const bytes = new Uint8Array(await file.arrayBuffer())
  let binary = ''
  for (let index = 0; index < bytes.length; index += 1) {
    binary += String.fromCharCode(bytes[index] ?? 0)
  }
  return btoa(binary)
}
