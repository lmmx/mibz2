"""Install the exact width-16 spelling reuse pass into the frozen edge constructor."""
from pathlib import Path

HERE = Path(__file__).resolve().parent


def apply(src: str) -> str:
    anchor = "        stats.bands = self.bands.len();"
    allocation = "        self.edges.resize(m as usize, 0);"
    if src.count(anchor) != 1 or src.count(allocation) != 1:
        raise ValueError("unexpected edge constructor layout")
    hook = """        self.edges.resize(m as usize, 0);
        if INDUCE && limit == 16 {
            self.alias_spellings(&mut stats);
        }
"""
    return src.replace(allocation + "\n", "").replace(anchor, hook + anchor) + (HERE / "byte_alias.inc.rs").read_text()
