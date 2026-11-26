// Each bit at offset j-1 records the start of a run j rows inside the page.
// A run starting exactly at the page boundary is accounted for by the base.
const BASE_SHIFT: u32 = 44;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct WidePage {
    cuts: u64,
    base: u32,
}

#[derive(Default)]
struct RankPages {
    packed: Vec<u64>,
    wide: Vec<WidePage>,
}

impl RankPages {
    fn clear<const WIDTH: u32>(&mut self, rows: u32) {
        let len = rows.div_ceil(WIDTH) as usize;
        if WIDTH == 64 {
            self.wide.clear();
            self.wide.resize(len, WidePage::default());
        } else {
            self.packed.clear();
            self.packed.resize(len, 0);
        }
    }

    #[inline]
    fn cut<const WIDTH: u32>(&mut self, row: u32) {
        let offset = row % WIDTH;
        if offset != 0 {
            let cut = 1u64 << (offset - 1);
            let page = (row / WIDTH) as usize;
            if WIDTH == 64 {
                self.wide[page].cuts |= cut;
            } else {
                self.packed[page] |= cut;
            }
        }
    }

    fn bases<const WIDTH: u32>(&mut self, rows: u32, runs: &[Run]) {
        let mut k = 0;
        for page in 0..rows.div_ceil(WIDTH) {
            while runs[k + 1].start <= page * WIDTH {
                k += 1;
            }
            if WIDTH == 64 {
                self.wide[page as usize].base = k as u32;
            } else {
                self.packed[page as usize] |= (k as u64) << BASE_SHIFT;
            }
        }
    }

    #[inline]
    fn locate<const WIDTH: u32>(&self, x: u32) -> usize {
        let page = (x / WIDTH) as usize;
        let prefix = (1u64 << (x % WIDTH)) - 1;
        if WIDTH == 64 {
            let entry = self.wide[page];
            (entry.base + (entry.cuts & prefix).count_ones()) as usize
        } else {
            let entry = self.packed[page];
            ((entry >> BASE_SHIFT) as u32 + (entry & prefix).count_ones()) as usize
        }
    }
}
