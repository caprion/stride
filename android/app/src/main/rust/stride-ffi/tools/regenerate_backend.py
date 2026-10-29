#!/usr/bin/env python3
"""Regenerates the delegation in `src/backend.rs`.

The phone's backend implements `stride_api::backend::Backend`, and that trait
grows: every method the router needs is one the phone has to answer. Writing
the delegations by hand is fifty-odd copies of the same line, and a trait
that grows by one method is a compile error in a file of noise.

This script reads the trait out of `crates/stride-api/src/backend.rs` and
rewrites the block between the markers in `src/backend.rs`. The methods the
phone answers itself — the ones marked `KEEP` below — are left alone.

    python3 tools/regenerate_backend.py

Run it after the trait changes, and read the diff.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CRATE = HERE.parent
TRAIT = CRATE.parents[5] / "crates" / "stride-api" / "src" / "backend.rs"
TARGET = CRATE / "src" / "backend.rs"

BEGIN = "    // --- generated: the delegation to the shared stack ---"
END = "    // --- end of the generated delegation ---"

# The methods the phone answers itself. They are not delegated.
KEEP = {
    "pair_code",
    "pair",
    "devices",
    "revoke_device",
    "network_status",
    "public_identity",
    "exit_nodes",
    "set_network_route",
    "tile_policy",
    "tile",
    "load_history",
    "run_sync",
    # The phone answers the status itself: the state layer forgets the key
    # as soon as a pull is over, so the time of the last pull is read from
    # the sync log in the database.
    "public_status",
}


def trait_methods(source: str) -> list[tuple[str, str, str]]:
    """The trait's methods, as (name, signature, arguments) triples."""
    start = source.index("pub trait Backend: Send + Sync {")
    block = source[start:]
    methods: list[tuple[str, str, str]] = []
    for part in re.split(r"\n    fn ", block)[1:]:
        name = part.split("(")[0]
        open_paren = part.index("(")
        depth = 0
        close = None
        for index in range(open_paren, len(part)):
            if part[index] == "(":
                depth += 1
            elif part[index] == ")":
                depth -= 1
                if depth == 0:
                    close = index
                    break
        if close is None:
            continue
        depth = 0
        semi = None
        for index in range(close, len(part)):
            char = part[index]
            if char == "<":
                depth += 1
            elif char == ">":
                depth -= 1
            elif char == ";" and depth <= 0:
                semi = index
                break
        if semi is None:
            continue
        methods.append((name, part[: semi + 1].strip(), part[open_paren + 1 : close]))
    return methods


def split_arguments(text: str) -> list[str]:
    out: list[str] = []
    depth = 0
    current = ""
    for char in text:
        if char in "([<":
            depth += 1
        elif char in ")]>":
            depth -= 1
        if char == "," and depth == 0:
            out.append(current.strip())
            current = ""
        else:
            current += char
    if current.strip():
        out.append(current.strip())
    return out


def main() -> int:
    methods = trait_methods(TRAIT.read_text())
    body: list[str] = []
    for name, signature, raw_arguments in methods:
        if name in KEEP:
            continue
        arguments = split_arguments(raw_arguments)
        arguments = [a for a in arguments if a and not a.strip().startswith("&self")]
        names = [a.split(":")[0].strip() for a in arguments]
        arrow = signature.rindex("->")
        depth = 0
        semi = None
        for index in range(arrow + 2, len(signature)):
            char = signature[index]
            if char in "(<[":
                depth += 1
            elif char in ")>]":
                depth -= 1
            elif char == ";" and depth == 0:
                semi = index
                break
        returns = signature[arrow + 2 : semi].strip()
        body.append(
            "    fn {name}(&self{args}) -> {returns} {{\n"
            "        self.inner.{name}({call})\n"
            "    }}".format(
                name=name,
                args="".join(", " + a for a in arguments),
                returns=returns,
                call=", ".join(names),
            )
        )

    generated = "\n".join([BEGIN, *body, END])
    text = TARGET.read_text()
    if BEGIN not in text or END not in text:
        print("the markers are missing from src/backend.rs", file=sys.stderr)
        return 1
    head = text[: text.index(BEGIN)]
    tail = text[text.index(END) + len(END) :]
    TARGET.write_text(head + generated + tail)
    print(f"{len(methods)} methods in the trait, {len(body)} delegated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
