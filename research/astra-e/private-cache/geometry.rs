fn run_geometry(records: &[u16], p: usize) -> ([usize;17],[usize;17]) {
    let mut hist = [0usize;17];
    let mut all = 0;
    let mut start = 0;
    let mut marked = (0,0);
    let mut i = 0;
    while i < records.len() {
        let byte = records[i] >> 8;
        let mut end = start;
        while i < records.len() && records[i] >> 8 == byte {
            end += (records[i] & 255) as usize;
            i += 1;
        }
        let width = end-start;
        all += 1;
        if width <= 16 {hist[width] += 1;}
        if (start..end).contains(&p) {marked = (width, usize::from(start<p)+usize::from(p+1<end));}
        start = end;
    }
    assert!(marked.0>0);
    let mut rows = [0;17];
    let mut strips = [0;17];
    let (mut short_rows,mut short_runs) = (0,0);
    for t in 0..=16 {
        short_rows += t*hist[t];
        short_runs += hist[t];
        rows[t] = short_rows + usize::from(marked.0>t);
        strips[t] = if marked.0>t {all-short_runs-1+marked.1} else {all-short_runs};
    }
    (rows,strips)
}

