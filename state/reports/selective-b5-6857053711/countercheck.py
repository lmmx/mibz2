"""Bounded source/provenance checks and byte-copy model; not a Rust/Miri run."""
import hashlib, json, re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
head = (ROOT / 'head.rs').read_text()
old = (ROOT / 'original.rs').read_text()
eager = (ROOT / 'comparator.rs').read_text()

def function(s, name):
    start = s.index('    fn ' + name + '(')
    p = s.index('{', start)
    depth, end = 1, p + 1
    while depth:
        depth += (s[end] == '{') - (s[end] == '}')
        end += 1
    return s[start:end]

def unhook(s):
    s, n = re.subn(r'#\[cfg\(test\)\]\s*self.record_materialization\(\[.*?\]\);', '', s, flags=re.S)
    assert n == 1
    s, n = re.subn(r'#\[cfg\(test\)\]\s*self.record_selected_slots\(ts, bs\);', '', s)
    assert n == 3
    return s

compact = lambda s: re.sub(r'\s+', '', s)
assert compact(unhook(function(eager, 'induce_cells_original')).replace('induce_cells_original','induce_cells')) == compact(function(old, 'induce_cells'))
unchanged = ['slots', 'write_cells', 'try_invert']
for name in unchanged:
    assert function(head, name) == function(old, name), name

# Replay the exact two 16-byte load-before-store copies for every short length
# pair and every placement of meaningful input prefixes at/before top=32.
# Old/new buffers have different dead tails, but identical meaningful bytes.
# This includes overlapping operand windows and operand tails crossing top.
cases = 0
for la in range(1, 16):
    for lb in range(1, 17-la):
        for a in range(33-la):
            for b in range(33-lb):
                x, y = bytearray([0x39]*80), bytearray([0xC6]*80)
                for at in set(range(a,a+la)) | set(range(b,b+lb)):
                    x[at] = y[at] = (at*37+11)%256
                want = bytes(x[a:a+la] + x[b:b+lb])
                for buf in (x,y):
                    buf[32:48] = bytes(buf[a:a+16])
                    buf[32+la:48+la] = bytes(buf[b:b+16])
                    assert bytes(buf[32:32+la+lb]) == want
                cases += 1

evidence = json.loads((ROOT/'evidence.json').read_text())
manifest = json.loads(evidence['SHA256.json'])
verified = []
for path, content in evidence.items():
    if path in manifest:
        assert hashlib.sha256(content.encode()).hexdigest() == manifest[path], path
        verified.append(path)
sha = hashlib.sha256(head.encode()).hexdigest()
old_sha = hashlib.sha256(old.encode()).hexdigest()
new_mutation_sha = json.loads(evidence['cells-selective-mutations/results.json'])['source_sha256']
original_mutation_sha = json.loads(evidence['cells-selective-original-mutations/results.json'])['source_sha256']
assert original_mutation_sha == sha
out = dict(reviewed_head='f2dc198ee22ad13b69a72d42744fb1a0d135b95f', source_sha256=sha,
    original_source_sha256=old_sha, comparator_equal_after_exactly_four_hooks=True,
    unchanged_functions=unchanged, short_copy_placement_cases=cases,
    exhaustive_column_origin_cases=sum(n*a**n for a,m in [(2,8),(3,6)] for n in range(1,m+1)),
    verified_evidence_files=verified, new_mutations_source_sha256=new_mutation_sha,
    new_mutations_match_reviewed_head=(new_mutation_sha == sha),
    limitation='Python byte model and source inspection only; Rust and Miri were not rerun.')
(ROOT/'countercheck.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
