//! Real dump blocks: byte-range windows of a .bz2 (e.g. `curl -r` from dumps.wikimedia.org)
//! checked against libbzip2 and lbzip2. In each window, every whole block is found (the
//! first block magic at any bit, then block after block, across stream ends and headers)
//! and decoded with its CRC checked; the whole blocks are then written as a stream of their
//! own (a header, their bits, an end marker, their combined CRC), which libbzip2, lbzip2 and
//! this decoder (sequential on both kernel sets, and parallel) must each decode to exactly
//! the blocks' bytes; and `decode_blocks` on the window's own bits must give them too.
//!
//! Usage: windows OUT_DIR WINDOW... (each window's stream is left in OUT_DIR as
//! NAME.blocks.bz2, for timing the decoders on real blocks)

#[path = "../tests/common/mod.rs"]
mod common;

use bunzip::bits::Bits;
use bunzip::block::{read_item, Item, BLOCK_MAGIC};
use bunzip::{decode_block, Kernels, Scratch};
use common::*;
use std::path::Path;
use std::process::{Command, Stdio};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out_dir = Path::new(&args[1]);
    std::fs::create_dir_all(out_dir).unwrap();
    let (mut total_blocks, mut total_bytes, mut failed) = (0usize, 0usize, 0usize);
    for path in &args[2..] {
        let data = std::fs::read(path).unwrap();
        let name = Path::new(path).file_stem().unwrap().to_string_lossy().to_string();
        // the first block magic, at any bit
        let first = (0..data.len() as u64 * 8 - 48).find(|&b| {
            let mut br = Bits::at(&data, b);
            (br.bits(24) as u64) << 24 | br.bits(24) as u64 == BLOCK_MAGIC
        });
        let Some(mut bit) = first else {
            println!("{name}: no block");
            continue;
        };
        let mut level = 9; // (each stream header's, once one is met)
        let mut blocks: Vec<(u64, u64, u32)> = Vec::new();
        let mut want = Vec::new();
        let mut scratch: Scratch = Scratch::default();
        let mut syms = Vec::new();
        let mut streams = 0;
        loop {
            let mut br = Bits::at(&data, bit);
            let item = read_item(&mut br, level, syms);
            if br.overran() {
                break; // the window ends inside it
            }
            let end = br.bit_pos();
            match item {
                Ok(Item::Block(b)) => {
                    let crc = decode_block(&b, Kernels::detect(), &mut scratch, &mut want)
                        .unwrap_or_else(|k| panic!("{name}: block at bit {bit}: {k}"));
                    blocks.push((bit, end, crc));
                    syms = b.syms;
                    bit = end;
                }
                Ok(Item::End(_)) => {
                    // the next stream's header, at the next byte
                    streams += 1;
                    let at = end.div_ceil(8) as usize;
                    match data.get(at..at + 4) {
                        Some([b'B', b'Z', b'h', l @ b'1'..=b'9']) => {
                            level = (l - b'0') as u32;
                            bit = (at as u64 + 4) * 8;
                            syms = Vec::new();
                        }
                        None => break,
                        Some(_) => {
                            println!("{name}: {} bytes after a stream's end, not a stream", data.len() - at);
                            break;
                        }
                    }
                }
                Err((k, _)) => panic!("{name}: at bit {bit}: {k}"),
            }
        }
        // the whole blocks as a stream of their own
        let mut w = BitWriter::default();
        for &c in b"BZh9" {
            w.put(c as u64, 8);
        }
        for &(s, e, _) in &blocks {
            let mut br = Bits::at(&data, s);
            let mut left = e - s;
            while left > 0 {
                let n = left.min(32) as u32;
                w.put(br.bits(n) as u64, n);
                left -= n as u64;
            }
        }
        w.put(0x1772_4538_5090, 48);
        w.put(bunzip::combine(blocks.iter().map(|b| b.2)) as u64, 32);
        let stream = w.finish();
        let file = out_dir.join(format!("{name}.blocks.bz2"));
        std::fs::write(&file, &stream).unwrap();
        let mut checks = Vec::new();
        checks.push(("libbzip2", oracle(&stream).map_err(|e| e.to_string())));
        let lb = Command::new("lbzip2").args(["-dc", "-n4"]).arg(&file).stderr(Stdio::inherit()).output().unwrap();
        checks.push(("lbzip2", if lb.status.success() { Ok(lb.stdout) } else { Err("lbzip2 failed".into()) }));
        for (n, k) in [("scalar", Kernels::scalar()), ("simd", Kernels::detect())] {
            checks.push((n, bunzip::decode_with::<bunzip::bwt::Segmented>(&stream, k).map_err(|e| e.to_string())));
        }
        checks.push(("parallel", parallel(&stream, 4, 8, 8 << 20, 16 << 20, Kernels::detect()).map_err(|e| e.to_string())));
        let mut ranged = Vec::new();
        let r = bunzip::decode_blocks(&data, blocks[0].0, blocks.last().unwrap().1, level, Kernels::detect(), &mut ranged);
        checks.push(("decode_blocks", r.map(|_| ranged).map_err(|e| e.to_string())));
        let bad: Vec<&str> = checks.iter().filter(|(_, r)| r.as_ref() != Ok(&want)).map(|(n, _)| *n).collect();
        failed += !bad.is_empty() as usize;
        total_blocks += blocks.len();
        total_bytes += want.len();
        println!(
            "{name}: {} whole blocks (bits {}..{}), {} streams ended, {} bytes decoded, every block CRC checked: {}",
            blocks.len(),
            blocks[0].0,
            blocks.last().unwrap().1,
            streams,
            want.len(),
            if bad.is_empty() { "identical in libbzip2, lbzip2, scalar, simd, parallel, decode_blocks".to_string() } else { format!("DIFFERENT in {bad:?}") }
        );
    }
    println!("{total_blocks} blocks, {total_bytes} bytes, {failed} windows failed");
    std::process::exit(if failed == 0 { 0 } else { 1 });
}
