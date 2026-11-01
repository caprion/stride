/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** The base every route is served under. */
  readonly VITE_API_BASE?: string
  /**
   * Set on a phone build: the native bridge is expected to be installed.
   * It never decides a layout — only which transport carries the data.
   */
  readonly VITE_NATIVE?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
