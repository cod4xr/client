#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare aggregate Rust wavelet pixels with a separate string-bit Python decoder.

Reads an operator-owned main directory and existing inspector evidence. Never writes
pixels or names. The immutable numerical codebooks are the only shared codec data.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from zipfile import ZipFile
from import_iwi_wavelet_tables import codebooks, pinned_source


def tables() -> dict:
    result = {}
    for name, rows in codebooks(pinned_source().decode("utf-8")).items():
        table = {}
        for code, count, value in rows:
            prefix = f"{code:0{count}b}"[::-1]
            for suffix in range(1 << (12 - count)):
                key = prefix + f"{suffix:0{12 - count}b}" if count < 12 else prefix
                table[key] = (value, count)
        assert len(table) == 4096
        result[name] = table
    return result


class Reader:
    def __init__(self, data: bytes, books: dict):
        self.data, self.books = data, books
        self.raw_position = self.position = 0
        self.text = None

    def raw(self) -> int:
        if self.text is not None or self.raw_position >= len(self.data):
            raise ValueError("Missing or late raw wavelet byte")
        value = self.data[self.raw_position]
        self.raw_position += 1
        return value

    def start(self) -> None:
        if self.text is None:
            self.text = "".join(f"{byte:08b}"[::-1] for byte in self.data)
            self.position = self.raw_position * 8

    def take(self, count: int) -> int:
        self.start()
        end = self.position + count
        if end > len(self.text):
            raise ValueError("Missing wavelet entropy bits")
        value = int(self.text[self.position:end][::-1], 2)
        self.position = end
        return value

    def value(self, name: str) -> int:
        self.start()
        prefix = self.text[self.position:self.position + 12].ljust(12, "0")
        value, count = self.books[name][prefix]
        self.take(count)
        if value == -32768:
            return self.take(10 if name == "red_green" else 9) - (510 if name == "red_green" else 255)
        return value


def decode(data: bytes, books: dict) -> tuple:
    assert data[:4] == b"IWi\x06" and 6 <= data[4] <= 10
    width, height, _ = struct.unpack_from("<HHH", data, 6)
    assert width and height and width & (width - 1) == height & (height - 1) == 0
    assert not data[5] & 8 or data[5] & 4
    faces = 6 if data[5] & 4 else 1
    channels = {6: 4, 7: 3, 8: 2, 9: 1, 10: 1}[data[4]]
    stride = 4 if channels == 3 else channels
    levels = 1 if data[5] & 2 else max(width, height).bit_length()
    reader = Reader(data[28:], books)
    previous = None
    decoded = bytearray()
    for level in reversed(range(levels)):
        w, h = max(1, width >> level), max(1, height >> level)
        current = []
        for face in range(faces):
            pixels = bytearray(w * h * stride)
            if min(w, h) == 1:
                for pixel in range(w * h):
                    values = [reader.raw() for _ in range(channels)] + [255] * (stride - channels)
                    pixels[pixel * stride:(pixel + 1) * stride] = bytes(values)
            else:
                parent = bytearray(previous[face])
                if reader.take(1):
                    for index in range(0, len(parent), stride):
                        for channel in range(channels):
                            parent[index + channel] = (parent[index + channel] + reader.value("alpha")) % 256
                for y in range(0, h, 2):
                    for x in range(0, w, 2):
                        reference = ((y // 2) * (w // 2) + x // 2) * stride
                        blue = [0, 0, 0]
                        for channel in range(channels):
                            parity = reader.take(1)
                            book = "alpha" if channel == channels - 1 and channels != 3 else "blue" if channel == 0 else "red_green"
                            values = [reader.value(book) for _ in range(3)]
                            if book == "blue":
                                blue = values[:]
                            elif book == "red_green":
                                values = [a + b for a, b in zip(values, blue)]
                            # Independent inverse transform as four signed dot products.
                            for dx, dy, signs in [(0, 0, (1, 1, 1)), (1, 0, (1, -1, -1)),
                                                  (0, 1, (-1, 1, -1)), (1, 1, (-1, -1, 1))]:
                                value = parent[reference + channel] * 2 + sum(a * b for a, b in zip(values, signs))
                                pixels[((y + dy) * w + x + dx) * stride + channel] = (value // 2 + (parity if dx == dy == 0 else 0)) % 256
                        if channels == 3:
                            for dx, dy in [(0, 0), (1, 0), (0, 1), (1, 1)]:
                                pixels[((y + dy) * w + x + dx) * stride + 3] = 255
            current.append(pixels)
            decoded.extend(pixels)
        previous = current
    consumed = reader.raw_position * 8 if reader.text is None else reader.position
    return decoded, consumed


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("main_directory", type=Path)
    parser.add_argument("evidence", type=Path)
    args = parser.parse_args()
    expected = json.loads(args.evidence.read_text(encoding="utf-8"))
    archives = sorted(args.main_directory.glob("*.iwd"))
    assert len(archives) == expected["archives"] == len(expected["records"])
    books = tables()
    total = 0
    for ordinal, (path, wanted) in enumerate(zip(archives, expected["records"]), 1):
        actual = dict.fromkeys(["wavelet_decoded_images", "wavelet_decoded_bytes",
                               "wavelet_consumed_bits", "wavelet_encoded_bytes"], 0)
        digest = hashlib.sha256()
        with ZipFile(path) as archive:
            for entry in archive.infolist():
                if not entry.filename.lower().endswith(".iwi"):
                    continue
                with archive.open(entry) as stream:
                    header = stream.read(28)
                if len(header) != 28 or not 6 <= header[4] <= 10:
                    continue
                if entry.file_size > 4 * 1024 * 1024:
                    raise ValueError("Wavelet reference input budget exceeded")
                data = archive.read(entry)  # Python independently checks ZIP CRC.
                pixels, consumed = decode(data, books)
                digest.update(pixels)
                actual["wavelet_decoded_images"] += 1
                actual["wavelet_decoded_bytes"] += len(pixels)
                actual["wavelet_consumed_bits"] += consumed
                actual["wavelet_encoded_bytes"] += len(data) - 28
        actual["wavelet_pixel_sha256"] = list(digest.digest()) if actual["wavelet_decoded_images"] else None
        if actual != wanted:
            raise ValueError(f"Wavelet aggregate differs at archive ordinal {ordinal}")
        total += actual["wavelet_decoded_images"]
        print(f"Independent pixel comparison: archive {ordinal} of {len(archives)}", flush=True)
    print(f"Matched {total} complete wavelet images and per-archive pixel SHA-256 digests")


if __name__ == "__main__":
    main()
