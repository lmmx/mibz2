#!/usr/bin/env python3
"""Cycle 17 Part 3: Full Acquisition Manifest Verification (Cycles 1-17)."""
import hashlib
import subprocess
import sys
from pathlib import Path

CHECKER = Path("/home/antigravity/lmmx/mibz2/docs/team/acquisition/check_manifest.py")
REGISTRY = Path("/home/antigravity/lmmx/mibz2/docs/team/acquisition/primitives.toml")
JOURNAL_ROOT = Path("/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-foundational-contraction-programme")
TMP_ROOT = Path("/tmp")


def sha256_file(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def run_check(manifest: Path):
    proc = subprocess.run(
        [sys.executable, str(CHECKER), str(manifest), "--registry", str(REGISTRY)],
        capture_output=True,
        text=True,
    )
    return proc.returncode == 0, proc.returncode, proc.stdout.strip(), proc.stderr.strip()


def main():
    manifests = sorted(JOURNAL_ROOT.rglob("*manifest*.toml"))
    tmp_manifests = []
    for d in sorted(TMP_ROOT.glob("fcp-*")):
        if d.is_dir():
            tmp_manifests.extend(sorted(d.rglob("*manifest*.toml")))

    all_prog = manifests + tmp_manifests
    print(f"Checker:  {CHECKER} (sha256={sha256_file(CHECKER)[:16]})")
    print(f"Registry: {REGISTRY} (sha256={sha256_file(REGISTRY)[:16]})")
    print(f"Programme manifests found: {len(manifests)} in journal, {len(tmp_manifests)} in /tmp/fcp-*/")
    print("=" * 96)

    passed = 0
    failed = 0
    for m in all_prog:
        ok, rc, out, err = run_check(m)
        if ok:
            passed += 1
        else:
            failed += 1
        rel = m.relative_to(JOURNAL_ROOT) if m.is_relative_to(JOURNAL_ROOT) else m
        first = out.splitlines()[0] if out else err
        dep_lines = [ln.strip() for ln in out.splitlines() if "dependent/n-scale:" in ln]
        print(f"[{'PASS' if ok else 'FAIL'}] (rc={rc}) {rel} | sha256={sha256_file(m)[:12]}")
        print(f"  {first}")
        for dl in dep_lines:
            print(f"  {dl}")
        if not ok and err:
            print(f"  STDERR: {err}")

    print("=" * 96)
    print(f"PROGRAMME SUMMARY: {passed}/{len(all_prog)} PASS, {failed}/{len(all_prog)} FAIL")

    # Sanity-check the 3 reference example manifests shipped with check_manifest.py
    ex_dir = CHECKER.parent
    print("\nReference example manifests (sanity-checking checker sensitivity):")
    for ex in sorted(ex_dir.glob("example-*.toml")):
        ok, rc, out, err = run_check(ex)
        first = out.splitlines()[0] if out else err
        print(f"  [{'PASS' if ok else 'FAIL'}] (rc={rc}) {ex.name}: {first}")


if __name__ == "__main__":
    main()
