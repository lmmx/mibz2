"""robust.py GOOD.bz2 TRUTH OUT.jsonl : each decoder on damaged copies of GOOD.bz2.

Variants (written next to OUT.jsonl):
- truncated: the first 60% of the bytes;
- flipdata: one bit flipped in the middle byte (inside a block's Huffman-coded data);
- blockcrc: one bit flipped in the first block's stored CRC (the block decodes cleanly, so
  only the CRC check can catch it);
- streamcrc: one bit flipped in the last stream's combined CRC (every block is good).

For each decoder and thread count (1, 4): the exit code (a signal is negative), the bytes
written, how many of them are a correct prefix of TRUTH, and whether any byte is wrong. A
pass is a non-zero exit with no wrong byte (bytes of the good blocks before the damage
are allowed). "Wrong bytes then an error" and "exit 0" are reported apart.
"""
import json, os, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mkstream import EOS, magics  # noqa: E402
from run import command  # noqa: E402

DECODERS = ["bz2k", "bunzip", "pbz2", "fbz", "lbzip2rs", "bzip2rs", "indexed_bzip2", "lbzip2", "bzip2"]
ONE_THREAD = {"bzip2", "bzip2rs"}


def variants(good: bytes) -> dict[str, bytes]:
    v = {"truncated": good[: len(good) * 6 // 10]}
    b = bytearray(good)
    b[len(b) // 2] ^= 0x08
    v["flipdata"] = bytes(b)
    b = bytearray(good)
    b[13] ^= 0x01  # "BZh9" + the 6-byte magic, then the CRC's 4 bytes (10-13)
    v["blockcrc"] = bytes(b)
    tail = good[-64:]
    e = magics(tail, EOS)[-1] + (len(good) - 64) * 8
    bit = e + 48 + 31
    b = bytearray(good)
    b[bit // 8] ^= 0x80 >> (bit % 8)
    v["streamcrc"] = bytes(b)
    return v


def main() -> None:
    good_path, truth_path, out = sys.argv[1:4]
    good, truth = open(good_path, "rb").read(), open(truth_path, "rb").read()
    d = os.path.dirname(os.path.abspath(out))
    for name, data in variants(good).items():
        p = os.path.join(d, f"robust-{name}.bz2")
        open(p, "wb").write(data)
        for dec in DECODERS:
            for t in (1, 4):
                if t > 1 and dec in ONE_THREAD:
                    continue
                try:
                    r = subprocess.run(command(dec, t, p), capture_output=True, timeout=120)
                    code, o, err = r.returncode, r.stdout, r.stderr.decode(errors="replace")
                except subprocess.TimeoutExpired as e:
                    code, o, err = "timeout", e.stdout or b"", "timeout"
                k = 0
                n = min(len(o), len(truth))
                while k < n and o[k] == truth[k]:
                    k += 1  # (4 MB at most: fine in Python)
                wrong = len(o) - k
                verdict = ("PASS" if code != 0 and wrong == 0 else
                           "SILENT WRONG OUTPUT" if code == 0 and o != truth else
                           "exit 0, output correct" if code == 0 else "error after wrong bytes")
                if isinstance(code, int) and (code < 0 or code == 101 or "Traceback" in err or "panicked" in err):
                    verdict += " (crash/panic)"
                rec = {"variant": name, "decoder": dec, "threads": t, "exit": code, "out_bytes": len(o),
                       "correct_prefix": k, "wrong_bytes": wrong, "verdict": verdict, "stderr": err.strip()[-200:]}
                with open(out, "a") as f:
                    f.write(json.dumps(rec) + "\n")
                print(f"{name:10s} {dec:14s} t={t} exit={code} out={len(o)} wrong={wrong} {verdict} | {err.strip()[-90:]!r}", flush=True)


if __name__ == "__main__":
    main()
