//! Research only (`ideal-delivery`): factor D of the ideal-parts matrix.
//!
//! A reader mode, not an oracle:
//! - [`Mode::Real`]: the in-order reader as shipped, copying decoded bytes into the caller's
//!   buffer.
//! - [`Mode::Discard`]: takes each decoded block in order from the parallel decoder, recycles
//!   its buffer immediately and discards it without reading its bytes or copying to the caller,
//!   measuring the in-order reader and copy cost.
//! - [`Mode::Check`]: executes the same in-order discard loop, feeding each taken block into a
//!   streaming SHA-256 without copying to the caller, to verify byte-identical block delivery.

use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Mutex;

/// What the parallel decoder's in-order reader does with each completed block
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    /// the reader as shipped (copies bytes into the caller's buffer)
    Real = 0,
    /// takes each block in order and discards it without copying to the caller
    Discard = 1,
    /// takes each block in order, hashes it (SHA-256) and discards it without copying
    Check = 2,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Real => "real",
            Mode::Discard => "discard",
            Mode::Check => "check",
        }
    }

    pub fn parse(s: &str) -> Option<Mode> {
        [Mode::Real, Mode::Discard, Mode::Check]
            .into_iter()
            .find(|m| m.name() == s)
    }

    fn of(v: u8) -> Mode {
        match v {
            1 => Mode::Discard,
            2 => Mode::Check,
            _ => Mode::Real,
        }
    }
}

/// Counts collected across a decode by the delivery reader mode
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub discarded_blocks: u64,
    pub discarded_bytes: u64,
    pub copied_bytes: u64,
}

static MODE: AtomicU8 = AtomicU8::new(Mode::Real as u8);
static DISCARDED_BLOCKS: AtomicU64 = AtomicU64::new(0);
static DISCARDED_BYTES: AtomicU64 = AtomicU64::new(0);
static COPIED_BYTES: AtomicU64 = AtomicU64::new(0);

/// Switch the delivery reader mode for decodes started from now on
pub fn set_mode(m: Mode) {
    MODE.store(m as u8, Ordering::SeqCst);
}

/// The mode now set
pub fn mode() -> Mode {
    Mode::of(MODE.load(Ordering::Relaxed))
}

/// Delivery counts since the last call (and reset)
pub fn take_stats() -> Stats {
    Stats {
        discarded_blocks: DISCARDED_BLOCKS.swap(0, Ordering::SeqCst),
        discarded_bytes: DISCARDED_BYTES.swap(0, Ordering::SeqCst),
        copied_bytes: COPIED_BYTES.swap(0, Ordering::SeqCst),
    }
}

pub(crate) fn count_copied(n: u64) {
    COPIED_BYTES.fetch_add(n, Ordering::Relaxed);
}

pub(crate) fn record_discarded(slice: &[u8], m: Mode) {
    DISCARDED_BLOCKS.fetch_add(1, Ordering::Relaxed);
    DISCARDED_BYTES.fetch_add(slice.len() as u64, Ordering::Relaxed);
    if m == Mode::Check {
        let mut g = CHECK_HASH.lock().unwrap_or_else(|e| e.into_inner());
        g.update(slice);
    }
}

static CHECK_HASH: Mutex<Sha256> = Mutex::new(Sha256::new());

/// Reset the `Mode::Check` streaming SHA-256 before a check decode
pub fn clear_check() {
    *CHECK_HASH.lock().unwrap_or_else(|e| e.into_inner()) = Sha256::new();
}

/// Finalise and reset the `Mode::Check` streaming SHA-256 after a check decode
pub fn take_check_sha256() -> [u8; 32] {
    let mut g = CHECK_HASH.lock().unwrap_or_else(|e| e.into_inner());
    let h = std::mem::replace(&mut *g, Sha256::new());
    h.finish()
}

/// Minimal std-only SHA-256 (FIPS 180-4) for `Mode::Check` byte-identity verification
struct Sha256 {
    state: [u32; 8],
    buf: [u8; 64],
    buf_len: usize,
    total_bytes: u64,
}

impl Sha256 {
    const H0: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];

    const K: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];

    const fn new() -> Self {
        Self {
            state: Self::H0,
            buf: [0; 64],
            buf_len: 0,
            total_bytes: 0,
        }
    }

    fn compress(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[4 * i],
                block[4 * i + 1],
                block[4 * i + 2],
                block[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for (&ki, &wi) in Self::K.iter().zip(&w) {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(ki)
                .wrapping_add(wi);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, delta) in self.state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(delta);
        }
    }

    fn update(&mut self, mut data: &[u8]) {
        self.total_bytes += data.len() as u64;
        if self.buf_len > 0 {
            let take = (64 - self.buf_len).min(data.len());
            self.buf[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            data = &data[take..];
            if self.buf_len == 64 {
                let block = self.buf;
                self.compress(&block);
                self.buf_len = 0;
            }
        }
        while data.len() >= 64 {
            let block: &[u8; 64] = data[..64].try_into().unwrap();
            self.compress(block);
            data = &data[64..];
        }
        if !data.is_empty() {
            self.buf[..data.len()].copy_from_slice(data);
            self.buf_len = data.len();
        }
    }

    fn finish(mut self) -> [u8; 32] {
        let bits = self.total_bytes.wrapping_mul(8);
        self.buf[self.buf_len] = 0x80;
        self.buf_len += 1;
        if self.buf_len > 56 {
            self.buf[self.buf_len..].fill(0);
            let block = self.buf;
            self.compress(&block);
            self.buf_len = 0;
        }
        self.buf[self.buf_len..56].fill(0);
        self.buf[56..64].copy_from_slice(&bits.to_be_bytes());
        let block = self.buf;
        self.compress(&block);
        let mut out = [0u8; 32];
        for (i, &word) in self.state.iter().enumerate() {
            out[4 * i..4 * i + 4].copy_from_slice(&word.to_be_bytes());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(d: [u8; 32]) -> String {
        d.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn mode_names_and_roundtrip() {
        for m in [Mode::Real, Mode::Discard, Mode::Check] {
            assert_eq!(Mode::parse(m.name()), Some(m));
            assert_eq!(Mode::of(m as u8), m);
        }
        assert_eq!(Mode::parse("other"), None);
    }

    #[test]
    fn sha256_nist_vectors() {
        for (input, want) in [
            (
                b"".as_slice(),
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                b"abc".as_slice(),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
            (
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".as_slice(),
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            ),
        ] {
            let mut h = Sha256::new();
            h.update(input);
            assert_eq!(hex(h.finish()), want);
        }
    }
}
