//! An MSB-first bit reader over a byte slice. Past the slice's end it reads zeros, and
//! says it has (`overran`): the caller then either has more input to give it (and starts
//! the item again) or the stream is truncated. So the decoder's loops never index past
//! the input, and an item cut short by the input's end is never taken as whole.

pub struct Bits<'a> {
    data: &'a [u8],
    /// the next byte to load
    pos: usize,
    /// bits, MSB first: the top `live` are valid (the bits below them are the input's
    /// next bits or zeros, never anything else)
    buf: u64,
    live: u32,
}

impl<'a> Bits<'a> {
    /// A reader at bit `bit` of `data` (bit 0 the first byte's most significant)
    pub fn at(data: &'a [u8], bit: u64) -> Self {
        let mut b = Bits {
            data,
            pos: (bit / 8) as usize,
            buf: 0,
            live: 0,
        };
        let r = (bit % 8) as u32;
        if r != 0 {
            b.refill();
            b.skip(r);
        }
        b
    }

    /// At least 56 bits live: the next 8 bytes loaded whole, without a test of how many
    /// bits are live (the bits below the live ones are the input's next bits already, so
    /// or-ing them in again changes nothing); a byte at a time near the slice's end
    #[inline(always)]
    pub fn refill(&mut self) {
        if let Some(w) = self.data.get(self.pos..self.pos + 8) {
            let v = u64::from_be_bytes(w.try_into().unwrap());
            self.buf |= v >> self.live;
            let take = (63 - self.live) >> 3;
            self.pos += take as usize;
            self.live += take * 8;
        } else {
            self.refill_slow();
        }
    }

    #[cold]
    fn refill_slow(&mut self) {
        while self.live <= 56 {
            let b = self.data.get(self.pos).copied().unwrap_or(0);
            self.pos += 1;
            self.buf |= (b as u64) << (56 - self.live);
            self.live += 8;
        }
    }

    /// The next `n` bits (1 to 32), not consumed
    #[inline(always)]
    pub fn peek(&mut self, n: u32) -> u32 {
        debug_assert!((1..=32).contains(&n));
        if self.live < n {
            self.refill();
        }
        (self.buf >> (64 - n)) as u32
    }

    /// The next `n` bits (1 to 32), not consumed, where at least `n` are live (after
    /// `refill`, and fewer consumed since than it leaves over `n`)
    #[inline(always)]
    pub fn peek_live(&self, n: u32) -> u32 {
        debug_assert!((1..=32).contains(&n) && n <= self.live);
        (self.buf >> (64 - n)) as u32
    }

    /// Consume `n` bits (below 64, and at most those live)
    #[inline(always)]
    pub fn skip_short(&mut self, n: u32) {
        debug_assert!(n < 64 && n <= self.live);
        self.buf <<= n & 63;
        self.live -= n;
    }

    /// Consume `n` bits (at most those peeked)
    #[inline(always)]
    pub fn skip(&mut self, n: u32) {
        debug_assert!(n <= self.live);
        self.buf = if n >= 64 { 0 } else { self.buf << n };
        self.live -= n;
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
        self.pos as u64 * 8 - self.live as u64
    }

    /// Whether more bits were consumed than the slice holds (zeros read past its end)
    pub fn overran(&self) -> bool {
        self.bit_pos() > self.data.len() as u64 * 8
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
}
