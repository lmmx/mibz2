"""Algebra-only counterexample and cost arithmetic. No decoder/candidate execution."""
import json
text = "aab"
rotations = sorted(text[i:] + text[:i] for i in range(len(text)))
L = "".join(r[-1] for r in rotations)
F = "".join(sorted(L))
# Stable inverse of LF by matching occurrences.
positions = {c: [i for i, x in enumerate(L) if x == c] for c in set(L)}
used = {c: 0 for c in set(L)}
psi = []
for c in F:
    psi.append(positions[c][used[c]])
    used[c] += 1
start = [q == 0 or psi[q] != psi[q-1] + 1 or F[q] != F[q-1] for q in range(len(L))]
def word(q, h):
    out = ""
    for _ in range(h):
        out += F[q]
        q = psi[q]
    return out, q
q = 1
K = 0
while not start[q]:
    K += 1
    q = psi[q]
claimed_h = K + 1
a = word(0, claimed_h)
b = word(1, claimed_h)
assert a[0] != b[0]
assert b[1] != a[1] + 1
assert word(0, K)[0] == word(1, K)[0]
assert word(1, K)[1] == word(0, K)[1] + 1
result = {"counterexample": {"text": text, "L": L, "F": F, "psi": psi, "starts": start,
    "cell": [0,2], "K_at_interior": K, "claimed_height": claimed_h,
    "row0_word_exit": a, "row1_word_exit": b},
    "conditional_k8": {"f": .48, "entries_per_row": .0086,
        "table_lookups_per_row": (1-.48)/8+.0086,
        "lookup_saving_per_row": .48/8-.0086,
        "switch2_entry12_Ir_per_row": 2/8+.0086*12,
        "break_even_lookup_minus_copy_Ir": (2/8+.0086*12)/(.48/8-.0086)}}
print(json.dumps(result,indent=2))
