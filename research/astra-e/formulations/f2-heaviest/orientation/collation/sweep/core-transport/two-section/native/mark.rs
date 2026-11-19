impl<const MODE: u8, const SCHEDULE: u8> Dispatch<MODE, SCHEDULE> {
    fn lift_mark(&self, p: u32, u: (u32, u32)) -> (Option<(u32, u32)>, u32) {
        let m = u.1 - u.0;
        // A coincident U/V row starts on V so that its first edge has positive weight.
        if let Some(ordinal) = self.plan.target.rank(p) {
            return (Some((ordinal, 0)), 0);
        }
        if (u.0..u.1).contains(&p) {
            return (Some((m + (p - u.0), 0)), 0);
        }
        if let Some((source, distance)) = self.plan.hidden_mark {
            // p lies strictly between its own U source and its physical cut.
            return (Some((m + source, distance)), 0);
        }
        let start = self.plan.forbid.complement(p);
        let mut x = start;
        let mut distance = 0;
        loop {
            let run = self.runs[self.locate(x)];
            x = run.map(x);
            distance += 1;
            if run.terminal() {
                return (Some((x, distance)), 0);
            }
            if x == start {
                return (None, start);
            }
        }
    }
}
