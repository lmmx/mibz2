// Independent whole-stream checks with recycled capacities and a cleared worker output slot.
pub fn astra_corpus(path: &str, output: &str, evidence: &str, save_runs: bool) {
    use std::io::Write;
    let data = std::fs::read(path).unwrap();
    let mut scanner = stream::Scanner::new(stream::SliceSource(&data));
    let mut scratch = Scratch::<bwt::LazyCells>::default();
    let mut syms = Vec::new();
    let mut out = Vec::new();
    let mut file = std::io::BufWriter::new(std::fs::File::create(output).unwrap());
    let k = Kernels::detect();
    println!("ordinal,primary,run_front,rows,records,output,stock_cells,direct_size_gate");
    while let stream::Event::Block { symbols, index, .. } = scanner.next(syms).unwrap() {
        out.clear();
        let front = runs::takes_runs(&symbols);
        let before = scratch.inverse.counts().admitted;
        decode_block(&symbols,k,&mut scratch,&mut out).unwrap();
        let (n,r) = if front {
            (scratch.runs.astra_n(),scratch.runs.records().len())
        } else { (scratch.ll.len(),0) };
        let gate = front && n < (1<<20) && 128+13*(r+2)<=n+64;
        let cells = scratch.inverse.counts().admitted != before;
        println!("{},{},{},{},{},{},{},{}",index,symbols.orig_ptr,front as u8,n,r,out.len(),cells as u8,gate as u8);
        if save_runs && (index == 41 || index == 270) {
            let mut f = std::fs::File::create(format!("{evidence}/{index:03}.u16")).unwrap();
            for rec in scratch.runs.records() { f.write_all(&rec.to_le_bytes()).unwrap(); }
        }
        file.write_all(&out).unwrap();
        syms = symbols.syms;
    }
    file.flush().unwrap();
}
