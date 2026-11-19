fn cap<T>(v: &Vec<T>) -> usize {
    v.capacity() * std::mem::size_of::<T>()
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
    let mut front = runs::Runs::default();
    let mut accepted = Vec::new();
    for (i, b) in blocks.iter().enumerate() {
        if !runs::takes_runs(&b.symbols) {
            continue;
        }
        let n = runs::front(&b.symbols, &mut front, k).unwrap();
        let caps = front.records().len();
        if n >= 65536 && caps * 20 < n * 3 && caps * 64 >= n {
            accepted.push((i, n, caps, front.byte_counts()[34]));
        }
    }
    assert_eq!(accepted.len(), 228);
    assert!(std::arch::is_x86_feature_detected!("popcnt"));
    let mut kernel = [
        worker::<writer::Candidate>(),
        worker::<counted::Candidate>(),
    ];
    let mut stage = log("observations.csv");
    writeln!(stage,"round,position,arm,block,stream,rows,output,caps,quotes,front_ns,inverse_ns,rle_ns,crc_ns,total_ns").unwrap();
    for r in 0..=turns {
        for &(i, n, caps, quotes) in &accepted {
            let b = &blocks[i];
            for j in 0..2 {
                let arm = (j + r + i) % 2;
                let (m, end) = kernel[arm](b, &oracle, k, r == 0);
                assert_eq!((m.rows, m.caps, m.fused), (n, caps, false));
                if r > 0 {
                    writeln!(
                        stage,
                        "{r},{j},{},{i},{},{n},{},{caps},{quotes},{},{},{},{},{}",
                        ["writer", "popcnt"][arm],
                        b.stream,
                        b.output.len(),
                        m.front - m.start,
                        m.inverse - m.front,
                        m.rle - m.inverse,
                        end - m.rle,
                        end - m.start
                    )
                    .unwrap();
                }
            }
        }
        stage.flush().unwrap();
    }
    drop(kernel);
    let (mut a, mut b) = (writer::Candidate::default(), counted::Candidate::default());
    let (mut x, mut y) = (Vec::new(), Vec::new());
    let mut mem = log("capacity.csv");
    writeln!(mem, "block,writer_vectors,popcnt_vectors").unwrap();
    for &(i, n, _, _) in &accepted {
        assert_eq!(n, runs::front(&blocks[i].symbols, &mut front, k).unwrap());
        let p = blocks[i].symbols.orig_ptr;
        assert!(a.invert_runs(&front, n, p, &mut x, k));
        assert!(b.invert_runs(&front, n, p, &mut y, k));
        assert!(x == y);
        writeln!(mem, "{i},{},{}", a.retained(), b.retained()).unwrap();
    }
    eprintln!("228 admissions checked for both paid paths on 1 warm + {turns} measured iterations; all marked outputs checked again while counting retained Vec capacity");
}
