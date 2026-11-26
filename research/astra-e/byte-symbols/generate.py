"""Build the byte path in an isolated checkout of the pinned sources."""
from pathlib import Path
import re
import sys

root = Path(sys.argv[1]).resolve()
assert root != Path(__file__).resolve().parents[3]
src = root / "src"
def once(s, a, b):
    assert s.count(a) == 1, a
    return s.replace(a,b)
def item(s, name):
    match = re.search(r"^(?:pub |pub\(crate\) )?(?:unsafe )?fn "+name+r"\b.*?^}", s, re.M | re.S)
    assert match, name
    return match[0]
def add_args(fn):
    return fn.replace("b: &Symbols,", "b: &Symbols, bytes: &[u8],")
def rename(s,names):
    return re.sub(r"\b("+"|".join(names)+r")\b",lambda m:m[0]+"_bytes", s)

p = src/"block.rs"
s = p.read_text()
sig = "pub fn read_item(\n    br: &mut Bits,\n    level: u32,\n    mut syms: Vec<u16>,\n) -> Result<Item, (ErrorKind, Vec<u16>)> {"
s = once(s,sig,sig.replace("mut syms", "syms") + """
    read_item_width::<false>(br, level, syms)
}

pub(crate) fn read_item_width<const PACK: bool>(
    br: &mut Bits, level: u32, mut syms: Vec<u16>
) -> Result<Item, (ErrorKind, Vec<u16>)> {""")
start = s.index("    syms.clear();")
end = s.index("    Ok(Item::Block",start)
loop = s[s.index("    let mut n = 0;",start):s.index("    if let Err(k) = res",start)]
loop = loop.replace('as u16);','as $word);').replace('.write(s);','.write(s as $word);')
loop = loop.replace('out[','$output[')
loop += '    res.map(|_| n)'
# Defining a macro inside another macro takes an explicit dollar token.
loop = loop.replace('$live','$d live')
s = s[:start] + """
    syms.clear();
    macro_rules! read_symbols {
        ($output:ident, $word:ty, $d:tt) => {{
""" + loop + """
        }};
    }
    if PACK && eob <= 255 {
        syms.reserve(1 + MAX_SYMS.div_ceil(2));
        // The Vec keeps its original u16 owner and alignment for pool reuse.
        // The first word tags the encoded tape and its parity. Every exposed
        // byte is initialized before set_len, including the odd trailing pad.
        let out = unsafe { std::slice::from_raw_parts_mut(
            syms.as_mut_ptr().add(1).cast::<std::mem::MaybeUninit<u8>>(), MAX_SYMS) };
        let n = match read_symbols!(out, u8, $) { Ok(n)=>n, Err(k)=>fail!(k) };
        if n & 1 != 0 { out[n].write(0); }
        unsafe {
            syms.as_mut_ptr().write(0xfffe | (n & 1) as u16);
            syms.set_len(1 + n.div_ceil(2));
        }
    } else {
        syms.reserve(MAX_SYMS);
        let out = &mut syms.spare_capacity_mut()[..MAX_SYMS];
        let n = match read_symbols!(out, u16, $) { Ok(n)=>n, Err(k)=>fail!(k) };
        unsafe { syms.set_len(n); }
    }
""" + s[end:]
# The decoded length excludes both the metadata word and the initialized pad.
s += """
impl Symbols {
    pub fn byte_tape(&self) -> Option<&[u8]> {
        let (&tag, words) = self.syms.split_first()?;
        if tag < 0xfffe || self.eob > 255 { return None; }
        let len = (2 * words.len()).checked_sub(usize::from(tag & 1))?;
        Some(unsafe { std::slice::from_raw_parts(words.as_ptr().cast::<u8>(), len) })
    }
}
"""
p.write_text(s)

p = src/"stream.rs"
s = p.read_text()
sig = "pub fn next(&mut self, mut syms: Vec<u16>) -> Result<Event, Error> {"
s = once(s, sig, sig.replace("mut syms:", "syms:") + """
        self.next_width::<false>(syms)
    }

    pub fn next_width<const PACK: bool>(&mut self, mut syms: Vec<u16>) -> Result<Event, Error> {""")
p.write_text(once(s, "block::read_item(&mut br, level, syms)", "block::read_item_width::<PACK>(&mut br, level, syms)"))

p = src/"parallel.rs"
s = p.read_text()
s = once(s,"fn launch<W>(threads", "fn launch<const PACK: bool, W>(threads")
s = once(s,"Self::launch(threads","Self::launch::<true,_>(threads")
s = s.replace("Self::launch(teams", "Self::launch::<false,_>(teams")
s = once(s,"fn scanner(shared", "fn scanner<const PACK: bool>(shared")
s = once(s,"spawn(move || scanner(", "spawn(move || scanner::<PACK>(")
s = once(s,"Box::new(move || scanner(", "Box::new(move || scanner::<false>(")
s = once(s,"sc.next(syms)", "sc.next_width::<PACK>(syms)")
p.write_text(once(s,"decode_block(&symbols, k, &mut scratch, &mut out)",
                   "crate::decode_byte_block(&symbols, k, &mut scratch, &mut out)"))

p = src/"runs.rs"
s = p.read_text()
names = ["takes_runs","front","front_scalar","front_avx2"]
extra = []
for name in names:
    fn = item(s,name)
    fn = add_args(rename(fn,names)).replace("b: &Symbols)", "b: &Symbols, bytes: &[u8])")
    fn = fn.replace("let syms = &b.syms;", "let syms = bytes;").replace("w: &[u16]", "w: &[u8]")
    fn = fn.replace("front_avx2_bytes(b, r)", "front_avx2_bytes(b, bytes, r)").replace("front_scalar_bytes(b, r)", "front_scalar_bytes(b, bytes, r)")
    fn = fn.replace("for &s in &b.syms {", "for &raw in bytes { let s = u16::from(raw);")
    if name=="front_avx2":
        fn = fn.replace("b.syms.", "bytes.").replace("b.syms[", "bytes[")
        start=fn.index("    let v1 ="); end=fn.index("    let mut end",start)
        fn = fn[:start]+"""    let v1 = _mm256_set1_epi8(1);
    let v0 = _mm256_setzero_si256();
    let ve = _mm256_set1_epi8(b.eob as i8);
"""+fn[end:]
        start=fn.index("        let y ="); end=fn.index("        let (mut l",start)
        fn=fn[:start]+"""        let run = _mm256_movemask_epi8(_mm256_cmpeq_epi8(_mm256_min_epu8(x,v1),x)) as u32;
        let runa = _mm256_movemask_epi8(_mm256_cmpeq_epi8(x,v0)) as u32;
        let eob = _mm256_movemask_epi8(_mm256_cmpeq_epi8(x,ve)) as u32 & !run;
"""+fn[end:]
        fn=fn.replace('if s == b.eob {','if u16::from(s) == b.eob {')
        fn='#[cfg(target_arch="x86_64")]\n#[target_feature(enable="avx2,bmi1,bmi2,lzcnt,popcnt")]\n'+fn
    extra.append(fn)
p.write_text(s+"\n"+"\n\n".join(extra)+"\n")

p = src/"mtf.rs"
s = p.read_text()
names = ["mtf","mtf_scalar","mtf_avx2","mtf_checkpoints","mtf_checkpoints_scalar","mtf_checkpoints_avx2"]
extra = []
for name in names:
    fn = add_args(rename(item(s,name),names))
    for other in names:
        fn = fn.replace(other+"_bytes(b, ll",other+"_bytes(b, bytes, ll")
    fn = fn.replace("for &s in &b.syms {", "for &raw in bytes { let s = u16::from(raw);")
    fn = fn.replace("let mut syms = b.syms.iter();","let mut syms = bytes.iter().copied().map(u16::from);")
    fn = fn.replace("for &s in syms.by_ref()", "for s in syms.by_ref()")
    if name.endswith("_avx2"):
        fn='#[cfg(target_arch="x86_64")]\n#[target_feature(enable="avx2")]\n'+fn
    extra.append(fn)
p.write_text(s+"\n"+"\n\n".join(extra)+"\n")

p = src/"lib.rs"
s = p.read_text()
fn = item(s,"decode_block").replace("pub fn decode_block", "pub fn decode_byte_block")
key = "    let cooperative ="
fn=once(fn,key,"    let Some(bytes) = b.byte_tape() else { return decode_block(b,k,s,out); };\n"+key)
# The frozen inverse (and stock Lazy/eager) use the standard run interface.
start = fn.index('        #[cfg(feature = "moonshot")]')
end = fn.index("        let n = runs::front",start)
fn = fn[:start]+fn[end:]
fn=fn.replace("runs::takes_runs(b)", "runs::takes_runs_bytes(b, bytes)")
for old in ['runs::front','mtf::mtf','mtf::mtf_checkpoints']:
    fn=fn.replace(old+"(b,",old+"_bytes(b, bytes,")
p.write_text(s+"\n"+fn+"\n")
