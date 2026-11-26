fn plain(records: &[u16], p: u32) -> Vec<u8> {
    let mut l = Vec::new();
    let mut c = [0; 256];
    for &r in records {
        let b = (r >> 8) as u8;
        l.resize(l.len() + (r & 255) as usize, b);
        c[b as usize] += u32::from(r & 255);
    }
    let mut out = Vec::new();
    Plain::default().invert(&l, &c, p, &mut out);
    out
}

fn small() {
    let mut k = Kernel::default();
    let mut count = 0;
    let mut map = ResultMap::default();
    let mut out = Vec::new();
    for n in 1..=10u32 {
        for mask in 0..1 << n {
            let r: Vec<u16> = (0..n)
                .map(|i| 1 | (((mask >> i) & 1) << 8) as u16)
                .collect();
            for p in 0..n {
                let expected = plain(&r, p);
                for limit in [1, 2, 3] {
                    k.factor::<true>(&r, n, p, limit, &mut map);
                    map.emit::<true>(&k, n as usize, &mut out);
                    assert_eq!(out, expected, "n={n}, mask={mask}, p={p}");
                    count += 1;
                }
            }
        }
    }
    println!("PASS {count} binary column/mark/cutoff combinations");
}

fn real(path: &str) {
    let data = fs::read(path).unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut r = runs::Runs::default();
    let mut k = Kernel::default();
    let (mut ord, mut large) = (0, 0);
    let mut map = ResultMap::default();
    let mut text = Vec::new();
    println!("ordinal,n,admitted,records,m,thin_runs,first_bands,intersections,tile_starts,anchors,macro_bands,width_one,total_residual_edges,lookups,tail_bytes,marked_anchors,sparse_index_bytes,program_bytes,old_edges_bytes,orbit_bytes,plain_equal");
    while let Event::Block { symbols, .. } = scanner.next(syms).unwrap() {
        let n = runs::front(&symbols, &mut r, Kernels::detect()).unwrap();
        if n >= 65536 && large % 24 == 0 {
            let p = symbols.orig_ptr;
            k.factor::<true>(r.records(), n as u32, p, 16, &mut map);
            map.emit::<true>(&k, n, &mut text);
            assert_eq!(text, plain(r.records(), p));
            let w = &map.work;
            let index = 16 * map.rows.pages.len() + 4 * (map.payload.len() + map.extra.len());
            let program = std::mem::size_of::<Macro>() * map.macros.len()
                + map.tail.len()
                + 16 * map.commands.pages.len();
            let old = 4 * map.m;
            println!("{ord},{n},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{index},{program},{old},{},true",runs::takes_runs(&symbols) && r.records().len()*20<n*3,r.records().len(),map.m,w.thin_runs,k.top_bands.len(),w.splits,map.payload.len(),map.anchors,map.macros.len(),w.right_min,w.physical,w.lookups,map.tail.len(),w.emit_steps,w.orbit_area);
        }
        large += usize::from(n >= 65536);
        ord += 1;
        syms = symbols.syms;
    }
    assert_eq!(large, 233);
}

fn main() {
    let arg = env::args().nth(1).expect("small or Wikidata corpus path");
    if arg == "small" {
        small()
    } else {
        real(&arg)
    }
}
