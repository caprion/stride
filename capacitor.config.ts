import type { CapacitorConfig } from '@capacitor/cli'

/**
 * The app is Android-first and local-first.
 *
 * **There is no server here.** The phone is the whole thing: the Garmin
 * credential, the encrypted database and the analysis all live on the
 * device, and the web bundle that draws them is packaged in the APK. The
 * WebView loads the bundled assets and the phone talks to no server at all.
 *
 * The URL is a *runtime* setting, not a build setting. It lives in the
 * native shell's settings (`AppSettings.serverUrl`), it is **empty by
 * default**, and `MainActivity` applies it when — and only when — it is
 * not empty. An empty URL means the bundled app, which is the shipped
 * behaviour.
 *
 * So this file points at nothing, and it has no environment variable to
 * point it at one: a build that named a server would put a fixed origin
 * into every APK, and the origin is the operator's business, not the
 * repository's.
 *
 * When the runtime URL *is* set it has to be HTTPS. The WebView carries
 * the paired-device credential, and a cleartext or mixed-content WebView is
 * how that credential ends up on the wire in the clear.
 */

const config: CapacitorConfig = {
  appId: 'app.stride.running',
  appName: 'Stride',
  webDir: 'frontend/dist',
  server: {
    // No URL. The bundled app, on the device.
    cleartext: false,
    androidScheme: 'https',
  },
  android: {
    allowMixedContent: false,
    // The WebView is the app's only surface. No remote content, ever.
    // Capacitor reads this from the generated assets/capacitor.config.json;
    // a release build is not debuggable, so this is the last word on whether
    // `chrome://inspect` can attach to the WebView.
    webContentsDebuggingEnabled: false,
  },
}

export default config
