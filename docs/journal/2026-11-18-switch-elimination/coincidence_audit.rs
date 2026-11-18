use super::*;
use crate::bwt::Plain;

fn expand(id: u32, words: &Words, out: &mut Vec<u8>) {
    if id < 256 {
        out.push(id as u8);
        return;
    }
    let w = &words.records[(id - 256) as usize];
    let start = (w.offset & !LITERAL) as usize;
    let end = start + w.count as usize;
    if w.offset & LITERAL != 0 {
        out.extend_from_slice(&words.literal[start..end]);
    } else {
        for &child in &words.children[start..end] {
            expand(child, words, out);
        }
    }
}

fn check(l: &[u8], p: usize) {
    let mut run = Coincidence::default();
    let mut count = [0; 256];
    for &c in l {
        count[c as usize] += 1;
    }
    let mut at = 0;
    let mut slots = count.map(|n| {
        let a = at;
        at += n as usize;
        a
    });
    let lf: Vec<_> = l
        .iter()
        .map(|&c| {
            let s = &mut slots[c as usize];
            let a = *s;
            *s += 1;
            a
        })
        .collect();
    // Independently construct the initial image order by sorting runs of the
    // column, then carry actual original coordinates through every later cut.
    let mut g = Exchange::default();
    let mut starts = vec![0];
    starts.extend((1..l.len()).filter(|&i| l[i] != l[i - 1]));
    starts.push(l.len());
    let mut ids: Vec<_> = (0..starts.len() - 1).collect();
    ids.sort_by_key(|&i| (l[starts[i]], i));
    g.domain.resize(ids.len(), NONE);
    for i in ids {
        g.domain[i] = g.image.len() as u32;
        g.image.push(Band {
            w: (starts[i + 1] - starts[i]) as u32,
            word: l[starts[i]] as u32,
        });
    }
    let mut live: Vec<_> = (0..l.len()).collect();
    let mut words = Words::default();
    let mut work = Work::default();
    let mut mark = p;
    let mut last_defect = usize::MAX;
    for stage in 0..=5 {
        let mut membership = vec![false; l.len()];
        for &q in &live {
            membership[q] = true;
        }
        let mut dest = Vec::new();
        let mut y = 0;
        for b in &g.image {
            dest.push(y);
            y += b.w as usize;
        }
        assert_eq!(y, live.len());
        assert_eq!(live[mark], p);
        let mut keep = vec![true; live.len()];
        let (mut x, mut j) = (0, 0);
        for &id in &g.domain {
            let b = g.image[id as usize];
            let w = b.w as usize;
            let mut spelling = Vec::new();
            expand(b.word, &words, &mut spelling);
            for k in 0..w {
                let mut q = live[x + k];
                let mut direct = Vec::new();
                loop {
                    direct.push(l[q]);
                    q = lf[q];
                    if membership[q] {
                        break;
                    }
                }
                direct.reverse();
                assert_eq!(direct, spelling);
                assert_eq!(live[dest[id as usize] + k], q);
            }
            while dest[j] + g.image[j].w as usize <= x {
                j += 1;
            }
            if dest[j] == x && g.image[j].w as usize == w && !(x..x + w).contains(&mark) {
                keep[x..x + w].fill(false);
            }
            x += w;
        }
        if stage == 5 || g.image.len() == 1 {
            break;
        }
        let mut next = Exchange::default();
        let (s, new_mark) = g.round(
            live.len() as u32,
            mark as u32,
            &mut next,
            &mut work,
            &mut words,
        );
        let d = s.bands - s.common;
        assert!(d <= last_defect && s.next_bands <= 4 * d + 1);
        assert_eq!(s.next_bands, 2 * s.bands - s.common - s.removed);
        last_defect = d;
        live = live
            .into_iter()
            .zip(keep)
            .filter_map(|(q, yes)| yes.then_some(q))
            .collect();
        assert_eq!(s.next_rows, live.len());
        mark = new_mark as usize;
        g = next;
    }
    let (mut expected, mut actual) = (Vec::new(), Vec::new());
    Plain::default().invert(l, &count, p as u32, &mut expected);
    g.emit(mark as u32, l.len(), &mut words, &mut actual);
    assert_eq!(actual, expected);
    run.invert(l, &count, p as u32, &mut actual);
    assert_eq!(actual, expected);
}

#[test]
fn every_band_in_every_round() {
    let mut total = 0;
    for (base, last) in [(2usize, 9), (3, 6)] {
        for n in 1..=last {
            for mut v in 0..base.pow(n) {
                let l: Vec<_> = (0..n)
                    .map(|_| {
                        let c = v % base;
                        v /= base;
                        c as u8
                    })
                    .collect();
                for p in 0..n {
                    check(&l, p as usize);
                    total += 1;
                }
            }
        }
    }
    println!("{total} exhaustive marked columns");
}

#[test]
fn periodic_and_crossing_wide_bands() {
    for l in [
        vec![5; 700],
        [&[1][..], &vec![0; 900]].concat(),
        (0..750).map(|x| ((11 * x) % 19) as u8).collect(),
        [vec![3; 301], vec![1; 128], vec![7; 258], vec![4; 9]].concat(),
    ] {
        for p in [0, 1, 126, 127, 128, l.len() / 2, l.len() - 1] {
            check(&l, p);
        }
    }
}
