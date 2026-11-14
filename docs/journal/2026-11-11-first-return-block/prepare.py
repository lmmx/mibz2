"""Prepare a private test-only export at the exact baseline revision."""

import argparse
import hashlib
import io
import shutil
import subprocess
import tarfile
from pathlib import Path

BASE = "57f51bbc0ca62ac3ee1cc148a1010842eea6222a"


def main():
    """Copy frozen source; add only a cfg(test) module for private run access."""
    parser = argparse.ArgumentParser()
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    args.destination.mkdir(parents=True, exist_ok=False)
    archive = subprocess.check_output(["git", "archive", BASE])
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        tar.extractall(args.destination, filter="data")
    source = Path(__file__).with_name("export.rs")
    shutil.copyfile(source, args.destination / "src/first_return_export.rs")
    lib = args.destination / "src/lib.rs"
    original = lib.read_bytes()
    lib.write_bytes(original + b"\n#[cfg(test)]\nmod first_return_export;\n")
    print("baseline", BASE)
    print("original_lib_sha256", hashlib.sha256(original).hexdigest())
    print("export_sha256", hashlib.sha256(source.read_bytes()).hexdigest())


if __name__ == "__main__":
    main()
