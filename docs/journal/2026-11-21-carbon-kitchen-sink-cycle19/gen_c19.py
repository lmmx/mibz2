from pathlib import Path

# Generate a byte-stratified threshold module `cand_strat` from `master_a1` where `try_run_table` accepts `limits: &[u32; 256]`!
base_mod = Path("/tmp/cks_bench/master_a1/mod.rs").read_text()
base_ep = Path("/tmp/cks_bench/master_a1/endpoint.rs").read_text()
base_oq = Path("/tmp/cks_bench/master_a1/outq.rs").read_text()

strat_mod = base_mod.replace(
    "    fn take<const QUEUE: bool>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {",
    "    fn take<const QUEUE: bool>(&mut self, c: u8, pos: u32, width: u32, p: u32, limits: &[u32; 256]) {\n        let limit = limits[c as usize];"
).replace(
    "pass.take::<QUEUE>(c, pos, width, p, limit)",
    "pass.take::<QUEUE>(c, pos, width, p, limits)"
).replace(
    "    fn construct_capped<const INDUCE: bool, const QUEUE: bool>(\n        &mut self,\n        records: &[u16],\n        n: u32,\n        p: u32,\n        limit: u32,",
    "    fn construct_capped<const INDUCE: bool, const QUEUE: bool>(\n        &mut self,\n        records: &[u16],\n        n: u32,\n        p: u32,\n        limits: &[u32; 256],"
).replace(
    "Some(self.finish::<INDUCE, QUEUE>(&f, records, n, p, limit, &out[..], &heads))",
    "Some(self.finish::<INDUCE, QUEUE>(&f, records, n, p, limits[0], &out[..], &heads))"
).replace(
    "    pub fn try_run<const INDUCE: bool, const QUEUE: bool>(\n        &mut self,\n        records: &[u16],\n        n: usize,\n        p: u32,\n        limit: u32,",
    "    pub fn try_run<const INDUCE: bool, const QUEUE: bool>(\n        &mut self,\n        records: &[u16],\n        n: usize,\n        p: u32,\n        limits: &[u32; 256],"
).replace(
    "self.construct_capped::<INDUCE, QUEUE>(records, n as u32, p, limit, cap, out)?;",
    "self.construct_capped::<INDUCE, QUEUE>(records, n as u32, p, limits, cap, out)?;"
).replace(
    "    pub fn run_entry(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, out: &mut Vec<u8>) -> Option<Counts> {\n        self.try_run::<true, true>(records, n, p, LIMIT, cap, out)\n    }",
    "    pub fn run_entry(&mut self, records: &[u16], n: usize, p: u32, limits: &[u32; 256], cap: Cap, out: &mut Vec<u8>) -> Option<Counts> {\n        self.try_run::<true, true>(records, n, p, limits, cap, out)\n    }"
).replace(
    "    pub fn run_stages(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, out: &mut Vec<u8>) -> (f64, f64) {\n        let t0 = std::time::Instant::now();\n        let (stats, mark, bits) = self.construct_capped::<true, true>(records, n as u32, p, LIMIT, cap, out).unwrap();",
    "    pub fn run_stages(&mut self, records: &[u16], n: usize, p: u32, limits: &[u32; 256], cap: Cap, out: &mut Vec<u8>) -> (f64, f64) {\n        let t0 = std::time::Instant::now();\n        let (stats, mark, bits) = self.construct_capped::<true, true>(records, n as u32, p, limits, cap, out).unwrap();"
)

# Also allow `cand_mn16_direct` to accept an arbitrary `lim: u32` so we test Epistatic `MN16-Direct @ LIMIT = 12, 14, 16, 20`!
mn_mod = Path("/tmp/cks_bench/cand_mn16_direct/mod.rs").read_text()
if "pub fn run_lim" not in mn_mod:
    mn_mod = mn_mod.replace(
        "    pub fn run_entry(",
        """    #[inline(never)]
    pub fn run_lim(&mut self, records: &[u16], n: usize, p: u32, lim: u32, cap: Cap, out: &mut Vec<u8>) -> Option<Counts> {
        self.try_run::<true, true>(records, n, p, lim, cap, out)
    }
    pub fn run_entry("""
    )
    Path("/tmp/cks_bench/cand_mn16_direct/mod.rs").write_text(mn_mod)

m1_mod = Path("/tmp/cks_bench/master_a1/mod.rs").read_text()
if "pub fn run_lim" not in m1_mod:
    m1_mod = m1_mod.replace(
        "    pub fn run_entry(",
        """    #[inline(never)]
    pub fn run_lim(&mut self, records: &[u16], n: usize, p: u32, lim: u32, cap: Cap, out: &mut Vec<u8>) -> Option<Counts> {
        self.try_run::<true, true>(records, n, p, lim, cap, out)
    }
    pub fn run_entry("""
    )
    Path("/tmp/cks_bench/master_a1/mod.rs").write_text(m1_mod)

d_st = Path("/tmp/cks_bench/cand_strat")
d_st.mkdir(parents=True, exist_ok=True)
(d_st / "mod.rs").write_text(strat_mod)
(d_st / "endpoint.rs").write_text(base_ep)
(d_st / "outq.rs").write_text(base_oq)
print("Generated cand_strat and run_lim helpers.")
