#!/usr/bin/env python3
"""Convert Focusa's authenticated legacy Minisign envelope to EdScB2.

Secret input and output are intentionally value-only. Callers must capture stdout
without logging it. Diagnostics never include key material or passwords.
"""

import base64
import ctypes
import ctypes.util
import hashlib
import hmac
import os
import struct
import sys


def fail(message: str) -> "NoReturn":
    raise SystemExit(message)


def decode_key_box(value: str) -> tuple[bytes, bytes]:
    encoded = value.encode()
    if value.startswith("untrusted comment:"):
        key_box = encoded
    else:
        try:
            key_box = base64.b64decode(encoded, validate=True)
        except ValueError:
            fail("invalid Tauri signing key transport")
    if key_box.startswith((b"EdSc", b"EdScB2")):
        return b"untrusted comment: minisign encrypted secret key", key_box
    lines = key_box.splitlines()
    if len(lines) != 2 or not lines[0].startswith(b"untrusted comment:"):
        fail("invalid Tauri signing key box")
    try:
        return lines[0], base64.b64decode(lines[1], validate=True)
    except ValueError:
        fail("invalid Tauri signing key payload")


def open_legacy(raw: bytes, password: bytes) -> bytes:
    if len(raw) != 164 or raw[:4] != b"EdSc":
        fail("unsupported legacy Tauri signing key payload")
    checksum = raw[4:12]
    salt = raw[12:28]
    ops_bytes = raw[28:36]
    memory_bytes = raw[36:44]
    sealed = raw[44:]
    ops = struct.unpack("<Q", ops_bytes)[0]
    memory = struct.unpack("<Q", memory_bytes)[0]
    if ops != 1 << 18 or memory != 1 << 30 or len(sealed) != 120:
        fail("unexpected legacy Tauri signing key parameters")
    seed = hashlib.scrypt(
        password,
        salt=salt,
        n=ops,
        r=8,
        p=1,
        maxmem=memory,
        dklen=32,
    )
    library_name = ctypes.util.find_library("sodium")
    if not library_name:
        fail("system libsodium is unavailable")
    sodium = ctypes.CDLL(library_name)
    byte_pointer = ctypes.POINTER(ctypes.c_ubyte)
    sodium.crypto_box_seed_keypair.argtypes = [byte_pointer, byte_pointer, byte_pointer]
    sodium.crypto_box_seed_keypair.restype = ctypes.c_int
    sodium.crypto_box_seal_open.argtypes = [
        byte_pointer,
        byte_pointer,
        ctypes.c_ulonglong,
        byte_pointer,
        byte_pointer,
    ]
    sodium.crypto_box_seal_open.restype = ctypes.c_int
    public_key = (ctypes.c_ubyte * 32)()
    secret_key = (ctypes.c_ubyte * 32)()
    seed_buffer = (ctypes.c_ubyte * 32).from_buffer_copy(seed)
    if sodium.crypto_box_seed_keypair(public_key, secret_key, seed_buffer) != 0:
        fail("legacy Tauri key derivation failed")
    plaintext = (ctypes.c_ubyte * 72)()
    ciphertext = (ctypes.c_ubyte * len(sealed)).from_buffer_copy(sealed)
    if sodium.crypto_box_seal_open(
        plaintext,
        ciphertext,
        len(sealed),
        public_key,
        secret_key,
    ) != 0:
        fail("legacy Tauri key authentication failed")
    keynum_secret = bytes(plaintext)
    expected = hashlib.blake2b(
        salt + ops_bytes + memory_bytes + keynum_secret,
        digest_size=8,
    ).digest()
    if not hmac.compare_digest(checksum, expected):
        fail("legacy Tauri key checksum failed")
    return keynum_secret


def encode_current(keynum_secret: bytes, password: bytes) -> bytes:
    if len(keynum_secret) != 72:
        fail("invalid decrypted Tauri signing key length")
    salt = os.urandom(32)
    ops = 1_048_576
    memory = 33_554_432
    checksum = hashlib.blake2b(b"Ed" + keynum_secret, digest_size=32).digest()
    plaintext = keynum_secret + checksum
    stream = hashlib.scrypt(
        password,
        salt=salt,
        n=32768,
        r=8,
        p=1,
        maxmem=memory * 2,
        dklen=len(plaintext),
    )
    encrypted = bytes(value ^ mask for value, mask in zip(plaintext, stream))
    return b"EdScB2" + salt + struct.pack("<Q", ops) + struct.pack("<Q", memory) + encrypted


def main() -> None:
    value = os.environ.get("TAURI_SIGNING_PRIVATE_KEY", "")
    password_value = os.environ.get("TAURI_SIGNING_PRIVATE_KEY_PASSWORD", "")
    if not value or not password_value:
        fail("missing Tauri signing key input")
    comment, raw = decode_key_box(value)
    if raw.startswith(b"EdScB2") and len(raw) == 158:
        current = raw
    else:
        current = encode_current(open_legacy(raw, password_value.encode()), password_value.encode())
    key_box = comment + b"\n" + base64.b64encode(current) + b"\n"
    sys.stdout.write(base64.b64encode(key_box).decode())


if __name__ == "__main__":
    main()
