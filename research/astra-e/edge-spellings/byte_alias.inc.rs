
// Routing samples only some bytes. The equality decision compares the complete spelling.
#[inline]
fn spelling_hash(word: &[u8]) -> usize {
    let n = word.len();
    let sample = if n < 4 {
        u64::from(word[0]) | (u64::from(word[n / 2]) << 8) | (u64::from(word[n - 1]) << 16)
    } else {
        let part = |at| u64::from(u32::from_le_bytes(word[at..at + 4].try_into().unwrap()));
        let ends = part(0) | (part(n - 4) << 32);
        ends ^ part((n - 4) / 2).wrapping_mul(0x9e3779b97f4a7c15)
    };
    let h = sample ^ (n as u64).wrapping_mul(0x9e3779b97f4a7c15);
    let h = (h ^ (h >> 33)).wrapping_mul(0xff51afd7ed558ccd);
    (h ^ (h >> 33)) as usize
}

impl Kernel {
    fn alias_spellings(&mut self, stats: &mut Counts) {
        let old_words = self.offsets.len() - 1;
        let slots = (old_words * 2).next_power_of_two();
        if old_words < 2 || slots > self.edges.len() {
            return;
        }
        let table = &mut self.edges[..slots];
        table.fill(0);
        let mut read = 0;
        let mut write = 0;
        let mut words = 0;
        for band in &mut self.bands {
            if band.code < 256 {
                continue;
            }
            let end = self.offsets[(band.code - 255) as usize] as usize;
            let source = &self.spelling[read..end];
            let mut slot = spelling_hash(source) & (slots - 1);
            let mut alias = 0;
            for _ in 0..32 {
                let owner = table[slot] as usize;
                if owner == 0 {
                    table[slot] = (words + 1) as u32;
                    break;
                }
                let start = self.offsets[owner - 1] as usize;
                let stop = self.offsets[owner] as usize;
                if &self.spelling[start..stop] == source {
                    alias = owner;
                    break;
                }
                slot = (slot + 1) & (slots - 1);
            }
            if alias == 0 {
                // A full probe window keeps this word as its own entry with its original
                // bytes. Before each copy write <= read, and all older owners end at write.
                if write != read {
                    self.spelling.copy_within(read..end, write);
                }
                write += end - read;
                words += 1;
                self.offsets[words] = write as u32;
                alias = words;
            }
            band.code = (255 + alias) as u32;
            // Even if compaction overwrote the current old offset, the next unread end
            // is one slot farther right. The saved end supplies its original start.
            read = end;
        }
        self.spelling.truncate(write);
        self.offsets.truncate(words + 1);
        stats.words = words;
        // The constructor overwrites every row of the former hash table before emission.
        // Rule.first is read only by endpoint append, which has finished for this block.
    }
}
