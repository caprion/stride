import type { CapacitorConfig } from '@capacitor/cli'

/**
 * Capacitor configuration.
 *
 * The app is Android-first and local-first. The web bundle is the whole UI;
 * the Garmin integration, the database and the analysis live on the machine
 * the athlete trusts — for now the instance on the tailnet.
 *
 * `server.url` points the WebView at that instance. When the Kotlin core
 * lands, this becomes a bundled UI with a native bridge, and the URL goes
 * away entirely: the phone would then own the Garmin key and the database,
 * and need no server at all.
 *
 * The WebView is pointed at a *remote* origin, which is the whole attack
 * surface of this configuration: whatever answers on that URL is inside the
 * app's WebView, with the Capacitor bridge injected into it and the device
 * cookie for that origin in the WebView's jar. So the URL has to be HTTPS —
 * the build refuses to produce an APK that points at a cleartext origin,
 * because a mixed-content or cleartext WebView is how a device credential
 * ends up on the wire in the clear.
 */

const DEFAULT_SERVER_URL = 'https://cf-openclaw.tail62fa14.ts.net:8443'
const serverUrl = process.env.STRIDE_SERVER_URL ?? DEFAULT_SERVER_URL

if (!/^https:\/\//i.test(serverUrl)) {
  // Fail the build rather than ship an APK whose WebView talks cleartext.
  throw new Error(
    `STRIDE_SERVER_URL must be an https:// URL (got "${serverUrl}"). ` +
      'The WebView carries the paired-device credential; it must not travel over plain HTTP.',
  )
}

const config: CapacitorConfig = {
  appId: 'app.stride.running',
  appName: 'Stride',
  webDir: 'frontend/dist',
  server: {
    url: serverUrl,
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
