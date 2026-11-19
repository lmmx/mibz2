// A tile header gives an F character and the first affine piece. Bits 0..20
// suffice for all piece indices because a piece contains at least one row.
#[derive(Default)]
pub(crate) struct AstraTiles { headers: Vec<u32>, pieces: Vec<u32> }
const TILE: u32 = 128;
const INDEX: u32 = (1<<20)-1;
const MIXED: u32 = 1<<20;

fn runs(records: &[u16], mut visit: impl FnMut(usize,u32,u32)) {
    let mut i = 0;
    let mut row = 0;
    while i < records.len() {
        let c = records[i] >> 8;
        let a = row;
        while i < records.len() && records[i] >> 8 == c {
            row += u32::from(records[i] & 255);
            i += 1;
        }
        visit(c as usize,a,row-a);
    }
}

impl AstraTiles {
    pub(crate) fn ranges(&self) {
        crate::vector("headers",&self.headers);
        crate::vector("pieces",&self.pieces);
    }
    fn build(&mut self, records: &[u16], counts: &[u32;256], n: u32) -> [u32;256] {
        let mut first = [0;256];
        let mut end = [0;256];
        let mut row = 0;
        for (c,&length) in counts.iter().enumerate() {
            first[c] = row;
            row += length;
            end[c] = row;
        }
        assert_eq!(row,n);
        let mut slots = [0u32;256];
        let mut next = first;
        runs(records,|c,_,len| {
            let q = next[c];
            slots[c] += (q + len-1)/TILE - q/TILE + 1;
            next[c] += len;
        });
        let mut total = 0;
        for slot in &mut slots {
            let k = *slot;
            *slot = total;
            total += k;
        }
        self.headers.resize(n.div_ceil(TILE) as usize,0);
        self.pieces.resize(total as usize,0);
        next = first;
        runs(records,|c,mut a,mut len| {
            let mut q = next[c];
            next[c] += len;
            while len != 0 {
                let local = q%TILE;
                let take = len.min(TILE-local);
                let idx = slots[c];
                if local == 0 {
                    let mixed = if end[c] < (q+TILE).min(n) { MIXED } else { 0 };
                    self.headers[(q/TILE) as usize] = idx | mixed | ((c as u32)<<24);
                }
                let bias = a+(TILE-1)-local;
                self.pieces[idx as usize] = ((local+take-1)<<24) | bias;
                slots[c] += 1;
                q += take;
                a += take;
                len -= take;
            }
        });
        end
    }

    #[inline]
    fn step(&self, row: u32, ends: &[u32;256]) -> (u8,u32) {
        let h = self.headers[(row/TILE) as usize];
        let local = row%TILE;
        let mut c = (h>>24) as usize;
        if h & MIXED != 0 {
            while row >= ends[c] { c += 1; }
        }
        let mut i = (h&INDEX) as usize;
        loop {
            let w = self.pieces[i];
            if w>>24 >= local { return (c as u8,(w&INDEX)+local-(TILE-1)); }
            i += 1;
        }
    }

    pub(crate) fn decode(&mut self, r: &crate::runs::Runs, n: usize, mut row: u32,
                        out: &mut Vec<u8>, k: Kernels, stored: u32) -> Result<u32,ErrorKind> {
        astra_mark(16);
        let ends = self.build(r.records(),r.astra_counts(),n as u32);
        out.reserve(n+64);
        let (mut crc,mut last,mut streak) = (!0,0,0);
        astra_mark(17);
        for base in (0..n).step_by(4096) {
            let start = out.len();
            for _ in base..(base+4096).min(n) {
                let (byte,next) = self.step(row,&ends);
                row = next;
                if streak == 4 {
                    out.resize(out.len()+byte as usize,last);
                    streak = 0;
                } else {
                    out.push(byte);
                    streak = if last == byte {streak+1} else {1};
                    last = byte;
                }
            }
            crc = crate::crc::astra_update(crc,&out[start..],k);
        }
        if streak == 4 { out.clear(); return Err(ErrorKind::RunUnfinished); }
        let computed = !crc;
        if computed == stored { Ok(computed) }
        else { out.clear(); Err(ErrorKind::BlockCrc{stored,computed}) }
    }
}
