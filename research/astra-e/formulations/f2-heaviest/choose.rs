    /// Each unresolved width bounds every terminal below it.
    fn heaviest(&mut self, section: u32, count: u32) -> (u32, u32) {
        use std::cmp::Reverse;
        let mut widths = [0u32; 256];
        let mut starts = [0u32; 256];
        let mut touched = [0u8; 256];
        self.heap.clear();
        self.heap.push((count, Reverse(section)));
        let (mut best_start, mut best_width) = (u32::MAX, 0);
        while let Some((width, Reverse(left))) = self.heap.pop() {
            if width < best_width {
                break;
            }
            let right = left + width;
            let mut nbytes = 0;
            let mut index = self.locate(left);
            while self.runs[index].start < right {
                let run = self.runs[index];
                let lo = left.max(run.start);
                let length = right.min(self.runs[index + 1].start) - lo;
                let byte = usize::from(run.byte);
                if widths[byte] == 0 {
                    starts[byte] = run.lf + (lo - run.start);
                    touched[nbytes] = run.byte;
                    nbytes += 1;
                }
                widths[byte] += length;
                index += 1;
            }
            for &byte in &touched[..nbytes] {
                let slot = usize::from(byte);
                let start = starts[slot];
                let frequency = std::mem::take(&mut widths[slot]);
                if frequency < best_width {
                    continue;
                }
                if (section..section+count).contains(&start) {
                    if frequency > best_width || start < best_start {
                        best_width = frequency;
                        best_start = start;
                    }
                } else {
                    self.heap.push((frequency, Reverse(start)));
                }
            }
        }
        debug_assert!(best_width > 0);
        (best_start, best_width)
    }
