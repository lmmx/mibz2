pub use mibz2::{bwt, runs, Kernels};
use bwt::InverseBwt;
use mibz2::{block::Symbols, budget_clock, stream::{Event, Scanner, SliceSource}};
use std::io::{BufWriter, Write};
use std::ops::Range;

struct Block {
    symbols: Symbols,
    stream: u64,
    bit: u64,
    end: u64,
    output: Range<usize>,
}

fn bits(data: &[u8], offset: u64, count: u64) -> u64 {
    (offset..offset + count).fold(0, |value, i| {
        2 * value + ((data[(i / 8) as usize] >> (7 - i % 8)) & 1) as u64
    })
}

fn load(data: &[u8], oracle: &[u8], k: Kernels) -> Vec<Block> {
    let mut scanner = Scanner::new(SliceSource(data));
    let mut scratch = mibz2::Scratch::<bwt::LazyCells>::default();
    let mut out = Vec::new();
    let mut blocks = Vec::new();
    let (mut offset, mut combined, mut stream_count) = (0, 0u32, 0);
    while let Event::Block { symbols, stream, bit, end, index, .. } = scanner.next(Vec::new()).unwrap() {
        assert_eq!(index, blocks.len() as u64);
        assert_eq!(stream, stream_count);
        out.clear();
        let crc = mibz2::decode_block(&symbols, k, &mut scratch, &mut out).unwrap();
        assert_eq!(crc, symbols.crc);
        let output = offset..offset + out.len();
        assert!(out == oracle[output.clone()], "reference block {index}");
        offset = output.end;
        combined = combined.rotate_left(1) ^ crc;
        if bits(data, end, 48) == mibz2::block::END_MAGIC {
            assert_eq!(combined, bits(data, end + 48, 32) as u32);
            combined = 0;
            stream_count += 1;
        }
        blocks.push(Block { symbols, stream, bit, end, output });
    }
    assert_eq!(offset, oracle.len());
    assert_eq!(blocks.last().unwrap().stream + 1, stream_count);
    eprintln!("{} blocks, {} nonempty streams, {} oracle bytes, block and stream CRCs checked", blocks.len(), stream_count, offset);
    blocks
}

struct Worker<I: InverseBwt> {
    s: mibz2::Scratch<I>,
    out: Vec<u8>,
}

type Meter = Box<dyn FnMut(&Block, &[u8], Kernels, bool) -> (mibz2::Budget, u64)>;
fn worker<I: InverseBwt + 'static>() -> Meter {
    let mut work = Worker::<I> { s: Default::default(), out: Vec::new() };
    Box::new(move |block, oracle, k, check_normal| {
        if check_normal {
            work.out.clear();
            let crc = mibz2::decode_block(&block.symbols, k, &mut work.s, &mut work.out).unwrap();
            assert_eq!(crc, block.symbols.crc);
            assert!(work.out == oracle[block.output.clone()]);
        }
        let mut m = mibz2::Budget::default();
        work.out.clear();
        let crc = mibz2::budget_decode(&block.symbols, k, &mut work.s, &mut work.out, &mut m).unwrap();
        let end = budget_clock();
        assert_eq!(crc, block.symbols.crc);
        assert!(work.out == oracle[block.output.clone()]);
        (m, end)
    })
}

fn scan_only(data: &[u8], blocks: &[Block], turns: usize, mut log: impl Write) {
    writeln!(log,"round,block,ns").unwrap();
    for round in 0..=turns {
        let mut scanner = Scanner::new(SliceSource(data));
        let mut spare = Vec::new();
        let mut i = 0;
        loop {
            let start = budget_clock();
            let event = scanner.next(spare).unwrap();
            let end = budget_clock();
            if round > 0 { writeln!(log,"{round},{i},{}", end-start).unwrap(); }
            let Event::Block {index,stream,bit,end,symbols,..} = event else { break };
            let b = &blocks[i];
            assert_eq!((index,stream,bit,end),(i as u64,b.stream,b.bit,b.end));
            assert_eq!((symbols.crc,symbols.orig_ptr,symbols.eob,symbols.max_len),
                       (b.symbols.crc,b.symbols.orig_ptr,b.symbols.eob,b.symbols.max_len));
            assert_eq!(symbols.seq_to_unseq,b.symbols.seq_to_unseq);
            assert!(symbols.syms == b.symbols.syms);
            spare = symbols.syms;
            i += 1;
        }
        assert_eq!(i,blocks.len());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = std::fs::read(&args[1]).unwrap();
    let oracle = std::fs::read(&args[2]).unwrap();
    let turns: usize = args[3].parse().unwrap();
    let dir = std::path::Path::new(&args[4]);
    let log = |name| BufWriter::new(std::fs::File::create(dir.join(name)).unwrap());
    let k = Kernels::detect();
    let blocks = load(&input, &oracle, k);
    scan_only(&input, &blocks, turns, log("scanner.csv"));
    let mut copy = log("copy.csv");
    writeln!(copy,"round,block,ns").unwrap();
    let mut caller = vec![0u8;4<<20];
    for round in 0..=turns {
        for (index,b) in blocks.iter().enumerate() {
            let source = &oracle[b.output.clone()];
            let dest = &mut caller[..source.len()];
            let start = budget_clock();
            dest.copy_from_slice(std::hint::black_box(source));
            std::hint::black_box(&mut *dest);
            let end = budget_clock();
            assert!(dest == source);
            if round > 0 { writeln!(copy,"{round},{index},{}",end-start).unwrap(); }
        }
    }
    let mut empty = Vec::new();
    for _ in 0..10001 {
        let a = budget_clock();
        let b = budget_clock();
        empty.push(b-a);
    }
    empty.sort_unstable();
    eprintln!("adjacent thread CPU clocks p50 {} ns, p95 {} ns", empty[5000],empty[9500]);
    let mut meters: Vec<(&str,Meter)> = vec![
        ("lazy",worker::<bwt::LazyCells>()), ("opend",worker::<arms::OpenGated>()),
        ("ahead",worker::<ahead::Candidate>()), ("refill",worker::<refill::Candidate>()),
        ("bundle",worker::<bundle::Candidate>()),
    ];
    let mut stages = log("stages.csv");
    writeln!(stages,"round,arm,block,stream,rows,output,caps,fused,front_ns,inverse_ns,rle_ns,crc_ns,total_ns").unwrap();
    for round in 0..=turns {
        for i in 0..meters.len() {
            let pos = (i + round) % meters.len();
            let (arm, meter) = &mut meters[pos];
            for (index,b) in blocks.iter().enumerate() {
                let (m,end) = meter(b, &oracle, k, round == 0);
                if round > 0 {
                    writeln!(stages,"{round},{arm},{index},{},{},{},{},{},{},{},{},{},{}",b.stream,m.rows,b.output.len(),m.caps,m.fused as u8,m.front-m.start,m.inverse-m.front,m.rle-m.inverse,end-m.rle,end-m.start).unwrap();
                }
            }
        }
        stages.flush().unwrap();
        eprintln!("worker scopes round {round}/{turns} complete");
    }
}
