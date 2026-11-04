#!/usr/bin/env python3
"""Apply one registered mutant (or the U1 form, variant a) to examples/c30walk/ctor2c.rs.

    python3 mutate.py NAME      (from the worktree root; restore with git checkout)

The mutants are the components' own, each carried to the combined source at the same
operation:
- scanner (the U1/U2 trial's mutations/): MU1 n1 forced to 17; MU3 U2 one past the tile;
  MU2 U1's end past `ll` on a final full tile, on the U1 form (variant a, as the trial ran
  it, since the final form has no U1); M1 the held tile skipped; M2 the run short by a tile;
- descriptor (its mutation-M*.diff): DM1 `en` one field short at open; DM2 the snapshot's
  rank from `en` instead of `wr`;
- v3 (its mutation.diff): V3M the slow finish's two high-bit tests back to `== u32::MAX`.
"""
import sys
P = 'examples/c30walk/ctor2c.rs'
s = open(P).read()

def sub(old, new, count=1):
    global s
    assert s.count(old) == count, (old, s.count(old))
    s = s.replace(old, new)

VARIANT_A = [
    ("""        // SAFETY (U2): i + n1 ≤ i + 16 ≤ ll.len() by the contract above
        let s2 = unsafe { ll.get_unchecked(i + n1..i + 16) };""",
     """        // SAFETY (U1): i + n1 ≤ i + 16 ≤ ll.len() by the contract above
        let s1 = unsafe { ll.get_unchecked(i..i + n1) };
        // SAFETY (U2): i + n1 ≤ i + 16 ≤ ll.len() by the contract above
        let s2 = unsafe { ll.get_unchecked(i + n1..i + 16) };"""),
    ("""        let mut k = 0usize;
        loop {
            if COUNT {
                self.st.tile_refills_first += self.refills(t[k]) as u64;
                self.st.tile_refills_first_at[k] += self.refills(t[k]) as u64;
            }
            if !self.one(t[k], u.wrapping_add(k as u16)) {
                return false;
            }
            k += 1;
            if k >= n1 {
                break;
            }
        }""",
     """        let mut k = 0usize;
        let mut c = t[0];
        loop {
            if COUNT {
                self.st.tile_refills_first += self.refills(c) as u64;
                self.st.tile_refills_first_at[k] += self.refills(c) as u64;
            }
            if !self.one(c, u.wrapping_add(k as u16)) {
                return false;
            }
            k += 1;
            if k >= n1 {
                break;
            }
            c = s1[k];
        }"""),
]

name = sys.argv[1]
if name == 'MU1':
    sub("        let n1 = 16usize.min(ROWS.wrapping_sub(self.p));\n",
        "        let n1 = 16usize.min(ROWS.wrapping_sub(self.p));\n        let n1 = n1.max(17); // MUTANT MU1: n1 forced to 17 (tests U2 and range order only)\n")
elif name == 'MU3':
    sub("        let s2 = unsafe { ll.get_unchecked(i + n1..i + 16) };",
        "        let s2 = unsafe { ll.get_unchecked(i + n1..i + 17) }; // MUTANT MU3: U2 one past the tile")
elif name in ('variant-a', 'MU2'):
    for o, n in VARIANT_A:
        sub(o, n)
    if name == 'MU2':
        sub("        let s1 = unsafe { ll.get_unchecked(i..i + n1) };",
            "        let e1 = if i + 16 == ll.len() { ll.len() + 1 } else { i + n1 }; // MUTANT MU2\n"
            "        let s1 = unsafe { ll.get_unchecked(i..e1) }; // MUTANT MU2: U1's end past ll on a final full tile")
elif name == 'M1':
    sub("                    rest = r2;\n",
        "                    // MUTANT M1: the held tile is skipped\n"
        "                    if held.is_some() {\n                        held = None;\n                        rest = &r2[16..];\n"
        "                    } else {\n                        rest = r2;\n                    }\n")
elif name == 'M2':
    sub("                    if !w.run(a as u8, rest.len() - r2.len(), forced_cfg) {",
        "                    // MUTANT M2: the run's length short by one tile\n"
        "                    if !w.run(a as u8, rest.len() - r2.len() - 16, forced_cfg) {")
elif name == 'DM1':
    sub("        self.en[c] = w0.add((re - r) as usize);",
        "        self.en[c] = w0.add((re - r) as usize - 1); // MUTANT M1: end one field short")
elif name == 'DM2':
    sub("            *x = self.rank(c);",
        "            *x = self.cold[c].r0 + ((self.en[c] as usize).wrapping_sub(self.cold[c].w0 as usize) / 2) as u32; // MUTANT M2: rank from end")
elif name == 'V3M':
    sub("if d.raw & ORD != 0 {", "if d.raw == u32::MAX {", count=2)
else:
    sys.exit(f'unknown mutant {name}')
open(P, 'w').write(s)
print('applied', name)
