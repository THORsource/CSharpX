"""Translate native MSVC PDB line records from generated Rust to CSharpX."""
import hashlib
import struct
import sys
from pathlib import Path

FILE_CHECKSUMS = 0xF4
LINES = 0xF2
LOCAL_SYMBOL = 0x113E


def unpack(fmt, data, offset):
    return struct.unpack_from("<" + fmt, data, offset)


class Pdb:
    def __init__(self, data):
        self.data = data
        if not data.startswith(b"Microsoft C/C++ MSF 7.00"):
            raise ValueError("expected an MSF 7.0 PDB")
        self.block_size, _, _, size, _, block_map = unpack("6I", data, 32)
        blocks = self.block_list(data, block_map * self.block_size, size)
        directory = self.read_blocks(blocks, size)
        count, = unpack("I", directory, 0)
        sizes = unpack(f"{count}I", directory, 4)
        self.streams = []
        offset = 4 + 4 * count
        for size in sizes:
            size = 0 if size == 0xFFFFFFFF else size
            blocks = self.block_list(directory, offset, size)
            self.streams.append((blocks, size))
            offset += len(blocks) * 4

    def block_list(self, data, offset, size):
        count = (size + self.block_size - 1) // self.block_size
        return unpack(f"{count}I", data, offset)

    def read_blocks(self, blocks, size):
        return bytearray(b"".join(
            self.data[b * self.block_size:(b + 1) * self.block_size]
            for b in blocks
        )[:size])

    def read(self, index):
        return self.read_blocks(*self.streams[index])

    def write(self, index, contents):
        blocks, size = self.streams[index]
        if len(contents) != size:
            raise ValueError("stream size changed")
        for i, block in enumerate(blocks):
            start = block * self.block_size
            piece = contents[i * self.block_size:(i + 1) * self.block_size]
            self.data[start:start + len(piece)] = piece


def align(offset):
    return (offset + 3) & ~3


def modules(dbi):
    size, = unpack("I", dbi, 24)
    offset = 64
    while offset < 64 + size:
        stream, symbols, c11, c13 = unpack("HIII", dbi, offset + 34)
        name_end = dbi.index(0, offset + 64)
        file_end = dbi.index(0, name_end + 1)
        offset = align(file_end + 1)
        if stream != 0xFFFF and c13:
            yield stream, symbols, symbols + c11, c13


def translate_module(module, start, size, hashes, mapping):
    sections = []
    offset = start
    while offset < start + size:
        kind, length = unpack("II", module, offset)
        sections.append((kind, offset + 8, length))
        offset = align(offset + 8 + length)

    translated_files = set()
    for kind, section, length in sections:
        if kind != FILE_CHECKSUMS:
            continue
        offset = section
        while offset < section + length:
            _, count, algorithm = unpack("IBB", module, offset)
            begin = offset + 6
            old, new = hashes.get(algorithm, (None, None))
            if old is not None and bytes(module[begin:begin + count]) == old:
                module[begin:begin + count] = new
                translated_files.add(offset - section)
            offset = align(begin + count)

    translated_lines = 0
    for kind, section, length in sections:
        if kind != LINES:
            continue
        _, _, flags, _ = unpack("IHHI", module, section)
        offset = section + 12
        while offset < section + length:
            file_id, count, length_of_block = unpack("III", module, offset)
            if not 12 <= length_of_block <= section + length - offset:
                raise ValueError("invalid PDB line block size")
            if file_id in translated_files:
                for i in range(count):
                    record = offset + 12 + i * 8
                    value, = unpack("I", module, record + 4)
                    line = value & 0xFFFFFF
                    if 0 < line <= len(mapping):
                        struct.pack_into("<I", module, record + 4,
                                         (value & 0x80000000) | mapping[line - 1])
                        translated_lines += 1
                    if flags & 1:
                        struct.pack_into("<HH", module, offset + 12 + count * 8 + i * 4, 0, 0)
            offset += length_of_block
    return translated_lines


def restore_local_names(module, symbols_end):
    # The first four bytes are the CodeView signature. An S_LOCAL record
    # stores a type index and flags before its null-terminated name.
    offset = 4
    while offset < symbols_end:
        length, kind = unpack("HH", module, offset)
        end = offset + 2 + length
        if length < 2 or end > symbols_end:
            raise ValueError("invalid PDB symbol record size")
        name_offset = offset + 10
        if kind == LOCAL_SYMBOL and module[name_offset:name_offset + 2] == b"v_":
            name_end = module.index(0, name_offset, end)
            if name_end > name_offset + 2:
                # Keep the record and its position unchanged for other symbol records.
                module[name_offset:name_end] = module[name_offset + 2:name_end] + b"\x00\x00"
        offset = end

def main():
    pdb_path, generated_path, source_path = map(Path, sys.argv[1:])
    mapping = [int(line) for line in sys.stdin]
    generated = generated_path.read_bytes()
    source = source_path.read_bytes()
    hashes = {
        kind: (digest(generated).digest(), digest(source).digest())
        for kind, digest in enumerate((hashlib.md5, hashlib.sha1, hashlib.sha256), 1)
    }
    pdb = Pdb(bytearray(pdb_path.read_bytes()))
    total = 0
    for index, symbols, start, size in modules(pdb.read(3)):
        module = pdb.read(index)
        count = translate_module(module, start, size, hashes, mapping)
        if count:
            restore_local_names(module, symbols)
            pdb.write(index, module)
            total += count
    if not total:
        raise ValueError("no generated source line records matched")
    temporary = pdb_path.with_suffix(".pdb.tmp")
    temporary.write_bytes(pdb.data)
    temporary.replace(pdb_path)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, struct.error) as error:
        sys.exit(f"CSX9013: could not map native debug symbols: {error}")
