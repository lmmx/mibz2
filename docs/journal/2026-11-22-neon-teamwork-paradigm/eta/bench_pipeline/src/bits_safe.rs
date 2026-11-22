//! Safe MSB-first bit reader mirroring `mibz2::bits::Bits` / `Reg`.

#[derive(Clone, Copy)]
pub struct Reg {
    pos: usize,
    buf: u64,
    live: u32,
}

impl Reg {
    #[inline(always)]
    pub fn at(data: &[u8], bit: u64) -> Self {
        let mut r = Reg {
            pos: (bit / 8) as usize,
            buf: 0,
            live: 0,
        };
        let rem = (bit % 8) as u32;
        if rem != 0 {
            r.refill(data);
            r.skip(rem);
        }
        r
    }

    #[inline(always)]
    pub fn refill(&mut self, data: &[u8]) {
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

    #[inline(always)]
    pub fn peek(&mut self, data: &[u8], n: u32) -> u32 {
        if self.live < n {
            self.refill(data);
        }
        (self.buf >> (64 - n)) as u32
    }

    #[inline(always)]
    pub fn peek_live(&self, n: u32) -> u32 {
        (self.buf >> (64 - n)) as u32
    }

    #[inline(always)]
    pub fn skip_short(&mut self, n: u32) {
        self.buf <<= n & 63;
        self.live -= n;
    }

    #[inline(always)]
    pub fn skip(&mut self, n: u32) {
        self.buf = if n >= 64 { 0 } else { self.buf << n };
        self.live -= n;
    }

    #[inline(always)]
    pub fn bits(&mut self, data: &[u8], n: u32) -> u32 {
        let v = self.peek(data, n);
        self.skip(n);
        v
    }

    #[inline(always)]
    pub fn bit(&mut self, data: &[u8]) -> bool {
        self.bits(data, 1) == 1
    }

    #[inline(always)]
    pub fn bit_pos(&self) -> u64 {
        self.pos as u64 * 8 - self.live as u64
    }

    #[inline(always)]
    pub fn overran(&self, data: &[u8]) -> bool {
        self.bit_pos() > data.len() as u64 * 8
    }

    #[inline(always)]
    pub fn live(&self) -> u32 {
        self.live
    }
}
