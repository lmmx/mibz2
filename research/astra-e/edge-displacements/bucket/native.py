"""Install bounded queue select directly before the frozen physical-edge materialization."""
from pathlib import Path

HERE = Path(__file__).resolve().parent


def apply(src: str) -> str:
    def change(anchor, replacement):
        nonlocal src
        assert src.count(anchor) == 1, anchor
        src=src.replace(anchor,replacement)
    change("    edges: Vec<u32>,", "    bucket: BucketResidual,\n    edges: Vec<u32>,")
    anchor="        self.edges.resize(m as usize, 0);"
    change(anchor, """        self.bucket.used = false;
        if INDUCE && QUEUE && self.bucket.allowed && limit == 16 && !wide
            && self.bands.len() <= usize::from(u16::MAX) && bucket_features()
            && bucket_bound(m as usize, bits, self.bands.len()) <= 4 * m as usize
        {
            self.bucket.build(&self.bands, &self.queues, m as usize, bits);
            self.edges.clear();
            self.extra.clear();
            self.bucket.used = true;
            stats.table_bytes = self.bucket.live();
            return (stats, mark, bits);
        }
        self.bucket.reset();
""" + anchor)
    change("        let (stats, mark, bits) = self.construct::<INDUCE, QUEUE>(records,",
           "        self.bucket.allowed = !SEGMENTS;\n        let (stats, mark, bits) = self.construct::<INDUCE, QUEUE>(records,")
    change("        if SEGMENTS {\n            self.segments.emit(", """        if self.bucket.used {
            // DISPLACEMENT_EMIT_BOUNDARY
            self.emit_bucket(n, mark, bits, out);
        } else if SEGMENTS {
            self.segments.emit(""")
    change("        let (_, mark, bits) = self.construct::<INDUCE, QUEUE>(runs.records(),",
           "        self.bucket.allowed = true;\n        let (_, mark, bits) = self.construct::<INDUCE, QUEUE>(runs.records(),")
    a=src.index("        let start = std::time::Instant::now();",src.index("    pub fn price<"))
    b=src.index("        (built, start.elapsed()",a)
    emit=src[a:b]
    key="        emit("
    assert emit.count(key)==1
    src=src[:a]+emit.replace(key,"        if self.bucket.used {\n            self.emit_bucket(n, mark, bits, out);\n        } else {\n            emit(").rstrip()+"\n        }\n"+src[b:]
    return src+"\n"+(HERE/"native.inc.rs").read_text()
