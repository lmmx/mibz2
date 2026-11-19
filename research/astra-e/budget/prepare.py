"""Compile a duplicate of the exact worker function with clocks in a disposable source tree."""
import pathlib
import shutil
import subprocess
import sys

here = pathlib.Path(__file__).resolve().parent
root = here.parents[2]
dest = pathlib.Path(sys.argv[1]).resolve()
dest.mkdir(parents=True, exist_ok=True)
src = dest / "src"
shutil.copytree(root / "src", src, dirs_exist_ok=True)
lib = src / "lib.rs"
original = lib.read_text()
scope = original[original.index("pub fn decode_block<"):original.index("/// A whole .bz2")]

def edit(old, new):
    global scope
    assert scope.count(old) == 1, old
    scope = scope.replace(old, new)

edit("pub fn decode_block<", "pub fn budget_decode<")
edit(") -> Result<u32, ErrorKind> {\n    let cooperative", "    m: &mut Budget,\n) -> Result<u32, ErrorKind> {\n    m.start = budget_clock();\n    let cooperative")
edit("        #[cfg(feature = \"moonshot\")]","        m.rows = n;\n        m.caps = s.runs.records().len();\n        m.front = budget_clock();\n        #[cfg(feature = \"moonshot\")]")
edit("return check_streamed(result, start, b.crc, k, out);","m.fused = true;\n                m.inverse = budget_clock();\n                m.rle = m.inverse;\n                return budget_check_streamed(result, start, b.crc, k, out);")
edit("return finish(&s.ll, b.crc, k, out);","m.inverse = budget_clock();\n            return budget_finish(&s.ll, b.crc, k, out, m);")
edit("    let in_place =", "    m.rows = n;\n    m.front = budget_clock();\n    let in_place =")
edit("    finish(inverted, b.crc, k, out)", "    m.inverse = budget_clock();\n    budget_finish(inverted, b.crc, k, out, m)")
edit("fn check_streamed(", "fn budget_check_streamed(")
edit("fn finish(inverted: &[u8], stored: u32, k: Kernels, out: &mut Vec<u8>)", "fn budget_finish(inverted: &[u8], stored: u32, k: Kernels, out: &mut Vec<u8>, m: &mut Budget)")
edit("    let computed = crc::crc32(&out[start..], k);\n    if computed", "    m.rle = budget_clock();\n    let computed = crc::crc32(&out[start..], k);\n    if computed")
lib.write_text(original + (here / "clock.rs").read_text() + scope)
flags = ["rustc", "--edition=2021", "-C", "opt-level=3", "-C", "codegen-units=1"]
subprocess.run([*flags, "--crate-name", "mibz2", "--crate-type", "rlib", "--cfg", 'feature="moonshot"', lib, "-o", dest / "libmibz2.rlib"], check=True)
modules = {
    "open_return": "docs/journal/2026-11-18-open-return/open_return.rs",
    "arms": "bench/moonshot/src/arms.rs",
    "ahead": "research/astra-e/narrow-short/ahead.rs",
    "refill": "research/astra-e/formulations/f2-scheduler/candidate.rs",
    "bundle": "research/astra-e/formulations/f2-rle/combined/candidate.rs",
}
main = dest / "main.rs"
main.write_text('#![allow(dead_code)]\n' + ''.join(f'#[path = "{root / f}"] mod {n};\n' for n, f in modules.items()) + (here / "main.rs").read_text())
subprocess.run([*flags, main, "--extern", f"mibz2={dest}/libmibz2.rlib", "-o", dest / "measure"], check=True)
