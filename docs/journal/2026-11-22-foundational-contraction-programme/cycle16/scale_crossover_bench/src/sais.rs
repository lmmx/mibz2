#![forbid(unsafe_code)]
//! Pure-Rust O(n) SA-IS (Nong-Zhang-Chan 2009) for fast exact circular BWT construction
//! on multi-megabyte repetitive corpora (900 KiB -> 64 MiB).

fn is_lms(t: &[bool], i: usize) -> bool {
    i > 0 && t[i] && !t[i - 1]
}

fn induce_sort(s: &[u32], sa: &mut [i32], t: &[bool], k: usize, bkt_sizes: &[usize]) {
    let n = s.len();
    let mut bkt = vec![0usize; k];
    let mut sum = 0usize;
    for c in 0..k {
        bkt[c] = sum;
        sum += bkt_sizes[c];
    }
    for i in 0..n {
        let j = sa[i];
        if j > 0 {
            let jm1 = (j - 1) as usize;
            if !t[jm1] {
                let c = s[jm1] as usize;
                sa[bkt[c]] = jm1 as i32;
                bkt[c] += 1;
            }
        }
    }
    let mut sum = 0usize;
    for c in 0..k {
        sum += bkt_sizes[c];
        bkt[c] = sum;
    }
    for i in (0..n).rev() {
        let j = sa[i];
        if j > 0 {
            let jm1 = (j - 1) as usize;
            if t[jm1] {
                let c = s[jm1] as usize;
                bkt[c] -= 1;
                sa[bkt[c]] = jm1 as i32;
            }
        }
    }
}

fn sais_rec(s: &[u32], k: usize) -> Vec<i32> {
    let n = s.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![0];
    }
    if n == 2 {
        return if s[0] < s[1] { vec![0, 1] } else { vec![1, 0] };
    }

    let mut t = vec![false; n];
    t[n - 1] = true;
    for i in (0..n - 1).rev() {
        t[i] = if s[i] < s[i + 1] {
            true
        } else if s[i] > s[i + 1] {
            false
        } else {
            t[i + 1]
        };
    }

    let mut bkt_sizes = vec![0usize; k];
    for &c in s {
        bkt_sizes[c as usize] += 1;
    }

    let mut sa = vec![-1i32; n];
    let mut bkt = vec![0usize; k];
    let mut sum = 0usize;
    for c in 0..k {
        sum += bkt_sizes[c];
        bkt[c] = sum;
    }
    for i in 1..n {
        if is_lms(&t, i) {
            let c = s[i] as usize;
            bkt[c] -= 1;
            sa[bkt[c]] = i as i32;
        }
    }
    induce_sort(s, &mut sa, &t, k, &bkt_sizes);

    let mut n1 = 0usize;
    for i in 0..n {
        let pos = sa[i] as usize;
        if is_lms(&t, pos) {
            sa[n1] = pos as i32;
            n1 += 1;
        }
    }

    for x in &mut sa[n1..] {
        *x = -1;
    }
    let mut name = 0i32;
    let mut prev: i32 = -1;
    for i in 0..n1 {
        let pos = sa[i] as usize;
        let mut diff = false;
        if prev < 0 {
            diff = true;
        } else {
            let ppos = prev as usize;
            let mut d = 0usize;
            loop {
                if s[pos + d] != s[ppos + d] || t[pos + d] != t[ppos + d] {
                    diff = true;
                    break;
                }
                if d > 0 && (is_lms(&t, pos + d) || is_lms(&t, ppos + d)) {
                    break;
                }
                d += 1;
            }
        }
        if diff {
            name += 1;
            prev = pos as i32;
        }
        sa[n1 + (pos >> 1)] = name - 1;
    }

    let mut s1 = Vec::with_capacity(n1);
    let mut lms_pos = Vec::with_capacity(n1);
    for i in 1..n {
        if is_lms(&t, i) {
            lms_pos.push(i as i32);
            s1.push(sa[n1 + (i >> 1)] as u32);
        }
    }
    debug_assert_eq!(s1.len(), n1);

    let sa1 = if (name as usize) < n1 {
        sais_rec(&s1, name as usize)
    } else {
        let mut direct = vec![0i32; n1];
        for (i, &nm) in s1.iter().enumerate() {
            direct[nm as usize] = i as i32;
        }
        direct
    };

    sa.fill(-1);
    let mut sum = 0usize;
    for c in 0..k {
        sum += bkt_sizes[c];
        bkt[c] = sum;
    }
    for i in (0..n1).rev() {
        let pos = lms_pos[sa1[i] as usize] as usize;
        let c = s[pos] as usize;
        bkt[c] -= 1;
        sa[bkt[c]] = pos as i32;
    }
    induce_sort(s, &mut sa, &t, k, &bkt_sizes);
    sa
}

pub fn circular_bwt(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    assert!(n > 0 && !text.contains(&0));
    let mut doubled = Vec::with_capacity(2 * n + 1);
    for &b in text {
        doubled.push(u32::from(b));
    }
    for &b in text {
        doubled.push(u32::from(b));
    }
    doubled.push(0);
    let sa = sais_rec(&doubled, 256);
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for &pos in &sa {
        let pu = pos as usize;
        if pu < n {
            if pu == 0 {
                p = l.len() as u32;
                l.push(text[n - 1]);
            } else {
                l.push(text[pu - 1]);
            }
        }
    }
    assert_eq!(l.len(), n);
    (l, p)
}

pub fn to_r127_records(l: &[u8]) -> (Vec<u16>, usize) {
    let mut recs = Vec::new();
    let mut r0 = 0usize;
    let mut i = 0usize;
    let n = l.len();
    while i < n {
        let b = l[i];
        let mut j = i + 1;
        while j < n && l[j] == b {
            j += 1;
        }
        r0 += 1;
        let mut rem = j - i;
        while rem > 0 {
            let take = rem.min(127);
            recs.push((u16::from(b) << 8) | (take as u16));
            rem -= take;
        }
        i = j;
    }
    (recs, r0)
}
