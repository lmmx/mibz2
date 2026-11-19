    // Walk the old suffix in last-first order. Keep all grammar joins in their
    // original order and reconnect the suffix only after the last strict loss.
    fn strict_suffix<const SIDE: usize>(
        &mut self,
        winner: u16,
        tail: u16,
        remaining: u32,
    ) -> (u32, usize) {
        let win = self.nodes[winner as usize];
        let after_winner = win.next[SIDE];
        let budget = remaining.min(win.width - 1);
        let mut available = budget;
        let mut count = 0;
        let mut loser = tail;
        let first = loop {
            let node = self.nodes[loser as usize];
            debug_assert!(node.width <= available && loser != winner);
            self.nodes[loser as usize].word = if SIDE == 0 {
                self.join(node.word, win.word)
            } else {
                self.join(win.word, node.word)
            };
            available -= node.width;
            count += 1;
            if loser == after_winner || available == 0 {
                break loser;
            }
            let prev = node.prev[SIDE];
            if self.nodes[prev as usize].width > available {
                break loser;
            }
            loser = prev;
        };
        let width = budget - available;
        self.nodes[winner as usize].width -= width;
        if first != after_winner {
            let before = self.nodes[first as usize].prev[SIDE];
            self.nodes[before as usize].next[SIDE] = 0;
            self.nodes[0].prev[SIDE] = before;
            self.nodes[winner as usize].next[SIDE] = first;
            self.nodes[first as usize].prev[SIDE] = winner;
            self.nodes[tail as usize].next[SIDE] = after_winner;
            self.nodes[after_winner as usize].prev[SIDE] = tail;
        }
        // OBSERVE_STRICT
        (width, count)
    }

    fn eliminate(&mut self, mut remaining: u32) -> usize {
        let mut steps = 0;
        while remaining > 0 {
            let [x, y] = self.nodes[0].prev;
            let a = self.nodes[x as usize];
            let b = self.nodes[y as usize];
            let w = a.width.min(b.width);
            assert!(w > 0 && w <= remaining);
            if x == y {
                // OBSERVE_SELF
                self.cut(x, 0);
                self.cut(x, 1);
                remaining -= w;
                steps += 1;
            } else if a.width == b.width {
                // OBSERVE_EQUAL
                let word = self.join(a.word, b.word);
                self.cut(x, 0);
                self.insert(y, x, 0);
                self.nodes[x as usize].word = word;
                self.cut(y, 0);
                self.cut(y, 1);
                remaining -= w;
                steps += 1;
            } else {
                let (removed, count) = if a.width < b.width {
                    self.strict_suffix::<0>(y, x, remaining)
                } else {
                    self.strict_suffix::<1>(x, y, remaining)
                };
                remaining -= removed;
                steps += count;
            }
        }
        steps
    }

