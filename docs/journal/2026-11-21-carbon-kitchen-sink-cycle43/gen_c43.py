import os, shutil
from gen_c42 import patch_s3_arr, patch_band_width

ROOT = "/tmp/cks_c36"
BASE = f"{ROOT}/cand_c41_combo"
BASE_NOBI = f"{ROOT}/cand_c41_nobi41"

def clone_from(src, name):
    dst = f"{ROOT}/{name}"
    if os.path.exists(dst):
        shutil.rmtree(dst)
    shutil.copytree(src, dst)
    return dst

def patch_s2b_narrow_slice(dst):
    """
    Specialized `eliminate_narrow_plain` for `Narrow` (`L::STRIDE == 4`, `100%` of Wikidata!):
    Single-slice `&self.a[xi..xi+4]` & `&self.a[yi..yi+4]` reads/writes cut 16 bounds checks -> 7
    and eliminate 3 redundant memory loads per step (`prev(x,0)`, `next(y,0)`, `y[0]` RMW) + fuse `x[0]` links!
    """
    ep_p = f"{dst}/endpoint.rs"
    with open(ep_p) as f:
        e = f.read()

    # Add `const IS_NARROW: bool` to Links trait so eliminate::<false> dispatches cleanly at compile time!
    e = e.replace("    const STRIDE: usize;", "    const STRIDE: usize;\n    const IS_NARROW: bool;", 1)
    e = e.replace("    const STRIDE: usize = 4;", "    const STRIDE: usize = 4;\n    const IS_NARROW: bool = true;", 1)
    e = e.replace("    const STRIDE: usize = 6;", "    const STRIDE: usize = 6;\n    const IS_NARROW: bool = false;", 1)

    old_elim_start = """    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
        let (mut left, mut taken) = (*remaining, *steps);
        let room = if SPIRALS { SPIRAL_RULES * RULE } else { RULE };
        let done = loop {"""

    new_elim_start = """    fn eliminate_narrow_plain(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + RULE > self.a.len() {
                break false;
            }
            let sent = &self.a[..2];
            let x = sent[0] & 0xFFFF;
            let y = sent[1] & 0xFFFF;
            let xi = x as usize * 4;
            let yi = y as usize * 4;
            let nx = &self.a[xi..xi + 4];
            let (x0, x1, aw, wx) = (nx[0], nx[1], nx[2], nx[3]);
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let px0 = x0 & 0xFFFF;
                let px1 = x1 & 0xFFFF;
                self.a[px0 as usize * 4] &= 0xFFFF;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | px0;
                self.a[px1 as usize * 4 + 1] &= 0xFFFF;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | px1;
                continue;
            }
            let ny = &self.a[yi..yi + 4];
            let (y0, y1, bw, wy) = (ny[0], ny[1], ny[2], ny[3]);
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            let word = self.join(wx, wy);
            if aw <= bw {
                let px = x0 & 0xFFFF;
                self.a[px as usize * 4] &= 0xFFFF;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | px;
                if aw < bw {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y | (y_next << 16);
                    nxm[3] = word;
                    let nym = &mut self.a[yi..yi + 4];
                    nym[0] = (y0 & 0xFFFF) | (x << 16);
                    nym[2] = bw - aw;
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                } else {
                    // aw == bw: insert x after y on side 0, cut y from side 0, pop y from tail of side 1
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let y_prev = y0 & 0xFFFF;
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y_prev | (y_next << 16);
                    nxm[3] = word;
                    let ypi = y_prev as usize * 4;
                    self.a[ypi] = (self.a[ypi] & 0xFFFF) | (x << 16);
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                    let py = y1 & 0xFFFF;
                    self.a[py as usize * 4 + 1] &= 0xFFFF;
                    self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                }
            } else {
                let py = y1 & 0xFFFF;
                self.a[py as usize * 4 + 1] &= 0xFFFF;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                let x_next = if x == py { 0 } else { x1 >> 16 };
                let nym = &mut self.a[yi..yi + 4];
                nym[1] = x | (x_next << 16);
                nym[3] = word;
                let nxm = &mut self.a[xi..xi + 4];
                nxm[1] = (x1 & 0xFFFF) | (y << 16);
                nxm[2] = aw - bw;
                let xni = x_next as usize * 4 + 1;
                self.a[xni] = (self.a[xni] & 0xFFFF_0000) | y;
            }
        };
        *remaining = left;
        done
    }

    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
        if !SPIRALS && L::IS_NARROW {
            return self.eliminate_narrow_plain(remaining);
        }
        let (mut left, mut taken) = (*remaining, *steps);
        let room = if SPIRALS { SPIRAL_RULES * RULE } else { RULE };
        let done = loop {"""

    assert old_elim_start in e
    e = e.replace(old_elim_start, new_elim_start, 1)
    with open(ep_p, "w") as f:
        f.write(e)

def patch_wire_serial16(dst):
    """16-wide u64-unpacked serial OutQ16::wire for nobi mode."""
    outq_p = f"{dst}/outq.rs"
    with open(outq_p) as f:
        o = f.read()
    old_w = "            let mut i = 0usize;\n            while i + 8 <= span {"
    new_w = """            let mut i = 0usize;
            while i + 16 <= span {
                let b = &bytes[2 * i..2 * i + 32];
                let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                let q2 = u64::from_le_bytes([b[16], b[17], b[18], b[19], b[20], b[21], b[22], b[23]]);
                let q3 = u64::from_le_bytes([b[24], b[25], b[26], b[27], b[28], b[29], b[30], b[31]]);
                let d = &mut dst[i..i + 16];
                d[0] = hi | (q0 as u32 & 0xFFFF);  d[1] = hi | ((q0 >> 16) as u32 & 0xFFFF);
                d[2] = hi | ((q0 >> 32) as u32 & 0xFFFF); d[3] = hi | ((q0 >> 48) as u32);
                d[4] = hi | (q1 as u32 & 0xFFFF);  d[5] = hi | ((q1 >> 16) as u32 & 0xFFFF);
                d[6] = hi | ((q1 >> 32) as u32 & 0xFFFF); d[7] = hi | ((q1 >> 48) as u32);
                d[8] = hi | (q2 as u32 & 0xFFFF);  d[9] = hi | ((q2 >> 16) as u32 & 0xFFFF);
                d[10] = hi | ((q2 >> 32) as u32 & 0xFFFF); d[11] = hi | ((q2 >> 48) as u32);
                d[12] = hi | (q3 as u32 & 0xFFFF); d[13] = hi | ((q3 >> 16) as u32 & 0xFFFF);
                d[14] = hi | ((q3 >> 32) as u32 & 0xFFFF); d[15] = hi | ((q3 >> 48) as u32);
                i += 16;
            }
            if i + 8 <= span {"""
    assert old_w in o
    o = o.replace(old_w, new_w, 1)
    with open(outq_p, "w") as f:
        f.write(o)

# Build clean Cycle 42 baseline (S3Arr + BandW without Pair2!) and Cycle 43 candidates:
d = clone_from(BASE, "cand_c43_base");      patch_s3_arr(d); patch_band_width(d)
d = clone_from(BASE, "cand_c43_s2b");       patch_s3_arr(d); patch_band_width(d); patch_s2b_narrow_slice(d)
d = clone_from(BASE_NOBI, "cand_c43_nb42"); patch_s3_arr(d); patch_band_width(d)
d = clone_from(BASE_NOBI, "cand_c43_nb43"); patch_s3_arr(d); patch_band_width(d); patch_s2b_narrow_slice(d)
d = clone_from(BASE_NOBI, "cand_c43_nbw16"); patch_s3_arr(d); patch_band_width(d); patch_s2b_narrow_slice(d); patch_wire_serial16(d)

print("Generated 5 Cycle 43 candidates successfully!")
