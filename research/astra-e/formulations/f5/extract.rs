use mibz2::{
    runs,
    stream::{Event, Scanner, SliceSource},
    Kernels,
};
use std::{
    env, fs,
    io::{BufWriter, Write},
    path::Path,
};
fn main() {
    let args: Vec<_> = env::args().collect();
    let raw = fs::read(&args[1]).unwrap();
    let dir = Path::new(&args[2]);
    let mut scanner = Scanner::new(SliceSource(&raw));
    let mut scratch = Vec::new();
    let mut r = runs::Runs::default();
    let mut kernel = Kernel::default();
    let (mut ord, mut large) = (0, 0);
    println!("ordinal,n,primary,admitted,bit,end,level,crc,words,visits,period_bytes");
    while let Event::Block {
        symbols,
        bit,
        end,
        level,
        ..
    } = scanner.next(scratch).unwrap()
    {
        let n = runs::front(&symbols, &mut r, Kernels::detect()).unwrap();
        if n >= 65536 && large % 24 == 0 {
            let (_, mark, bits) =
                kernel.construct::<true, true>(r.records(), n as u32, symbols.orig_ptr, 16);
            let mask = (1 << bits) - 1;
            let mut row = mark;
            let mut codes = Vec::new();
            let mut size = 0;
            loop {
                let edge = kernel.edges[row as usize];
                let code = if kernel.extra.is_empty() {
                    edge >> bits
                } else {
                    kernel.extra[row as usize]
                };
                size += if code < 256 {
                    1
                } else {
                    let i = (code - 256) as usize;
                    kernel.offsets[i + 1] - kernel.offsets[i]
                };
                codes.push(code);
                row = edge & mask;
                if row == mark {
                    break;
                }
            }
            let mut tape =
                BufWriter::new(fs::File::create(dir.join(format!("{ord}.words"))).unwrap());
            let words = kernel.offsets.len() - 1;
            tape.write_all(&(words as u32).to_le_bytes()).unwrap();
            for o in kernel.offsets.windows(2) {
                tape.write_all(&(o[1] - o[0]).to_le_bytes()).unwrap();
                tape.write_all(&kernel.spelling[o[0] as usize..o[1] as usize])
                    .unwrap();
            }
            for &c in &codes {
                tape.write_all(&c.to_le_bytes()).unwrap();
            }
            println!(
                "{ord},{n},{},{},{bit},{end},{level},{},{words},{},{size}",
                symbols.orig_ptr,
                runs::takes_runs(&symbols) && r.records().len() * 20 < n * 3,
                symbols.crc,
                codes.len()
            );
        }
        large += usize::from(n >= 65536);
        ord += 1;
        scratch = symbols.syms;
    }
    assert_eq!(large, 233);
}
