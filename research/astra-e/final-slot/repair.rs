use mibz2::{ErrorKind, Kernels};

const MAX_EDITS: usize = 512;

// A block has <2^20 encoded bytes. Even the maximum expansion, 259 output
// bytes per five coded bytes, leaves the destination safely below 2^28.
#[derive(Clone, Copy)]
struct Edit(u64);
impl Edit {
    fn new(src: usize, dst: usize, byte: u8, extra: u8) -> Self {
        Self(src as u64 | ((dst as u64) << 20) | (u64::from(byte) << 48) | (u64::from(extra) << 56))
    }
    fn src(self) -> usize {
        (self.0 as usize) & 0xfffff
    }
    fn dst(self) -> usize {
        ((self.0 >> 20) & 0xfffffff) as usize
    }
    fn byte(self) -> u8 {
        (self.0 >> 48) as u8
    }
    fn len(self) -> usize {
        4 + (self.0 >> 56) as usize
    }
}

#[derive(Default)]
pub(super) struct Directory(Vec<Edit>);

#[derive(Default)]
pub(super) struct Costs {
    pub scanned: usize,
    pub left: usize,
    pub right: usize,
    pub filled: usize,
}
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Strategy {
    Applied,
    DirectoryFull,
    NeedsCapacity,
    Oversize,
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1")]
unsafe fn quad_avx2(s: &[u8], mut i: usize) -> Option<usize> {
    use std::arch::x86_64::*;
    while i + 35 <= s.len() {
        // SAFETY: i..i+35 contains all four unaligned 32-byte loads.
        let p = s.as_ptr().add(i);
        let v = _mm256_loadu_si256(p.cast());
        let eq = |j| _mm256_cmpeq_epi8(v, _mm256_loadu_si256(p.add(j).cast()));
        let mask =
            _mm256_movemask_epi8(_mm256_and_si256(eq(1), _mm256_and_si256(eq(2), eq(3)))) as u32;
        if mask != 0 {
            return Some(i + mask.trailing_zeros() as usize);
        }
        i += 32;
    }
    quad_scalar(s, i)
}
fn quad_scalar(s: &[u8], i: usize) -> Option<usize> {
    s[i..]
        .windows(4)
        .position(|w| w[0] == w[1] && w[0] == w[2] && w[0] == w[3])
        .map(|j| i + j)
}
fn next_quad(s: &[u8], i: usize, k: Kernels) -> Option<usize> {
    #[cfg(target_arch = "x86_64")]
    if k.avx2() {
        // SAFETY: Kernels::avx2 also requires BMI1 on x86_64.
        return unsafe { quad_avx2(s, i) };
    }
    let _ = k;
    quad_scalar(s, i)
}

impl Directory {
    pub(super) fn len(&self) -> usize {
        self.0.len()
    }
    pub(super) fn capacity_bytes(&self) -> usize {
        self.0.capacity() * std::mem::size_of::<Edit>()
    }

    pub(super) fn apply<const PROBE: bool>(
        &mut self,
        data: &mut Vec<u8>,
        k: Kernels,
        costs: &mut Costs,
    ) -> Result<Strategy, ErrorKind> {
        self.0.clear();
        let n = data.len();
        if n >= 1 << 20 {
            return Ok(Strategy::Oversize);
        }
        let (mut src, mut dst) = (0, 0);
        while let Some(j) = next_quad(data, src, k) {
            let extra = *data.get(j + 4).ok_or(ErrorKind::RunUnfinished)?;
            if self.0.len() == MAX_EDITS {
                if PROBE {
                    costs.scanned = j + 5;
                }
                return Ok(Strategy::DirectoryFull);
            }
            let q = dst + (j - src);
            self.0.push(Edit::new(j, q, data[j], extra));
            src = j + 5;
            dst = q + 4 + usize::from(extra);
        }
        if PROBE {
            costs.scanned = n;
        }
        let m = dst + (n - src);
        if m > data.capacity() {
            return Ok(Strategy::NeedsCapacity);
        }
        if self.0.is_empty() {
            return Ok(Strategy::Applied);
        }
        if m > n {
            data.resize(m, 0);
        }

        // Literal spans with negative displacement move in increasing order.
        src = 0;
        dst = 0;
        for &e in &self.0 {
            if dst < src {
                data.copy_within(src..e.src(), dst);
                if PROBE {
                    costs.left += e.src() - src;
                }
            }
            src = e.src() + 5;
            dst = e.dst() + e.len();
        }
        if dst < src {
            data.copy_within(src..n, dst);
            if PROBE {
                costs.left += n - src;
            }
        }
        // Positive displacements move in decreasing source order, after the
        // negative spans. Only then may fills overwrite their fixed destinations.
        let mut end = n;
        for &e in self.0.iter().rev() {
            src = e.src() + 5;
            dst = e.dst() + e.len();
            if dst > src {
                data.copy_within(src..end, dst);
                if PROBE {
                    costs.right += end - src;
                }
            }
            end = e.src();
        }
        for &e in &self.0 {
            data[e.dst()..e.dst() + e.len()].fill(e.byte());
            if PROBE {
                costs.filled += e.len();
            }
        }
        data.truncate(m);
        Ok(Strategy::Applied)
    }
}
