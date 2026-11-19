// Certify the unchanged sorted subsequence after replacing sparse RLE count positions.
use mibz2::{
    runs,
    stream::{Event, Scanner, SliceSource},
    Kernels,
};
fn cyclic_order(s: &[u8]) -> Vec<usize> {
    let n = s.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&x| s[x]);
    let mut rank = vec![0; n];
    let mut classes = 1;
    for i in 1..n {
        classes += usize::from(s[order[i]] != s[order[i - 1]]);
        rank[order[i]] = classes - 1;
    }
    let mut next = vec![0; n];
    let mut tmp = vec![0; n];
    let mut bucket = vec![0; n];
    let mut length = 1;
    while length < n && classes < n {
        bucket[..classes].fill(0);
        for &i in &order {
            let j = (i + n - length) % n;
            bucket[rank[j]] += 1;
        }
        let mut total = 0;
        for count in &mut bucket[..classes] {
            let size = *count;
            *count = total;
            total += size;
        }
        for &i in &order {
            let j = (i + n - length) % n;
            let dest = &mut bucket[rank[j]];
            tmp[*dest] = j;
            *dest += 1;
        }
        std::mem::swap(&mut order, &mut tmp);
        classes = 1;
        next[order[0]] = 0;
        for k in 1..n {
            let (a, b) = (order[k - 1], order[k]);
            classes +=
                usize::from((rank[a], rank[(a + length) % n]) != (rank[b], rank[(b + length) % n]));
            next[b] = classes - 1;
        }
        std::mem::swap(&mut rank, &mut next);
        length *= 2;
    }
    assert_eq!(
        classes, n,
        "this structural screen requires distinct decoded rotations"
    );
    rank
}
fn audit(caps: &[u16], p: usize, index: u64) {
    let l: Vec<_> = caps
        .iter()
        .flat_map(|&r| std::iter::repeat_n((r >> 8) as u8, (r & 255) as usize))
        .collect();
    let n = l.len();
    let mut positions: [Vec<usize>; 256] = std::array::from_fn(|_| Vec::new());
    for (x, &c) in l.iter().enumerate() {
        positions[c as usize].push(x);
    }
    let mut base = [0; 256];
    let mut psi = Vec::with_capacity(n);
    for (c, bucket) in positions.iter().enumerate() {
        base[c] = psi.len();
        psi.extend_from_slice(bucket);
    }
    let mut text = Vec::with_capacity(n);
    let mut row = Vec::with_capacity(n);
    let mut x = p;
    for j in 0..n {
        assert!(j == 0 || x != p);
        row.push(x);
        x = psi[x];
        text.push(l[x]);
    }
    assert_eq!(x, p);
    let mut changed = vec![false; n];
    let mut corrected = Vec::new();
    let mut mapped = vec![usize::MAX; n];
    let mut i = 0;
    let mut groups = Vec::new();
    let mut extra = 0;
    while i < n {
        if i + 4 < n && text[i..i + 4].iter().all(|&b| b == text[i]) {
            for j in i..i + 4 {
                mapped[j] = corrected.len();
                corrected.push(text[j]);
            }
            let k = usize::from(text[i + 4]);
            corrected.extend(std::iter::repeat_n(text[i], k));
            groups.push(i + 4);
            changed[i + 4] = true;
            extra += k;
            i += 5;
        } else {
            mapped[i] = corrected.len();
            corrected.push(text[i]);
            i += 1;
        }
    }
    let mut oracle = Vec::new();
    mibz2::rle::unrle(&text, &mut oracle, Kernels::scalar()).unwrap();
    assert_eq!(corrected, oracle);
    let new_rank = cyclic_order(&corrected);
    let mut vulnerable = vec![false; n];
    let (mut probes, mut max_depth) = (0, 0);
    for &end in &groups {
        let (mut j, mut lo, mut hi, mut depth) = (end, 0, n, 0);
        loop {
            j = (j + n - 1) % n;
            if changed[j] {
                break;
            }
            let c = text[j] as usize;
            lo = base[c] + positions[c].partition_point(|&r| r < lo);
            hi = base[c] + positions[c].partition_point(|&r| r < hi);
            assert!(lo <= row[j] && row[j] < hi);
            probes += 1;
            if hi - lo == 1 {
                break;
            }
            vulnerable[j] = true;
            depth += 1;
        }
        max_depth = max_depth.max(depth);
    }
    let mut by_row = vec![0; n];
    for (i, &r) in row.iter().enumerate() {
        by_row[r] = i;
    }
    let mut tails = Vec::new();
    let mut previous = None;
    for i in by_row {
        if mapped[i] == usize::MAX {
            continue;
        }
        let y = new_rank[mapped[i]];
        if !vulnerable[i] {
            assert!(previous.is_none_or(|z| z < y));
            previous = Some(y);
        }
        let k = tails.partition_point(|&z| z < y);
        if k == tails.len() {
            tails.push(y)
        } else {
            tails[k] = y;
        }
    }
    let v = vulnerable.iter().filter(|&&x| x).count();
    println!(
        "{index},{n},{},{},{},{probes},{v},{max_depth},{}",
        corrected.len(),
        groups.len(),
        extra,
        n - groups.len() - tails.len()
    );
}
fn main() {
    let data = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut r = runs::Runs::default();
    let mut ordinal = 0;
    println!("stream,original,decoded,deleted_counts,added_bytes,interval_extensions,vulnerable_literals,max_context,minimum_moved_literals");
    while let Ok(Event::Block { index, symbols, .. }) = sc.next(Vec::new()) {
        let n = runs::front(&symbols, &mut r, Kernels::detect()).unwrap();
        if n < 65536 {
            continue;
        }
        let sample = ordinal % 24 == 0;
        ordinal += 1;
        if sample && r.records().len() * 20 < 3 * n {
            audit(r.records(), symbols.orig_ptr as usize, index);
        }
    }
}
