from pathlib import Path

base_mod = Path("/tmp/cks_bench/master_a1/mod.rs").read_text()
base_ep = Path("/tmp/cks_bench/master_a1/endpoint.rs").read_text()
base_oq = Path("/tmp/cks_bench/master_a1/outq.rs").read_text()

# 1. `cand_ostrowski`: Unifies `eliminate` into ONE single loop using 1-cycle shift guard `aw > (bw << 2)` (`q >= 4`),
#    eliminating `CERTIFIED_WIDTH` branching/code duplication while protecting all blocks!
ost_ep = base_ep.replace(
    "if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= 2 {",
    "if aw > (bw << 2) && L::prev(self.a, y, 1) == x {"
).replace(
    "if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {",
    "if bw > (aw << 2) && L::prev(self.a, x, 0) == y {"
).replace(
    "let spirals = widest > CERTIFIED_WIDTH;",
    "let _ = widest;\n        let spirals = true;"
)
d_ost = Path("/tmp/cks_bench/cand_ostrowski")
d_ost.mkdir(parents=True, exist_ok=True)
(d_ost / "mod.rs").write_text(base_mod)
(d_ost / "endpoint.rs").write_text(ost_ep)
(d_ost / "outq.rs").write_text(base_oq)

# 2. `cand_q256` (`CHUNK = 256` -> `512 B` per chunk, pulling 90 active tail chunks = 45 KiB inside L1D!)
#    and `cand_q1024` (`CHUNK = 1024` -> `2 KiB` per chunk, halving chunk transitions!)
for cname, cval in [("cand_q128", 128), ("cand_q256", 256), ("cand_q1024", 1024)]:
    d = Path(f"/tmp/cks_bench/{cname}")
    d.mkdir(parents=True, exist_ok=True)
    (d / "mod.rs").write_text(base_mod)
    (d / "endpoint.rs").write_text(base_ep)
    (d / "outq.rs").write_text(base_oq.replace("const CHUNK: usize = 512;", f"const CHUNK: usize = {cval};"))

print("Generated cand_ostrowski, cand_q128, cand_q256, cand_q1024.")
