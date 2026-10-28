"""A minimal, deterministic FIT *writer*.

The reference implementation only ever reads FIT files, so the golden harness
needs a way to produce them. Writing them by hand as bytes is how you get
mysterious fixtures; writing them through the FIT profile that the decoder
itself uses (``fitparse.profile``) means the fixture is a real FIT file that a
real Garmin device could plausibly have produced.

Everything here is deterministic: no timestamps from the clock, no dict
ordering surprises, no randomness. The same case description always produces
the same bytes, which is what makes the SHA-256 in ``cases.json`` meaningful.

Only the handful of messages Stride reads are supported:
``file_id``, ``device_info``, ``record``, ``lap`` and ``session``.
"""

from __future__ import annotations

import struct
from datetime import datetime, timezone
from typing import Any

from fitparse.profile import MESSAGE_TYPES

# FIT timestamps count seconds from 1989-12-31T00:00:00Z.
UTC_REFERENCE = 631065600

_CRC_TABLE = (
    0x0000, 0xCC01, 0xD801, 0x1400, 0xF001, 0x3C00, 0x2800, 0xE401,
    0xA001, 0x6C00, 0x7800, 0xB401, 0x5000, 0x9C01, 0x8801, 0x4400,
)


def crc16(data: bytes, crc: int = 0) -> int:
    """The FIT CRC-16, exactly as the SDK computes it."""
    for byte in data:
        tmp = _CRC_TABLE[crc & 0xF]
        crc = (crc >> 4) & 0x0FFF
        crc = crc ^ tmp ^ _CRC_TABLE[byte & 0xF]
        tmp = _CRC_TABLE[crc & 0xF]
        crc = (crc >> 4) & 0x0FFF
        crc = crc ^ tmp ^ _CRC_TABLE[(byte >> 4) & 0xF]
    return crc


def _message_type(name: str):
    for message_type in MESSAGE_TYPES.values():
        if message_type.name == name:
            return message_type
    raise KeyError(f"unknown FIT message: {name}")


class FitWriter:
    """Build a FIT file byte by byte.

    Usage::

        writer = FitWriter()
        writer.message("file_id", {...})
        writer.message("record", {...})
        data = writer.to_bytes()
    """

    def __init__(self, profile_version: int = 2140) -> None:
        self._body = bytearray()
        self._definitions: dict[tuple[str, tuple[str, ...]], int] = {}
        self._next_local = 0
        self._profile_version = profile_version

    # ------------------------------------------------------------ writing

    def message(self, name: str, fields: dict[str, Any]) -> None:
        """Append one data message.

        ``fields`` maps FIT field names to values. Enum fields accept either
        the numeric value or the profile's name (``"sport": "running"``).
        """
        message_type = _message_type(name)
        ordered = list(fields.items())
        key = (name, tuple(field for field, _ in ordered))
        local = self._definitions.get(key)
        if local is None:
            local = self._next_local
            self._next_local += 1
            self._definitions[key] = local
            self._body += self._definition(message_type, ordered, local)
        self._body.append(local & 0x0F)
        for field_name, value in ordered:
            profile_field = self._lookup(message_type, field_name)
            self._body += self._encode(profile_field, value)

    def _definition(self, message_type, ordered, local: int) -> bytes:
        out = bytearray()
        out.append(0x40 | (local & 0x0F))  # definition record header
        out.append(0)  # reserved
        out.append(0)  # architecture: little endian
        out += struct.pack("<H", message_type.mesg_num)
        out.append(len(ordered))
        for field_name, _ in ordered:
            profile_field = self._lookup(message_type, field_name)
            base = self._base_type(profile_field)
            size = 16 if base.identifier == 0x07 else base.size
            out.append(profile_field.def_num)
            out.append(size)
            out.append(base.identifier)
        return bytes(out)

    @staticmethod
    def _lookup(message_type, field_name: str):
        for def_num, field in message_type.fields.items():
            if field.name == field_name:
                return field
        raise KeyError(f"{message_type.name} has no field {field_name!r}")

    @staticmethod
    def _base_type(profile_field):
        field_type = profile_field.type
        return field_type if hasattr(field_type, "identifier") else field_type.base_type

    def _encode(self, profile_field, value: Any) -> bytes:
        base = self._base_type(profile_field)
        identifier = base.identifier

        if identifier == 0x07:  # string
            return str(value).encode("utf-8").ljust(16, b"\x00")

        if isinstance(value, datetime):
            raw = int(value.timestamp()) - UTC_REFERENCE
        else:
            raw = value
            if hasattr(profile_field.type, "values") and isinstance(raw, str):
                raw = self._enum_value(profile_field, raw)
            scale = profile_field.scale
            offset = profile_field.offset
            if scale and offset:
                raw = round((raw + offset) * scale)
            elif scale:
                raw = round(raw * scale)
            else:
                raw = int(raw)

        formats = {
            0x00: "<B",  # enum
            0x01: "<b",  # sint8
            0x02: "<B",  # uint8
            0x83: "<h",  # sint16
            0x84: "<H",  # uint16
            0x85: "<i",  # sint32
            0x86: "<I",  # uint32
            0x8B: "<H",  # uint16z
            0x8C: "<I",  # uint32z
            0x88: "<f",  # float32
            0x89: "<d",  # float64
        }
        if identifier not in formats:
            raise KeyError(f"unsupported base type 0x{identifier:02X}")
        return struct.pack(formats[identifier], raw)

    @staticmethod
    def _enum_value(profile_field, name: str) -> int:
        for number, text in profile_field.type.values.items():
            if text == name:
                return number
        raise KeyError(f"{name!r} is not a value of {profile_field.name}")

    # ------------------------------------------------------------- output

    def to_bytes(self) -> bytes:
        header = bytearray()
        header.append(14)  # header size
        header.append(0x20)  # protocol version 2.0
        header += struct.pack("<H", self._profile_version)
        header += struct.pack("<I", len(self._body))
        header += b".FIT"
        header += struct.pack("<H", crc16(bytes(header)))
        data = bytes(header) + bytes(self._body)
        return data + struct.pack("<H", crc16(data))


def epoch(text: str) -> datetime:
    """Parse an ISO-8601 UTC timestamp into an aware datetime."""
    return datetime.fromisoformat(text.replace("Z", "+00:00")).astimezone(timezone.utc)
