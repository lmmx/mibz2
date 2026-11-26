// Store only a certified transparent word's final literal count.
// A zero hint takes the ordinary scalar word transition in any incoming state.
#[derive(Default)]
pub(super) struct Book {
    hints: Vec<u8>,
}
impl Book {
    fn compile(&mut self, spelling: &[u8], offsets: &[u32], _budget: usize) {
        self.hints.clear();
        self.hints.reserve(offsets.len().saturating_sub(1));
        for pair in offsets.windows(2) {
            let w = &spelling[pair[0] as usize..pair[1] as usize];
            let opaque = w
                .windows(4)
                .any(|v| v[0] == v[1] && v[0] == v[2] && v[0] == v[3]);
            let tail = if opaque {
                0
            } else {
                let last = *w.last().unwrap();
                w.iter().rev().take_while(|&&b| b == last).count() as u8
            };
            self.hints.push(tail);
        }
    }
    #[inline]
    fn write(
        &self,
        i: usize,
        spelling: &[u8],
        offsets: &[u32],
        remaining: usize,
        state: &mut State,
        out: &mut Vec<u8>,
    ) -> usize {
        let w = &spelling[offsets[i] as usize..offsets[i + 1] as usize];
        let hint = self.hints[i];
        if hint != 0
            && w.len() <= remaining
            && (state.count == 0 || (state.count < 4 && state.last != w[0]))
        {
            out.extend_from_slice(w);
            *state = State {
                last: *w.last().unwrap(),
                count: hint,
            };
            w.len()
        } else {
            let take = w.len().min(remaining);
            state.feed(&w[..take], out);
            take
        }
    }
}
