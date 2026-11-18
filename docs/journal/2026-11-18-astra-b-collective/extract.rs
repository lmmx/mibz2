// Compile as a main appended to the pinned crate root, in a temporary copy.
// This grants read-only access to Runs::records(). No inverse BWT runs here.
fn main() {
    use std::{env, fs, path::Path};
    use stream::{Event, Scanner, SliceSource};

    let args: Vec<_> = env::args().collect();
    let data = fs::read(&args[1]).unwrap();
    let dir = Path::new(&args[2]);
    fs::create_dir_all(dir).unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut runs = runs::Runs::default();
    let mut syms = Vec::new();
    let mut large = 0;
    println!("block,index,n,primary,records,symbols,front_admitted,bit,end,crc");
    while let Event::Block { symbols, index, bit, end, .. } = scanner.next(syms).unwrap() {
        let n = runs::front(&symbols, &mut runs, Kernels::detect()).unwrap();
        if n >= 65536 {
            if large % 24 == 0 {
                let tape: Vec<_> = runs.records().iter().flat_map(|r| r.to_le_bytes()).collect();
                fs::write(dir.join(format!("{large:04}.recs")), tape).unwrap();
                println!("{large},{index},{n},{},{},{},{},{bit},{end},{}",
                    symbols.orig_ptr, runs.records().len(), symbols.syms.len(),
                    runs::takes_runs(&symbols), symbols.crc);
            }
            large += 1;
        }
        syms = symbols.syms;
    }
    eprintln!("{large} large blocks");
}
