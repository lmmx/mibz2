// D046 source-level witnesses at 280643cef83b09ac89707b747670e8198b21b68e.
// These have NOT been compiled or run by the reviewer.
// Exercise separately under Miri with --features pair-runs.
// After a repair, convert them to rejection/checked-state regression tests.
use mibz2::pair_runs::{self, Build, PairScratch, PairTable};

fn default_table() {
    pair_runs::pair_start(&[], &PairTable::default(), 0);
    // first_byte reads first.get_unchecked(0), first.len() == 0.
}
fn wrong_histogram() {
    let mut counts = [0u32; 256];
    counts[0] = 1;
    pair_runs::build_pairs(&[1], &counts, &mut PairTable::default(), &[], &mut Vec::new());
    // All bucket slices are legal; pair_steps q = f[1] = 1, ll.len() = 1.
}
fn failed_run_fallback() {
    let mut counts = [0u32; 256];
    counts[0] = 1;
    pair_runs::setup(Build::Runs, &mut PairScratch::default(), &[1], &counts, 0, 1);
    // Run builder rejects histogram, then setup invokes the same unchecked row builder.
}
fn replace_prepared_table() {
    let mut counts = [0u32; 256];
    counts[0] = 17;
    let mut scratch = PairScratch::default();
    let r0 = pair_runs::setup(Build::Runs, &mut scratch, &[0; 17], &counts, 0, 2);
    scratch.t = PairTable::default();
    pair_runs::walk::<1, false>(&mut scratch, r0, &mut Vec::new());
    // Existing starts are nonempty; take dereferences the empty replacement table.
}
