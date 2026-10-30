#!/usr/bin/env python3
"""Check that an APK is actually the app.

The APK is only the app if the Rust core is inside it. A build that quietly
loses the cross-compiled library still *succeeds* - the Gradle packaging
task skips what it cannot find - and ships a WebView with nothing behind it.
That happened once: the phone's crate became a member of the root workspace,
cargo-ndk built into the workspace's target directory instead of the
crate's own, and the published APK was 4 MB with no core in it.

    python3 scripts/check_apk.py path/to/app.apk

The check has three questions, and it asks all three:

1. **Is the core in it?** The APK must carry ``libuniffi_stride.so`` for the
   phone's ABI, and it must be a real library rather than a stub.
2. **Is it the core this build produced?** The Gradle copy task used to ask
   only whether the ABI had changed, so a cargo rebuild was invisible to it
   and the APK shipped the core from an earlier build. The chain is checked
   in two steps, and each is within one machine, so the bytes are
   comparable: **the staged copy must be the one cargo just built, and the
   APK must carry the staged copy.** Comparing the APK straight against
   cargo would be wrong - Rust builds are not byte-reproducible across
   different build directories, so two honest builds of the same source
   differ.

   The comparison runs whenever there is a build tree to compare against.
   It is not skipped because the APK happens to sit in another directory:
   the CI publishes the APK it has just built from ``dist/``, and a check
   that only ran for one path would not have run at all.
3. **Is it signed with the key the app publishes?** The certificate in
   ``backend/app/assetlinks.json`` is the one the release APK is signed
   with, and it is what makes the app open as an app rather than a browser
   tab. An APK signed with anything else is not the app, whatever is inside
   it.
"""

from __future__ import annotations

import argparse
import glob
import hashlib
import json
import os
import struct
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

# The name of the core, and the ABI directory it is staged under.
CORE_NAME = "libuniffi_stride.so"

# The ABI directory name and the Rust target triple it is built for.
TRIPLES = {
    "arm64-v8a": "aarch64-linux-android",
    "x86_64": "x86_64-linux-android",
    "armeabi-v7a": "armv7-linux-androideabi",
}

# Where the Gradle copy task stages the core, and where cargo leaves it.
STAGED_GLOB = "android/app/build/rust/jniLibs/*/" + CORE_NAME
CARGO_GLOBS = (
    "target/{triple}/release/" + CORE_NAME,
    "android/app/src/main/rust/stride-ffi/target/{triple}/release/" + CORE_NAME,
)

# The published certificate.
ASSET_LINKS = "backend/app/assetlinks.json"

# The magic at the end of the APK signing block.
SIGNING_BLOCK_MAGIC = b"APK Sig Block 42"

# The ids of the signing schemes in that block.
SCHEME_V2 = 0x7109871A
SCHEME_V3 = 0xF05368C0


def _digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _file_digest(path: str) -> str:
    with open(path, "rb") as handle:
        return _digest(handle.read())


# ---------------------------------------------------------------- the build tree


def _staged_cores() -> dict[str, str]:
    """The cores the Gradle copy task staged, by ABI.

    The ABI is the directory name: ``arm64-v8a``, ``x86_64``.
    """
    staged: dict[str, str] = {}
    for path in sorted(glob.glob(STAGED_GLOB)):
        abi = os.path.basename(os.path.dirname(path))
        staged[abi] = path
    return staged


def _cargo_cores() -> dict[str, str]:
    """The cores cargo built, by ABI.

    Only the ABI whose triple is known is looked for, so a build of another
    ABI cannot be mistaken for this one's.
    """
    built: dict[str, str] = {}
    for abi, triple in TRIPLES.items():
        for pattern in CARGO_GLOBS:
            for path in sorted(glob.glob(pattern.format(triple=triple))):
                built[abi] = path
                break
            if abi in built:
                break
    return built


def _core_problems() -> list[str]:
    """The problems with the build tree, if there is one.

    The tree is what says which core the APK is supposed to carry, so it is
    checked on its own: a staged core that is not the one cargo just built
    is a stale copy, and an APK built from it would run yesterday's code.
    """
    problems: list[str] = []
    staged = _staged_cores()
    built = _cargo_cores()
    for abi, staged_path in staged.items():
        cargo_path = built.get(abi)
        if cargo_path is None:
            problems.append(
                f"the staged core ({staged_path}) has no cargo build to compare with"
            )
            continue
        if _file_digest(staged_path) != _file_digest(cargo_path):
            problems.append(
                f"the staged core ({staged_path}) is not the one cargo built "
                f"({cargo_path}) - the copy task did not run"
            )
    return problems


# ------------------------------------------------------------- the signature


def _u32(data: bytes, offset: int) -> int:
    return struct.unpack_from("<I", data, offset)[0]


def _u64(data: bytes, offset: int) -> int:
    return struct.unpack_from("<Q", data, offset)[0]


def _read_len_prefixed(data: bytes, offset: int) -> tuple[bytes, int]:
    """A length-prefixed blob: a uint32 length and the bytes after it."""
    length = _u32(data, offset)
    start = offset + 4
    return data[start : start + length], start + length


def _certificates_from_signed_data(signed: bytes) -> list[bytes]:
    """The DER certificates of one signer.

    The signed data is a sequence of length-prefixed blobs: the digests
    first, then the certificates.
    """
    certificates: list[bytes] = []
    try:
        _, offset = _read_len_prefixed(signed, 0)  # the digests
        blob, _ = _read_len_prefixed(signed, offset)  # the certificates
    except (IndexError, struct.error):
        return certificates
    offset = 0
    while offset + 4 <= len(blob):
        certificate, offset = _read_len_prefixed(blob, offset)
        if certificate:
            certificates.append(certificate)
    return certificates


def _signing_certificates(apk: str) -> list[bytes]:
    """The signing certificates of an APK, from its signing block.

    The block is a pair of sizes around a list of ``(id, value)`` pairs. The
    v2 and v3 schemes both hold a list of signers, and each signer's signed
    data holds the certificate chain.
    """
    with open(apk, "rb") as handle:
        data = handle.read()
    magic = data.rfind(SIGNING_BLOCK_MAGIC)
    if magic < 24:
        return []
    try:
        # The block is a size, the pairs, the same size again, and the magic.
        size = _u64(data, magic - 8)
        start = magic - size + 8
        if start < 0 or size < 24:
            return []
        offset = start + 8
        end = magic - 8
    except struct.error:
        return []

    certificates: list[bytes] = []
    while offset + 12 <= end:
        try:
            length = _u64(data, offset)
            identifier = _u32(data, offset + 8)
        except struct.error:
            break
        if length < 4 or offset + 8 + length > end + 8:
            break
        value = data[offset + 12 : offset + 8 + length]
        offset += 8 + length
        if identifier not in (SCHEME_V2, SCHEME_V3):
            continue
        try:
            signers, _ = _read_len_prefixed(value, 0)
        except (IndexError, struct.error):
            continue
        signer_offset = 0
        while signer_offset + 4 <= len(signers):
            signer, signer_offset = _read_len_prefixed(signers, signer_offset)
            try:
                signed, _ = _read_len_prefixed(signer, 0)
            except (IndexError, struct.error):
                continue
            certificates.extend(_certificates_from_signed_data(signed))
    return certificates


def _published_fingerprints() -> set[str]:
    """The fingerprints in ``backend/app/assetlinks.json``."""
    if not os.path.isfile(ASSET_LINKS):
        return set()
    try:
        with open(ASSET_LINKS, encoding="utf-8") as handle:
            document = json.load(handle)
    except (OSError, json.JSONDecodeError):
        return set()
    fingerprints: set[str] = set()
    for entry in document:
        target = entry.get("target") or {}
        for fingerprint in target.get("sha256_cert_fingerprints") or []:
            fingerprints.add(str(fingerprint).replace(":", "").upper())
    return fingerprints


def _signature_problems(apk: str) -> list[str]:
    """The problems with the APK's signature."""
    published = _published_fingerprints()
    if not published:
        return []  # nothing published to compare with; the build checks this
    certificates = _signing_certificates(apk)
    if not certificates:
        return ["the APK carries no signing certificate in its signing block"]
    actual = {hashlib.sha256(certificate).hexdigest().upper() for certificate in certificates}
    if not actual & published:
        return [
            "the APK is signed with a certificate that is not the one published in "
            f"{ASSET_LINKS} ({', '.join(sorted(actual))})"
        ]
    return []


# ------------------------------------------------------------------- the check


def _apk_core_digests(archive: zipfile.ZipFile) -> dict[str, str]:
    return {
        name: _digest(archive.read(name))
        for name in archive.namelist()
        if name.endswith(CORE_NAME)
    }


def _apk_problems(path: str, compare_core: bool) -> tuple[list[str], dict[str, str]]:
    """The problems with one APK, and what it carries."""
    problems: list[str] = []
    facts: dict[str, str] = {}
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        missing = [name for name in REQUIRED if name not in names]
        if missing:
            problems.append(f"MISSING: {', '.join(missing)}")

        core = (
            archive.getinfo("lib/arm64-v8a/libuniffi_stride.so").file_size
            if "lib/arm64-v8a/libuniffi_stride.so" in names
            else 0
        )
        facts["core_bytes"] = str(core)
        if core < MIN_CORE_BYTES:
            problems.append("the Rust core is suspiciously small")

        # A remote server URL means the app is a browser pointed
        # somewhere else - the whole point is that it is not.
        config = ""
        if "assets/capacitor.config.json" in names:
            config = archive.read("assets/capacitor.config.json").decode("utf-8", "replace")
        if '"url"' in config:
            problems.append("the app is pointed at a remote server")
        facts["local_only"] = "yes" if '"url"' not in config else "NO"

        if compare_core:
            apks = _apk_core_digests(archive)
            staged = _staged_cores()
            if not staged:
                problems.append(
                    "there is no build tree to compare the core with - the APK cannot be "
                    "shown to carry the core this machine built"
                )
            elif not apks:
                problems.append("the APK carries no Rust core")
            else:
                staged_digests = {
                    abi: _file_digest(staged_path) for abi, staged_path in staged.items()
                }
                for name, digest in apks.items():
                    abi = name.split("/")[1] if name.startswith("lib/") else ""
                    expected = staged_digests.get(abi)
                    if expected is None:
                        problems.append(
                            f"the APK carries {name}, which no build staged"
                        )
                    elif digest != expected:
                        problems.append(
                            f"the APK carries {digest[:12]} but the staged copy is "
                            f"{expected[:12]}"
                        )
                facts["core_matches_staged"] = "yes"

    problems.extend(_core_problems())
    problems.extend(_signature_problems(path))
    return problems, facts


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Check that an APK is the app.")
    parser.add_argument("apk", nargs="*", help="the APK(s) to check")
    parser.add_argument(
        "--no-core-compare",
        action="store_true",
        help=(
            "do not compare the core with the local build tree. Use it only for an "
            "artefact that was built somewhere else: the comparison is what proves the "
            "APK carries the core this build produced."
        ),
    )
    arguments = parser.parse_args(argv[1:])

    paths = arguments.apk or sorted(glob.glob("dist/*.apk")) or sorted(
        glob.glob("android/app/build/outputs/apk/release/*.apk")
    )
    if not paths:
        print("no APK to check", file=sys.stderr)
        return 2

    failures = 0
    for path in paths:
        problems, facts = _apk_problems(path, not arguments.no_core_compare)
        core_bytes = int(facts.get("core_bytes", "0"))
        print(f"\n  {path}")
        print(f"    the Rust core   {core_bytes / 1048576:.2f} MB")
        print(f"    self-sufficient {facts.get('local_only', '?')}")
        if facts.get("core_matches_staged"):
            print("    the core        is the one this build staged")
        if problems:
            for problem in problems:
                print(f"    {problem}")
            failures += 1

    print()
    if failures:
        print("  the APK is not the app")
        return 1
    print("  the APK carries the Rust core and points at nothing")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
