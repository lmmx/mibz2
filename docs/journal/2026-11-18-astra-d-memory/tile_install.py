"""Install the 128-row forward construction in the disposable native tracer."""

from pathlib import Path
import sys

tree = Path(sys.argv[1]) / "src"
code = (Path(__file__).resolve().parent / "tiles.rs").read_text()
p = tree / "lib.rs"
s = p.read_text()


def once(s, a, b):
    """Replace a substring that must occur exactly once."""
    assert s.count(a) == 1, a
    return s.replace(a, b)


s = once(s, "    inverse: I,", "    inverse: I,\n    tiles: AstraTiles,")
s = once(
    s,
    "            inverse: I::default(),",
    "            inverse: I::default(),\n            tiles: AstraTiles::default(),",
)
s = once(
    s,
    "let done = s.inverse.invert_runs(",
    """if out.is_empty() && n < (1<<20)-128 {
            return s.tiles.decode(&s.runs,n,b.orig_ptr,out,k,b.crc);
        }
        let done = s.inverse.invert_runs(""",
)
s = once(
    s,
    "    scratch.runs.astra_ranges();",
    "    scratch.runs.astra_ranges();\n    scratch.tiles.ranges();",
)
p.write_text(s + code)
p = tree / "runs.rs"
p.write_text(
    p.read_text()
    + "\nimpl Runs { pub(crate) fn astra_counts(&self) -> &[u32;256] { &self.counts } }\n"
)
p = tree / "crc.rs"
p.write_text(
    p.read_text()
    + """
pub(crate) fn astra_update(s:u32, data:&[u8], k:Kernels) -> u32 {
    if k.clmul && data.len() >= 128 {
        unsafe { clmul_update(s,data) }
    } else { update(s,data) }
}
"""
)
