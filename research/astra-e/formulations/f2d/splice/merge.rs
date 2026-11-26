// Compiled inside the F2 constructor's module. Node.out/cursor/used are
// respectively first cell, last cell, and remaining rows during this merge.
#[derive(Clone, Copy)]
struct Link {
    id: u32,
    width: u32,
    next: u32,
}

#[derive(Clone, Copy, Default, Debug)]
struct SpliceCounts {
    terminals: usize,
    splits: usize,
    whole_selectors: usize,
    partial_selectors: usize,
    partial_reads: usize,
}

#[derive(Default)]
struct Splice {
    cells: Vec<Link>,
}

impl Splice {
    fn new_cell(&mut self, value: Link) -> u32 {
        let pos = self.cells.len() as u32;
        self.cells.push(value);
        pos
    }

    fn take<const COUNT: bool>(
        &mut self,
        child: &mut Node,
        width: u32,
        stats: &mut SpliceCounts,
    ) -> (u32, u32) {
        let head = child.out;
        debug_assert!(width != 0 && width <= child.used);
        child.used -= width;
        if child.used == 0 {
            if COUNT {
                stats.whole_selectors += 1;
            }
            return (head, child.cursor);
        }
        if COUNT {
            stats.partial_selectors += 1;
        }
        let mut pos = head;
        let mut left = width;
        loop {
            let cell = self.cells[pos as usize];
            if COUNT {
                stats.partial_reads += 1;
            }
            if left > cell.width {
                left -= cell.width;
                pos = cell.next;
                continue;
            }
            if left == cell.width {
                child.out = cell.next;
            } else {
                let suffix = self.new_cell(Link {
                    width: cell.width - left,
                    ..cell
                });
                child.out = suffix;
                if child.cursor == pos {
                    child.cursor = suffix;
                }
                self.cells[pos as usize].width = left;
                if COUNT {
                    stats.splits += 1;
                }
            }
            self.cells[pos as usize].next = NONE;
            return (head, pos);
        }
    }

    fn merge<const COUNT: bool>(
        &mut self,
        nodes: &mut [Node],
        selectors: &[Piece],
        root: &mut Vec<Piece>,
    ) -> SpliceCounts {
        let mut stats = SpliceCounts::default();
        self.cells.clear();
        for i in (0..nodes.len()).rev() {
            let (mut first, mut last) = (NONE, NONE);
            for j in nodes[i].sel..nodes[i].sel + nodes[i].sels {
                let Piece { id, width } = selectors[j as usize];
                let (lo, hi) = if id == NONE {
                    let cell = self.new_cell(Link {
                        id: i as u32,
                        width,
                        next: NONE,
                    });
                    if COUNT {
                        stats.terminals += 1;
                    }
                    (cell, cell)
                } else {
                    self.take::<COUNT>(&mut nodes[id as usize], width, &mut stats)
                };
                if last == NONE {
                    first = lo;
                } else {
                    self.cells[last as usize].next = lo;
                }
                last = hi;
            }
            let node = &mut nodes[i];
            node.out = first;
            node.cursor = last;
            node.used = node.hi - node.lo;
        }
        root.clear();
        root.reserve(self.cells.len());
        let mut pos = nodes[0].out;
        while pos != NONE {
            let cell = self.cells[pos as usize];
            root.push(Piece {
                id: cell.id,
                width: cell.width,
            });
            pos = cell.next;
        }
        debug_assert_eq!(root.len(), self.cells.len());
        nodes[0].out = 0;
        stats
    }
}
