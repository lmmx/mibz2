//! The parallel decoder as a command, for timing it against lbzip2 alone:
//! `cargo run --release --example bzcat -- THREADS [SLOTS] < in.bz2 > out` (BUNZIP_SIMD=off: the scalar kernels).
//! Exits 1 with the error on a bad stream, after writing every block before it.
//! BZ2_INVERSE=plain: the plain walk in place of the segmented one (bwt::InverseBwt).

use std::io::{Read, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let threads: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(4);
    let slots: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(2 * threads);
    let k = match std::env::var("BUNZIP_SIMD").as_deref() {
        Ok("0") | Ok("off") => bunzip::Kernels::scalar(),
        _ => bunzip::Kernels::detect(),
    };
    let dec = match std::env::var("BZ2_INVERSE").as_deref() {
        Ok("plain") => bunzip::StreamDecoder::with_inverse::<bunzip::bwt::Plain>(threads, slots, k),
        _ => bunzip::StreamDecoder::new(threads, slots, k),
    };
    let code = std::thread::scope(|s| {
        s.spawn(|| {
            let mut stdin = std::io::stdin().lock();
            loop {
                let mut buf = vec![0u8; 8 << 20];
                let n = stdin.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                buf.truncate(n);
                if dec.write(buf).is_err() {
                    break;
                }
            }
            dec.close();
        });
        let mut out = std::io::stdout().lock();
        let mut buf = vec![0u8; 16 << 20];
        loop {
            match dec.read(&mut buf) {
                Ok(0) => break 0,
                Ok(n) => out.write_all(&buf[..n]).unwrap(),
                Err(e) => {
                    eprintln!("bzcat: {e}");
                    dec.cancel();
                    break 1;
                }
            }
        }
    });
    std::process::exit(code);
}
