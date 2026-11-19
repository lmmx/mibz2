"""Apply P32 successor and P16 label encoding to the generated frozen edge kernel."""
from pathlib import Path


def apply(src: str) -> str:
    field = "    edges: Vec<u32>,"
    assert src.count(field) == 1
    src = src.replace(field, "    packed_pages: DisplacementPages,\n" + field)
    start = src.index("    pub fn run<const INDUCE:")
    end = src.index("    pub fn price<const INDUCE:", start)
    old = src[start:end]
    a = old.index("        } else {\n            emit(")
    b = old.index("        }\n        stats", a)
    new = old[:a] + """        } else {
            self.packed_pages.encode(&self.edges, &self.extra, bits, n, mark);
            self.edges.clear();
            self.extra.clear();
            // DISPLACEMENT_EMIT_BOUNDARY
            self.packed_pages.emit(&self.spelling, &self.offsets, out);
""" + old[b:]
    return src[:start] + new + src[end:] + Path(__file__).with_name("native.inc.rs").read_text()
