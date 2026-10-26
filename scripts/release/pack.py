#!/usr/bin/env python3
"""SECURITY-SPEC S3-4: package one release binary as a byte-reproducible
.tar.gz. The archive holds a single member `invar`, mode 0755, owner and
group 0 with empty names, modification time 0; the gzip header carries no
file name and time 0. The same binary bytes give the same archive bytes on
any machine, at any time, from any path. Usage:
    python3 scripts/release/pack.py <path/to/invar> <out.tar.gz>
"""
import gzip
import io
import sys
import tarfile


def pack(binary: str, out: str) -> None:
    data = open(binary, "rb").read()
    info = tarfile.TarInfo("invar")
    info.size = len(data)
    info.mode = 0o755
    info.mtime = 0
    info.uid = info.gid = 0
    info.uname = info.gname = ""
    info.type = tarfile.REGTYPE
    raw = io.BytesIO()
    with tarfile.open(fileobj=raw, mode="w", format=tarfile.USTAR_FORMAT) as t:
        t.addfile(info, io.BytesIO(data))
    with open(out, "wb") as f:
        with gzip.GzipFile(filename="", mode="wb", fileobj=f, mtime=0, compresslevel=9) as z:
            z.write(raw.getvalue())


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit("usage: pack.py <binary> <out.tar.gz>")
    pack(sys.argv[1], sys.argv[2])
