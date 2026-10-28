import type { CapacitorConfig } from '@capacitor/cli'

/**
 * The app is Android-first and local-first.
 *
 * **The server URL is empty by default.** The phone is the whole thing: the
 * Garmin credential, the encrypted database and the analysis all live on the
 * device, and the web bundle that draws them is packaged in the APK. The
 * WebView loads `file:///android_asset/public` and the phone talks to no
 * server at all.
 *
 * The URL that used to be here — the instance on the tailnet — was the
 * interim build, where the phone was a browser for a desktop. It is gone:
 *
 * * `STRIDE_SERVER_URL` still points the build at an instance, for a build
 *   that wants the old behaviour (a development build against a running
 *   instance);
 * * the *runtime* URL lives in the native shell's settings, is empty by
 *   default, and is applied by `MainActivity` when it is set.
 *
 * When a URL is set it has to be HTTPS. The build refuses to produce an APK
 * that points at a cleartext origin, because a mixed-content or cleartext
 * WebView is how a device credential ends up on the wire in the clear.
 */

const serverUrl = process.env.STRIDE_SERVER_URL ?? ''

if (serverUrl && !/^https:\/\//i.test(serverUrl)) {
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
    // No URL: the bundled app, on the device.
    ...(serverUrl ? { url: serverUrl } : {}),
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
