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

A real watch writes more than that, so three escape hatches exist:

``RawField``
    A field the FIT profile does not know: a definition number, a base type
    and the raw value. Real watches write fields the published profile does
    not describe (Garmin adds them to their own firmware first), and a
    decoder has to skip them.

``FitWriter.raw_message``
    A whole message whose *global* message number is not in the profile.
    Real files are full of these; a decoder has to skip them too.

``FitWriter.message(..., local=N)``
    Write the message under a chosen local message number, re-emitting its
    definition every time. Real watches reuse the sixteen local message
    numbers and redefine them mid-file, so a decoder has to cope with a
    definition that replaces an earlier one.

The header can also be written as FIT protocol 1.0 (``protocol=0x10``),
which is what a current Garmin watch actually emits; the default is the
protocol 2.0 header the older cases were written with.
"""

from __future__ import annotations

import struct
from datetime import datetime, timezone
from typing import Any, NamedTuple

from fitparse.profile import MESSAGE_TYPES

# FIT timestamps count seconds from 1989-12-31T00:00:00Z.
UTC_REFERENCE = 631065600

_CRC_TABLE = (
    0x0000, 0xCC01, 0xD801, 0x1400, 0xF001, 0x3C00, 0x2800, 0xE401,
    0xA001, 0x6C00, 0x7800, 0xB401, 0x5000, 0x9C01, 0x8801, 0x4400,
)

# The base types of the FIT type byte, with their sizes. These are the
# identifiers ``fitparse`` uses, which are the ones on the wire.
BASE_ENUM = 0x00
BASE_SINT8 = 0x01
BASE_UINT8 = 0x02
BASE_STRING = 0x07
BASE_SINT16 = 0x83
BASE_UINT16 = 0x84
BASE_SINT32 = 0x85
BASE_UINT32 = 0x86
BASE_FLOAT32 = 0x88
BASE_FLOAT64 = 0x89
BASE_UINT8Z = 0x0A
BASE_UINT16Z = 0x8B
BASE_UINT32Z = 0x8C
BASE_BYTE = 0x0D
BASE_SINT64 = 0x8E
BASE_UINT64 = 0x8F

_FIXED_SIZES = {
    BASE_ENUM: 1,
    BASE_SINT8: 1,
    BASE_UINT8: 1,
    BASE_SINT16: 2,
    BASE_UINT16: 2,
    BASE_SINT32: 4,
    BASE_UINT32: 4,
    BASE_FLOAT32: 4,
    BASE_FLOAT64: 8,
    BASE_UINT8Z: 1,
    BASE_UINT16Z: 2,
    BASE_UINT32Z: 4,
    BASE_SINT64: 8,
    BASE_UINT64: 8,
}

_RAW_FORMATS = {
    BASE_ENUM: "<B",
    BASE_SINT8: "<b",
    BASE_UINT8: "<B",
    BASE_SINT16: "<h",
    BASE_UINT16: "<H",
    BASE_SINT32: "<i",
    BASE_UINT32: "<I",
    BASE_FLOAT32: "<f",
    BASE_FLOAT64: "<d",
    BASE_UINT8Z: "<B",
    BASE_UINT16Z: "<H",
    BASE_UINT32Z: "<I",
    BASE_SINT64: "<q",
    BASE_UINT64: "<Q",
}


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


class RawField(NamedTuple):
    """A field the FIT profile does not know.

    ``def_num`` is the field's definition number, ``base_type`` one of the
    ``BASE_*`` constants, and ``size`` the number of bytes the field occupies
    — needed for the variable-length types, the byte array and the string.

    The *value* of such a field is the raw value in the base type's own
    units: no scale and no offset are applied to it.
    """

    def_num: int
    base_type: int
    size: int | None = None


class _Layout(NamedTuple):
    """One field of a message, resolved to bytes."""

    def_num: int
    size: int
    base_type: int
    data: bytes


class FitWriter:
    """Build a FIT file byte by byte.

    Usage::

        writer = FitWriter()
        writer.message("file_id", {...})
        writer.message("record", {...})
        data = writer.to_bytes()
    """

    def __init__(self, profile_version: int = 2140, protocol: int = 0x20) -> None:
        self._body = bytearray()
        self._definitions: dict[tuple, int] = {}
        self._next_local = 0
        self._profile_version = profile_version
        self._protocol = protocol

    # ------------------------------------------------------------ writing

    def message(
        self, name: str, fields: dict[Any, Any], local: int | None = None, define: bool = True
    ) -> None:
        """Append one data message.

        ``fields`` maps FIT field names to values. Enum fields accept either
        the numeric value or the profile's name (``"sport": "running"``). A
        key that is a :class:`RawField` is written verbatim.

        ``local`` forces the local message number; with ``define=False`` only
        the data message is written, so a local message number can be
        defined once and then used many times — which is what a real watch
        does, and what happens when it redefines a local number mid-file.
        """
        message_type = _message_type(name)
        layout = self._layout(message_type, fields)
        if local is None:
            local = self._local_for(message_type.mesg_num, layout)
        elif define:
            self._body += self._definition(message_type.mesg_num, layout, local)
        self._body.append(local & 0x0F)
        for field in layout:
            self._body += field.data

    def raw_message(
        self,
        mesg_num: int,
        fields: list[tuple[RawField, Any]],
        local: int | None = None,
        define: bool = True,
    ) -> None:
        """Append a message whose global number the profile does not know."""
        layout = []
        for field, value in fields:
            size = field.size if field.size is not None else _size_of(field.base_type, value)
            layout.append(
                _Layout(field.def_num, size, field.base_type, _encode_raw(field.base_type, value, size))
            )
        if local is None:
            local = self._local_for(mesg_num, layout)
        elif define:
            self._body += self._definition(mesg_num, layout, local)
        self._body.append(local & 0x0F)
        for field in layout:
            self._body += field.data

    def _local_for(self, mesg_num: int, layout: list[_Layout]) -> int:
        """The local message number for a layout, defining it if it is new."""
        key = (mesg_num, tuple((field.def_num, field.size, field.base_type) for field in layout))
        local = self._definitions.get(key)
        if local is None:
            local = self._next_local
            self._next_local += 1
            self._definitions[key] = local
            self._body += self._definition(mesg_num, layout, local)
        return local

    def _layout(self, message_type, fields: dict[Any, Any]) -> list[_Layout]:
        """Resolve every field of a message to its definition and bytes."""
        layout: list[_Layout] = []
        for key, value in fields.items():
            if isinstance(key, RawField):
                size = key.size if key.size is not None else _size_of(key.base_type, value)
                layout.append(
                    _Layout(key.def_num, size, key.base_type, _encode_raw(key.base_type, value, size))
                )
                continue
            profile_field = self._lookup(message_type, key)
            base = self._base_type(profile_field)
            size = 16 if base.identifier == 0x07 else base.size
            layout.append(
                _Layout(profile_field.def_num, size, base.identifier, self._encode(profile_field, value))
            )
        return layout

    def _definition(self, mesg_num: int, layout: list[_Layout], local: int) -> bytes:
        out = bytearray()
        out.append(0x40 | (local & 0x0F))  # definition record header
        out.append(0)  # reserved
        out.append(0)  # architecture: little endian
        out += struct.pack("<H", mesg_num)
        out.append(len(layout))
        for field in layout:
            out.append(field.def_num)
            out.append(field.size)
            out.append(field.base_type)
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
        header.append(self._protocol)  # protocol version 2.0 (0x20) or 1.0 (0x10)
        header += struct.pack("<H", self._profile_version)
        header += struct.pack("<I", len(self._body))
        header += b".FIT"
        header += struct.pack("<H", crc16(bytes(header)))
        data = bytes(header) + bytes(self._body)
        return data + struct.pack("<H", crc16(data))


def _size_of(base_type: int, value: Any) -> int:
    """The number of bytes a raw field occupies."""
    if base_type == BASE_STRING:
        return len(str(value).encode("utf-8")) + 1
    if base_type == BASE_BYTE:
        return len(value)
    size = _FIXED_SIZES.get(base_type)
    if size is None:
        raise KeyError(f"unsupported base type 0x{base_type:02X}")
    return size


def _encode_raw(base_type: int, value: Any, size: int) -> bytes:
    """Encode a raw value in the base type's own units."""
    if base_type == BASE_STRING:
        return str(value).encode("utf-8").ljust(size, b"\x00")[:size]
    if base_type == BASE_BYTE:
        return bytes(value).ljust(size, b"\x00")[:size]
    fmt = _RAW_FORMATS.get(base_type)
    if fmt is None:
        raise KeyError(f"unsupported base type 0x{base_type:02X}")
    return struct.pack(fmt, value)


def epoch(text: str) -> datetime:
    """Parse an ISO-8601 UTC timestamp into an aware datetime."""
    return datetime.fromisoformat(text.replace("Z", "+00:00")).astimezone(timezone.utc)
