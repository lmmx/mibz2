include!("actions.rs");

impl Kernel {
    pub(super) fn expanded(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        book: &mut Book,
        out: &mut Vec<u8>,
    ) -> Result<(), mibz2::ErrorKind> {
        let (_, mark, bits) = self.construct::<true, true>(records, n as u32, p, 16);
        book.compile(&self.spelling, &self.offsets, n / 8);
        let mask = (1 << bits) - 1;
        let mut row = mark;
        let mut state = State::default();
        let mut remaining = n;
        out.reserve(n + 64);
        while remaining > 0 {
            let edge = self.edges[row as usize];
            let code = if self.extra.is_empty() {
                edge >> bits
            } else {
                self.extra[row as usize]
            };
            row = edge & mask;
            if code < 256 {
                state.byte(code as u8, out);
                remaining -= 1;
            } else {
                remaining -= book.write(
                    (code - 256) as usize,
                    &self.spelling,
                    &self.offsets,
                    remaining,
                    &mut state,
                    out,
                );
            }
        }
        if state.count == 4 {
            Err(mibz2::ErrorKind::RunUnfinished)
        } else {
            Ok(())
        }
    }
}
