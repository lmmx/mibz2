    fn len(&self, w: u32) -> u32 {
        match w >> 30 {
            0 => u32::from(w != 0),
            1 => 2,
            2 => 3,
            _ => self.rules[(w & 0x3fff_ffff) as usize].len,
        }
    }

    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 { return right; }
        if right == 0 { return left; }
        // Leave the original single-letter representation available to node()
        // and distinguish actual rules from two- and three-byte immediates.
        let (lt, rt) = (left >> 30, right >> 30);
        match (lt, rt) {
            (0, 0) => (1 << 30) | (left - 1) | ((right - 1) << 8),
            (0, 1) => (2 << 30) | (left - 1) | ((right & 0xffff) << 8),
            (1, 0) => (2 << 30) | (left & 0xffff) | ((right - 1) << 16),
            _ => {
                let id = (3 << 30) | self.rules.len() as u32;
                self.rules.push(Rule {
                    left,
                    right,
                    len: self.len(left) + self.len(right),
                    first: u32::MAX,
                });
                id
            }
        }
    }

