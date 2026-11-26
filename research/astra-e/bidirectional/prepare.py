"""Build the scoreable variant from its recorded conjugate control. Idempotent."""
from pathlib import Path
import subprocess

folder = Path(__file__).resolve().parent
text = subprocess.check_output(
    ["git", "show", "775e604849d4bba0c474bf9f48cda486d3dbf3c8:research/astra-e/open/candidate.rs"],
    cwd=folder, text=True,
)
def change(old, new):
    global text
    assert text.count(old) == 1, old
    text = text.replace(old, new)

change("mod endpoint;", '#[path = "../open/endpoint.rs"]\nmod endpoint;\nmod writer;')
change("    edges: Vec<u32>,", "    edges: Vec<u32>,\n    reverse: Vec<u32>,")
change("            self.fill::<false, LIMIT>", "            self.reverse.resize(m as usize, 0);\n            self.fill::<false, LIMIT>")
change("        emit(\n", """        if !wide {
            writer::emit(&self.edges, &self.reverse, &self.spelling, &self.offsets,
                n as usize, mark, bits, out);
            return;
        }
        emit(
""")
change("                if WIDE {\n                    self.extra",
"""                if !WIDE {
                    let span = next - dst;
                    for (entry, value) in self.reverse[source as usize..(source + span) as usize]
                        .iter_mut()
                        .zip((dst | pack)..)
                    {
                        *entry = value;
                    }
                }
                if WIDE {
                    self.extra""")
(folder / "candidate.rs").write_text(text)
