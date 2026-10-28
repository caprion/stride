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
 */
const config: CapacitorConfig = {
  appId: 'app.stride.running',
  appName: 'Stride',
  webDir: 'frontend/dist',
  server: {
    url: process.env.STRIDE_SERVER_URL ?? 'https://cf-openclaw.tail62fa14.ts.net:8443',
    cleartext: false,
    androidScheme: 'https',
  },
  android: {
    allowMixedContent: false,
    // The WebView is the app's only surface. No remote content, ever.
    webContentsDebuggingEnabled: false,
  },
}

export default config
