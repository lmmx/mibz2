
// Additional correctness gate on the unfiltered stream. Even column-route and
// refused blocks are explicitly sent through each candidate's Open kernel.
#[test]
#[ignore]
fn forced_parity() {
    fn check<K: InverseBwt + Default>(warm: &mut K, r: &Runs, want: &[u8], p: u32, out: &mut Vec<u8>) {
        for k in [warm, &mut K::default()] {
            assert!(k.invert_runs(r, want.len(), p, out, Kernels::detect()));
            assert_eq!(out.len(), want.len());
            assert!(out == want);
        }
    }
    let blocks = load_bz2(&std::env::var("ADMIT_BZ2").unwrap());
    let (mut a, mut b) = (xor::ForcedOpenCells::default(), relay6::ForcedOpenCells::default());
    let mut out = Vec::new();
    for block in &blocks {
        let (col, p) = match block {
            Blk::Run { runs, p, .. } => (column(runs), *p),
            Blk::Col { ll, p, .. } => (ll.clone(), *p),
        };
        let runs = Runs::of_column(&col);
        let want = plain(&col, p);
        check(&mut a, &runs, &want, p, &mut out);
        check(&mut b, &runs, &want, p, &mut out);
    }
    assert_eq!(a.opened, 431);
    assert_eq!(b.opened, 431);
}
