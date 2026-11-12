"""Materialize exact corrected365 source plus additive, safe diagnostic hooks.

No network, build, corpus decode or clocks. Refuse an existing destination.
Every edit is an asserted unique insertion; the original code is never replaced.
"""

import difflib
import hashlib
import json
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
PIN = "d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2"
CFG = '#[cfg(feature = "live-diagnostic")]\n'


def git(*args):
    return subprocess.check_output(["git", "-C", str(ROOT), *args])


def sha(data):
    return hashlib.sha256(data).hexdigest()


def insert(text, needle, code, after=True):
    assert text.count(needle) == 1, (needle, text.count(needle))
    return text.replace(needle, needle + code if after else code + needle)


def hook(code):
    return "\n" + CFG + code + "\n"


def generate():
    dest = HERE / ".source"
    dest.mkdir()  # never overwrite a prepared source/evidence tree
    paths = git("ls-tree", "-r", "--name-only", PIN).decode().splitlines()
    original = {}
    for p in paths:
        if p.startswith(("src/", "tests/", "examples/", "benches/")) or p in (
            "Cargo.toml",
            "Cargo.lock",
            "build.rs",
            "README.md",
            "bench/wikidata.json",
        ):
            data = git("show", f"{PIN}:{p}")
            target = dest / p
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            original[p] = sha(data)
    edits = {}

    def edit(p, fn):
        path = dest / p
        before = path.read_text()
        after = fn(before)
        path.write_text(after)
        edits[p] = {"original": original[p], "diagnostic": sha(after.encode())}

    edit("Cargo.toml", lambda t: insert(t, "[features]\n", "live-diagnostic = []\n"))
    edit(
        "src/lib.rs",
        lambda t: insert(
            t,
            "pub fn decode_block<I: bwt::InverseBwt>(",
            CFG + "pub mod live_diagnostic;\n\n",
            False,
        ),
    )

    def lib(t):
        t = insert(
            t,
            "    let cooperative = s.inverse.wants_checkpoints();",
            hook(
                "let _block = live_diagnostic::block(b.orig_ptr, b.crc);\n"
                + CFG
                + 'let front = live_diagnostic::phase("front");'
            ),
            False,
        )
        t = insert(
            t,
            "        let done = s.inverse.invert_runs",
            hook(
                "drop(front);\n"
                + CFG
                + 'live_diagnostic::set("rows", n);\n'
                + CFG
                + 'live_diagnostic::set("records", s.runs.records().len());\n'
                + CFG
                + 'live_diagnostic::set("outer_runs", 1);\n'
                + CFG
                + 'let inv = live_diagnostic::phase("inversion");'
            ),
            False,
        )
        t = insert(t, "        if done {", hook("drop(inv);"), False)
        t = insert(
            t,
            "    let in_place = !cooperative",
            hook(
                "drop(front);\n"
                + CFG
                + 'live_diagnostic::set("rows", n);\n'
                + CFG
                + 'let inv = live_diagnostic::phase("inversion");'
            ),
            False,
        )
        t = insert(t, "    let inverted: &[u8]", hook("drop(inv);"), False)
        t = insert(
            t,
            "    let start = out.len();",
            hook('let _finish = live_diagnostic::phase("finish_crc");'),
            False,
        )
        t = insert(
            t,
            "    Ok(computed)",
            hook(
                'live_diagnostic::set("decoded_bytes", out.len() - start);\n'
                + CFG
                + 'live_diagnostic::set("computed_crc", computed as usize);\n'
                + CFG
                + 'live_diagnostic::capacity("decoded", out.capacity());'
            ),
            False,
        )
        # The records branch cannot fall through with a live front guard. A borrowed
        # guard can be explicitly stopped twice; stop() is idempotent (no control change).
        t = t.replace("drop(front);", "front.stop();")
        return t

    edit("src/lib.rs", lib)
    edit(
        "src/stream.rs",
        lambda t: insert(
            t,
            "    pub fn next(&mut self, mut syms: Vec<u16>) -> Result<Event, Error> {",
            hook(
                'let _scan = crate::live_diagnostic::phase("scanner");\n'
                + CFG
                + "crate::live_diagnostic::scan_bit(self.bit);"
            ),
        ),
    )
    edit(
        "src/stream.rs",
        lambda t: insert(
            t,
            "                            return Ok(Event::Block {",
            hook("crate::live_diagnostic::scan_bit(at);"),
            False,
        ),
    )

    def cells(t):
        t = insert(
            t,
            "        let budget = if F",
            hook(
                'let _attempt = crate::live_diagnostic::phase("attempt");\n'
                + CFG
                + 'crate::live_diagnostic::set("records", recs.len());'
            ),
            False,
        )
        t = insert(
            t,
            "        let result = self.state.kernel.try_invert",
            hook('crate::live_diagnostic::set("budget", budget);'),
            False,
        )
        t = insert(
            t,
            "        let counts = &mut self.state.counts;",
            hook(
                'crate::live_diagnostic::outcome(&format!("{result:?}"));\n'
                + CFG
                + 'crate::live_diagnostic::capacity("cells_buf", self.state.kernel.buf.capacity());\n'
                + CFG
                + 'crate::live_diagnostic::capacity("cells_pos", self.state.kernel.pos.capacity() * 4);\n'
                + CFG
                + 'crate::live_diagnostic::capacity("inverted", out.capacity());'
            ),
            False,
        )
        for needle in [
            "        self.state.fallback.invert(ll",
            "        self.state.fallback.invert_in_place(ll",
            "        self.state.fallback.invert_runs(r",
        ]:
            t = insert(
                t,
                needle,
                hook(
                    'let _fallback = crate::live_diagnostic::phase("fallback");\n'
                    + CFG
                    + 'crate::live_diagnostic::add("fallback_calls", 1);'
                ),
                False,
            )
        t = insert(
            t,
            "        if n == 0 || n >= 1 << 24",
            hook('let _order = crate::live_diagnostic::phase("order");'),
            False,
        )
        t = insert(
            t,
            "        // Layout uses records",
            hook('crate::live_diagnostic::set("letters", letters);'),
            False,
        )
        t = insert(
            t,
            "        if self.buf.len() < allocation {",
            hook(
                'crate::live_diagnostic::add("cells_buf_growths", 1);\n'
                + CFG
                + 'crate::live_diagnostic::add("cells_buf_requested_zero_bytes", allocation);'
            ),
        )
        t = insert(
            t,
            "        const M: u64 = 0xFFFF_FFFF;",
            hook('let _induce = crate::live_diagnostic::phase("induction");'),
            False,
        )
        for needle in [
            "                        self.rule_end = rp;",
            "                        return Err(Refusal::Budget);",
        ]:
            t = insert(
                t,
                needle,
                hook(
                    'crate::live_diagnostic::add("attempted_cells", cells);\n'
                    + CFG
                    + 'crate::live_diagnostic::add("induction_copy_bytes", 32 * cells);'
                ),
                False,
            )
        t = insert(
            t,
            "    fn write_cells(&mut self, word: u32, len: u32, n: usize, out: &mut Vec<u8>) {",
            hook(
                'let _writer = crate::live_diagnostic::phase("writer");\n'
                + CFG
                + 'crate::live_diagnostic::set("period", (len & !MARK) as usize);\n'
                + CFG
                + 'crate::live_diagnostic::growth("inverted", out.capacity(), n + 32);'
            ),
        )
        t = insert(
            t,
            "            if self.pos.capacity() < nr {",
            hook(
                'crate::live_diagnostic::add("pos_initialized_bytes", nr * 4);\n'
                + CFG
                + 'crate::live_diagnostic::growth("cells_pos", self.pos.capacity() * 4, nr * 4);'
            ),
            False,
        )
        for name, metric, amount in [
            (
                "copy_small(dst: *mut u8, src: *const u8, len: usize)",
                "writer_small_copy_bytes",
                "len",
            ),
            (
                "copy_long(dst: *mut u8, src: *const u8, len: usize)",
                "writer_long_copy_bytes",
                "len",
            ),
        ]:
            t = insert(
                t,
                "unsafe fn " + name + " {",
                hook(f'crate::live_diagnostic::add("{metric}", {amount});'),
            )
        t = insert(
            t,
            "            out.copy_within(..more, have);",
            hook('crate::live_diagnostic::add("writer_repeat_copy_bytes", more);'),
        )
        return t

    edit("src/cells.rs", cells)
    (dest / "src/live_diagnostic.rs").write_bytes((HERE / "src/trace.rs").read_bytes())
    generated = HERE / ".generated"
    generated.mkdir()
    ab = (dest / "examples/ab.rs").read_text()
    hash_source = ab[ab.index("struct Hash {") : ab.index("/// Move the calling thread")]
    assert "unsafe" not in hash_source
    (generated / "hash.rs").write_text(hash_source)
    patch = ""
    for p in edits:
        patch += "".join(
            difflib.unified_diff(
                git("show", f"{PIN}:{p}").decode().splitlines(True),
                (dest / p).read_text().splitlines(True),
                fromfile="a/" + p,
                tofile="b/" + p,
            )
        )
    # No original executable line removed: hooks cannot rewrite an unsafe operation,
    # admission condition, fallback call or writer mutation regression.
    assert not any(
        line.startswith("-") and not line.startswith("---") for line in patch.splitlines()
    ), patch
    (generated / "instrumentation.patch").write_text(patch)
    manifest = {
        "generator_sha256": sha(Path(__file__).read_bytes()),
        "patch_sha256": sha(patch.encode()),
        "pin": PIN,
        "original": original,
        "edits": edits,
        "trace_sha256": sha((HERE / "src/trace.rs").read_bytes()),
        "hash_sha256": sha(hash_source.encode()),
    }
    (generated / "source.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps({"pin": PIN, "source_files": len(original), "edited_files": list(edits)}))


if __name__ == "__main__":
    generate()
