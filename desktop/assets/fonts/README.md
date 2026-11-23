# Inter — the app's own copy

The app is set in **Inter**, the mock's face, and it carries its own copy
of it. Nothing here is fetched from a CDN: the desktop app is a
self-hosted program and the phone is a webview, and both must draw the
same face with no network at all.

## What is here

Five weights, one file each, in the Web Open Font Format:

| File | Weight |
|------|--------|
| `Inter-Regular.woff2` | 400 |
| `Inter-Medium.woff2` | 500 |
| `Inter-SemiBold.woff2` | 600 |
| `Inter-Bold.woff2` | 700 |
| `Inter-ExtraBold.woff2` | 800 |

They are loaded by the app's own CSS (`frontend/src/index.css`) with
`@font-face`, and they are copied into the bundle by the build, so the
desktop binary and the phone's webview both get them from the bundle
they already carry.

## Where they came from

The official [Inter](https://github.com/rsms/inter) release, version 4.1
(`Inter-4.1.zip`, the `web/` directory), subset to Latin, Latin-Extended
and the Greek delta the app writes, and renamed so that all five weights
are one family. The subsetting changed nothing but the glyphs that were
never used.

## The licence

Inter is licensed under the **SIL Open Font License, Version 1.1**. The
licence is in `OFL.txt`, beside these files, and it applies to them.
