// (measurement only) the scratch the kernel keeps, in bytes, by buffer: master's fields
use super::*;
pub(super) fn b5_plain(l: &[u8], p: u32) -> Vec<u8> {
    use crate::bwt::{InverseBwt, Plain};
    let mut counts = [0u32; 256];
    for &c in l {
        counts[c as usize] += 1;
    }
    let mut out = Vec::new();
    Plain::default().invert(l, &counts, p, &mut out);
    out
}
pub(super) fn diet_parts(k: &Kernel) -> String {
    use std::mem::size_of;
    let e = &k.endpoint;
    let q: usize = k.spill.scratch();
    format!(
        "queues={} strips={} image_order={} tiles+dir={} bands={} spelling={} offsets={} edges={} extra={} inline16={} lens={} endpoint={}",
        q,
        size_of::<Strip>() * k.strips.capacity(),
        4 * k.image_order.capacity(),
        size_of::<Tile>() * k.tiles.capacity() + 4 * k.directory.capacity(),
        size_of::<Band>() * k.bands.capacity(),
        k.spelling.capacity(),
        4 * k.offsets.capacity(),
        4 * k.edges.capacity(),
        4 * k.extra.capacity(),
        16 * k.inline16.capacity(),
        k.lens.capacity(),
        e.scratch()
    )
}
pub(super) fn diet_scratch(k: &Kernel) -> usize {
    diet_parts(k)
        .split(' ')
        .map(|kv| kv.split('=').nth(1).unwrap().parse::<usize>().unwrap())
        .sum()
}
