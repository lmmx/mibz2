impl<const MODE: u8, const SCHEDULE: u8> Dispatch<MODE, SCHEDULE> {
    fn physical_mark(&self, p: u32, u: (u32, u32)) -> (Option<(u32, u32)>, u32) {
        if let Some(ordinal) = self.plan.target.rank(p) {
            return (Some((ordinal, 0)), 0);
        }
        // A hidden nonterminal lies strictly between its original source and cut.
        // No physical cut intervenes on this backward fragment.
        let (row, mut skip) = match self.plan.hidden_mark {
            Some((source, distance)) => (u.0 + source, distance),
            None => (p, 0),
        };
        let start = self.plan.forbid.complement(row);
        let mut x = start;
        loop {
            let run = self.runs[self.locate(x)];
            x = run.map(x);
            skip += 1;
            if run.terminal() {
                return (Some((x, skip)), 0);
            }
            if x == start {
                // A disjoint cycle cannot contain an atlas cut or stripped prefix.
                return (None, start);
            }
        }
    }
}
