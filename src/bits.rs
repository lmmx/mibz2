//! An MSB-first bit reader over a byte slice. Past the slice's end it reads zeros, and
//! says it has (`overran`): the caller then either has more input to give it (and starts
//! the item again) or the stream is truncated. So the decoder's loops never index past
//! the input, and an item cut short by the input's end is never taken as whole.

pub struct Bits<'a> {
    data: &'a [u8],
    r: Reg,
}

/// A reader's state apart from its slice: where it is and the bits it holds. A loop that
/// copies it into a local, hands it by value to anything not inlined and writes it back
/// when done keeps it in registers. Through a `&mut Bits` that the loop also passes to a
/// call it makes (a cold path), it lives in memory: a store and a reload every step, on
/// the step-to-step chain (block.rs's symbol loop).
#[derive(Clone, Copy)]
pub(crate) struct Reg {
    /// the next byte to load
    pos: usize,
    /// bits, MSB first: the top `live` are valid (the bits below them are the input's
    /// next bits or zeros, never anything else)
    buf: u64,
    live: u32,
}

impl Reg {
    /// At least 56 bits live: the next 8 bytes loaded whole, without a test of how many
    /// bits are live (the bits below the live ones are the input's next bits already, so
    /// or-ing them in again changes nothing); a byte at a time near the slice's end
    #[inline(always)]
    pub(crate) fn refill(&mut self, data: &[u8]) {
        if let Some(w) = data.get(self.pos..self.pos + 8) {
            let v = u64::from_be_bytes(w.try_into().unwrap());
            self.buf |= v >> self.live;
            let take = (63 - self.live) >> 3;
            self.pos += take as usize;
            self.live += take * 8;
        } else {
            *self = self.refill_slow(data);
        }
    }

    /// (by value, so that a caller's state stays in registers around it)
    #[cold]
    #[inline(never)]
    fn refill_slow(mut self, data: &[u8]) -> Reg {
        while self.live <= 56 {
            let b = data.get(self.pos).copied().unwrap_or(0);
            self.pos += 1;
            self.buf |= (b as u64) << (56 - self.live);
            self.live += 8;
        }
        self
    }

    /// The next `n` bits (1 to 32), not consumed
    #[inline(always)]
    pub(crate) fn peek(&mut self, data: &[u8], n: u32) -> u32 {
        debug_assert!((1..=32).contains(&n));
        if self.live < n {
            self.refill(data);
        }
        (self.buf >> (64 - n)) as u32
    }

    /// The next `n` bits (1 to 32), not consumed, where at least `n` are live (after
    /// `refill`, and fewer consumed since than it leaves over `n`)
    #[inline(always)]
    pub(crate) fn peek_live(&self, n: u32) -> u32 {
        debug_assert!((1..=32).contains(&n) && n <= self.live);
        (self.buf >> (64 - n)) as u32
    }

    /// Consume `n` bits (below 64, and at most those live)
    #[inline(always)]
    pub(crate) fn skip_short(&mut self, n: u32) {
        debug_assert!(n < 64 && n <= self.live);
        self.buf <<= n & 63;
        self.live -= n;
    }

    /// Consume `n` bits (at most those peeked)
    #[inline(always)]
    fn skip(&mut self, n: u32) {
        debug_assert!(n <= self.live);
        self.buf = if n >= 64 { 0 } else { self.buf << n };
        self.live -= n;
    }

    /// Bits consumed from the slice's start
    #[inline(always)]
    fn bit_pos(&self) -> u64 {
        self.pos as u64 * 8 - self.live as u64
    }

    /// Whether more bits were consumed than `data` (the reader's slice) holds
    #[inline(always)]
    pub(crate) fn overran(&self, data: &[u8]) -> bool {
        self.bit_pos() > data.len() as u64 * 8
    }
}

impl<'a> Bits<'a> {
    /// A reader at bit `bit` of `data` (bit 0 the first byte's most significant)
    pub fn at(data: &'a [u8], bit: u64) -> Self {
        let mut b = Bits {
            data,
            r: Reg {
                pos: (bit / 8) as usize,
                buf: 0,
                live: 0,
            },
        };
        let r = (bit % 8) as u32;
        if r != 0 {
            b.refill();
            b.skip(r);
        }
        b
    }

    /// The reader's slice and its state, for a loop to hold in locals (`set` gives the
    /// state back)
    #[inline(always)]
    pub(crate) fn parts(&self) -> (&'a [u8], Reg) {
        (self.data, self.r)
    }

    /// The reader on `data` (as `parts` gave it) in state `r`
    #[inline(always)]
    pub(crate) fn of(data: &'a [u8], r: Reg) -> Self {
        Bits { data, r }
    }

    #[inline(always)]
    pub(crate) fn set(&mut self, r: Reg) {
        self.r = r;
    }

    /// At least 56 bits live: the next 8 bytes loaded whole, without a test of how many
    /// bits are live (the bits below the live ones are the input's next bits already, so
    /// or-ing them in again changes nothing); a byte at a time near the slice's end
    #[inline(always)]
    pub fn refill(&mut self) {
        self.r.refill(self.data);
    }

    /// The next `n` bits (1 to 32), not consumed
    #[inline(always)]
    pub fn peek(&mut self, n: u32) -> u32 {
        self.r.peek(self.data, n)
    }

    /// The next `n` bits (1 to 32), not consumed, where at least `n` are live (after
    /// `refill`, and fewer consumed since than it leaves over `n`)
    #[inline(always)]
    pub fn peek_live(&self, n: u32) -> u32 {
        self.r.peek_live(n)
    }

    /// Consume `n` bits (below 64, and at most those live)
    #[inline(always)]
    pub fn skip_short(&mut self, n: u32) {
        self.r.skip_short(n);
    }

    /// Consume `n` bits (at most those peeked)
    #[inline(always)]
    pub fn skip(&mut self, n: u32) {
        self.r.skip(n);
    }

    #[inline(always)]
    pub fn bits(&mut self, n: u32) -> u32 {
        let v = self.peek(n);
        self.skip(n);
        v
    }

    #[inline(always)]
    pub fn bit(&mut self) -> bool {
        self.bits(1) == 1
    }

    /// Bits consumed from the slice's start
    pub fn bit_pos(&self) -> u64 {
        self.r.bit_pos()
    }

    /// Whether more bits were consumed than the slice holds (zeros read past its end)
    pub fn overran(&self) -> bool {
        self.r.overran(self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_bits_at_any_offset() {
        let data: Vec<u8> = (0..40u32).map(|i| (i * 37 + 11) as u8).collect();
        let all: Vec<bool> = data
            .iter()
            .flat_map(|&b| (0..8).rev().map(move |k| b >> k & 1 == 1))
            .collect();
        for start in 0..70u64 {
            for n in 1..=32u32 {
                let mut b = Bits::at(&data, start);
                let mut at = start as usize;
                while at + n as usize <= all.len() {
                    let want = all[at..at + n as usize]
                        .iter()
                        .fold(0u32, |v, &x| v << 1 | x as u32);
                    assert_eq!(b.bits(n), want, "start {start} n {n} at {at}");
                    at += n as usize;
                    assert_eq!(b.bit_pos(), at as u64);
                }
                assert!(!b.overran());
                b.bits(32);
                b.bits(32);
                assert!(b.overran());
            }
        }
    }

    /// The state taken out of a reader (`parts`), stepped in a local the way the symbol
    /// loop steps it (a refill, lookups on its live bits, then peeks that refill when
    /// short), reads the slice's bits, then zeros past its end and says so (`overran`);
    /// given back (`set`, `of`), the reader goes on from there
    #[test]
    fn state_in_a_local_reads_the_bits() {
        let data: Vec<u8> = (0..61u32).map(|i| (i * 89 + 7) as u8).collect();
        for len in [61, 9, 8, 7, 1, 0] {
            let d = &data[..len];
            let bit = |i: usize| i < len * 8 && d[i / 8] >> (7 - i % 8) & 1 == 1;
            let want =
                |at: usize, n: u32| (at..at + n as usize).fold(0u32, |v, i| v << 1 | bit(i) as u32);
            for start in 0..=(len * 8).min(40) {
                for n in 1..=10u32 {
                    let mut b = Bits::at(d, start as u64);
                    let (slice, mut r) = b.parts();
                    let mut at = start;
                    for k in 0..60 {
                        if k % 7 == 0 {
                            r.refill(slice);
                        }
                        let got = if k % 7 < 4 {
                            r.peek_live(n)
                        } else {
                            r.peek(slice, n)
                        };
                        assert_eq!(got, want(at, n), "len {len} start {start} n {n} k {k}");
                        r.skip_short(n);
                        at += n as usize;
                        assert_eq!(r.bit_pos(), at as u64);
                        assert_eq!(r.overran(slice), at > len * 8);
                    }
                    assert_eq!(Bits::of(slice, r).bit_pos(), at as u64);
                    b.set(r);
                    assert_eq!(b.bits(32), want(at, 32));
                    assert_eq!(b.overran(), at + 32 > len * 8);
                }
            }
        }
    }
}
