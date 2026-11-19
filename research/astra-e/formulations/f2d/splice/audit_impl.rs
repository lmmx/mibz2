impl Candidate {
    pub fn audit(
        &mut self,
        r: &Runs,
        n: usize,
        p: u32,
        primary_out: &mut Vec<u8>,
        arena: &mut Audit,
    ) -> [usize; 10] {
        let (quote, m) = self.prepare(r.records(), n as u32).unwrap();
        let (q, skip) = self.transfer_mark(p, quote).unwrap();
        self.build(quote, m);
        self.inverse_merge();
        let sum_runs = self.merged.len();
        let old_root = &self.merged[self.nodes[0].out as usize..];
        let original = old_root.to_vec();
        let stats = arena
            .0
            .merge::<true>(&mut self.nodes, &self.selectors, &mut self.merged);
        assert_eq!(original.len(), self.merged.len());
        assert!(original
            .iter()
            .zip(&self.merged)
            .all(|(a, b)| a.id == b.id && a.width == b.width));
        assert_eq!(stats.terminals + stats.splits, self.merged.len());
        assert!(self.second.invert(
            &self.merged,
            &self.nodes,
            &self.words,
            &self.dict,
            q,
            skip,
            m,
            n as u32,
            primary_out
        ));
        [
            self.nodes.len(),
            self.selectors.len(),
            sum_runs,
            original.len(),
            stats.terminals,
            stats.splits,
            stats.whole_selectors,
            stats.partial_selectors,
            stats.partial_reads,
            arena.0.cells.capacity() * std::mem::size_of::<Link>(),
        ]
    }
}
#[derive(Default)]
pub struct Audit(Splice);
