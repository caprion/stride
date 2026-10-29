#!/usr/bin/env python3
"""Check that an APK is actually the app.

The APK is only the app if the Rust core is inside it. A build that quietly
loses the cross-compiled library still *succeeds* - the Gradle packaging
task skips what it cannot find - and ships a WebView with nothing behind it.
That happened once: the phone's crate became a member of the root workspace,
cargo-ndk built into the workspace's target directory instead of the
crate's own, and the published APK was 4 MB with no core in it.

    python3 scripts/check_apk.py path/to/app.apk
"""

from __future__ import annotations

import glob
import sys
import zipfile

REQUIRED = [
    "lib/arm64-v8a/libuniffi_stride.so",
    "lib/arm64-v8a/libjnidispatch.so",
    "assets/public/index.html",
    "assets/capacitor.config.json",
    "classes.dex",
]

# The core is at least this big. A stub that links nothing is far smaller.
MIN_CORE_BYTES = 1_000_000


def main(argv: list[str]) -> int:
    paths = argv[1:] or sorted(glob.glob("dist/*.apk")) or sorted(
        glob.glob("android/app/build/outputs/apk/release/*.apk")
    )
    if not paths:
        print("no APK to check", file=sys.stderr)
        return 2

    failures = 0
    for path in paths:
        with zipfile.ZipFile(path) as archive:
            names = archive.namelist()
            missing = [name for name in REQUIRED if name not in names]
            unpacked = sum(archive.getinfo(n).file_size for n in names)
            core = (
                archive.getinfo("lib/arm64-v8a/libuniffi_stride.so").file_size
                if "lib/arm64-v8a/libuniffi_stride.so" in names
                else 0
            )
            # A remote server URL means the app is a browser pointed
            # somewhere else - the whole point is that it is not.
            config = ""
            if "assets/capacitor.config.json" in names:
                config = archive.read("assets/capacitor.config.json").decode("utf-8", "replace")
            local_only = '"url"' not in config

        print(f"\n  {path}")
        print(f"    unpacked        {unpacked / 1048576:.1f} MB")
        print(f"    the Rust core   {core / 1048576:.2f} MB")
        print(f"    self-sufficient {'yes' if local_only else 'NO - it points at a server'}")

        if missing:
            print(f"    MISSING: {', '.join(missing)}")
            failures += 1
        if core < MIN_CORE_BYTES:
            print("    the Rust core is suspiciously small")
            failures += 1
        if not local_only:
            print("    the app is pointed at a remote server")
            failures += 1

    print()
    if failures:
        print("  the APK is not the app")
        return 1
    print("  the APK carries the Rust core and points at nothing")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
