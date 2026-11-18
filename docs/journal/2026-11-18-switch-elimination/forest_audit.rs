use super::*;
use crate::bwt::Plain;

fn check(l: &[u8], p: usize) {
    let mut h = [0; 256];
    let mut kernel = Forest::default();
    for &c in l {
        h[c as usize] += 1;
        kernel.add(c, 1);
    }
    let mut sum = 0;
    let mut pos = h.map(|n| {
        let y = sum;
        sum += n as usize;
        y
    });
    let lf: Vec<_> = l
        .iter()
        .map(|&v| {
            let x = &mut pos[v as usize];
            let old = *x;
            *x += 1;
            old
        })
        .collect();
    let mut g = Graph::initial(&kernel.runs, p as u32);
    let mut words = Recipes::default();
    let mut output = Vec::new();
    loop {
        let mut keep = vec![false; l.len()];
        let mut mass = 0;
        for a in &g.arcs {
            let x = (g.base[a.u] + a.x) as usize;
            for row in &mut keep[x..x + a.w as usize] {
                assert!(!*row);
                *row = true;
            }
            mass += a.w as usize * words.len(a.word) as usize;
        }
        assert_eq!(mass, l.len());
        assert_eq!(g.base[g.mark.0] + g.mark.1, p as u32);
        for a in &g.arcs {
            // Force fresh execution of this recipe. The independent reference below
            // checks every ordinate, including returns on cycles outside the mark.
            for r in &mut words.rules {
                r.first = NONE;
            }
            output.clear();
            words.write(a.word, usize::MAX, &mut output);
            for k in 0..a.w {
                let mut row = (g.base[a.u] + a.x + k) as usize;
                let mut path = Vec::new();
                loop {
                    path.push(l[row]);
                    row = lf[row];
                    if keep[row] {
                        break;
                    }
                }
                path.reverse();
                assert_eq!(path, output);
                assert_eq!(row as u32, g.base[a.v] + a.y + k);
            }
        }
        let mut s = Layer::default();
        if let Some(next) = g.reduce(&mut words, usize::MAX, &mut s) {
            assert!(next.rows < g.rows);
            g = next;
        } else {
            break;
        }
    }
    let mut reference = Vec::new();
    Plain::default().invert(l, &h, p as u32, &mut reference);
    kernel.invert(l, &h, p as u32, &mut output);
    assert_eq!(reference, output);
}

#[test]
fn all_boundary_pairings() {
    let mut cases = 0;
    for (b, nmax) in [(2usize, 9), (3, 6)] {
        for n in 1..=nmax {
            for mut v in 0..b.pow(n) {
                let l: Vec<_> = (0..n)
                    .map(|_| {
                        let a = v % b;
                        v /= b;
                        a as u8
                    })
                    .collect();
                for p in 0..n {
                    check(&l, p as usize);
                    cases += 1;
                }
            }
        }
    }
    println!("{cases} exhaustive marked columns");
}

#[test]
fn wide_shifted_and_disconnected() {
    for l in [
        vec![4; 512],
        [vec![2; 150], vec![0; 150], vec![1; 150]].concat(),
        [vec![1; 1], vec![0; 510]].concat(),
        (0..185).map(|i| ((i * 7) % 17) as u8).collect(),
    ] {
        for p in [0, 1, 12, 127, l.len() / 2, l.len() - 1] {
            check(&l, p);
        }
    }
}
