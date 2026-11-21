include!("contract.rs");

impl Kernel {
    pub fn pairs<const PROBE: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) {
        let (counts, mark, bits) = self.construct::<true, true>(records, n as u32, p, 16);
        let first_pair = 256 + counts.words as u32;
        let fits = self.extra.is_empty() && first_pair + PAIR_CODES - 1 <= u32::MAX >> bits;
        let mask = (1 << bits) - 1;
        let mut initial_hops = 0;
        if PROBE {
            let mut row = mark;
            loop {
                initial_hops += 1;
                row = self.edges[row as usize] & mask;
                if row == mark {
                    break;
                }
            }
        }
        let (lookups, rewritten) = if fits {
            contract::<PROBE>(&mut self.edges, mark, bits, first_pair)
        } else {
            (0, 0)
        };

        // In the no-room arm compile out both the pair test and the probe counters.
        let final_hops = if fits {
            self.emit_pairs::<true, PROBE>(mark, n, bits, first_pair, out)
        } else {
            self.emit_pairs::<false, PROBE>(mark, n, bits, first_pair, out)
        };
        if PROBE {
            eprintln!("pairs,{n},{p},{mark},{bits},{},{},{},{lookups},{rewritten},{initial_hops},{final_hops}",
                      counts.retained, counts.words, u8::from(fits));
        }
    }

    fn emit_pairs<const PAIRS: bool, const PROBE: bool>(
        &self,
        mark: u32,
        n: usize,
        bits: u32,
        first_pair: u32,
        out: &mut Vec<u8>,
    ) -> usize {
        let mask = (1 << bits) - 1;
        out.clear();
        out.reserve(n);
        let mut row = mark;
        let mut hops = 0;
        loop {
            let edge = self.edges[row as usize];
            let code = if PAIRS || self.extra.is_empty() {
                edge >> bits
            } else {
                self.extra[row as usize]
            };
            row = edge & mask;
            if code < 256 {
                out.push(code as u8);
            } else if PAIRS && code >= first_pair {
                let pair = code - first_pair;
                out.extend_from_slice(&[(pair / ALPHABET) as u8, (pair % ALPHABET) as u8]);
            } else {
                let w = (code - 256) as usize;
                out.extend_from_slice(
                    &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize],
                );
            }
            if PROBE {
                hops += 1;
            }
            if row == mark {
                break;
            }
        }
        assert!(!out.is_empty() && out.len() <= n);
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
        hops
    }
}
