"""Generate isolated fixed-first-ID entries from the frozen stock scanner source."""
from pathlib import Path
from build import once
HERE=Path(__file__).resolve().parent

def transform(stock,width):
    # No changes to selector parsing, long/bad code handling or reader ownership.
    t=stock[:stock.index("#[cfg(test)]\nmod tests")]
    old=t[t.index("const BAD:"):t.index("/// Table lookups")]
    constants=f"""const BAD: u32 = 1 << 31;
const MAX_PHRASE: usize = {width};
const TERMINAL: u32 = 1 << 10;
const FIRST: u32 = 11;
"""
    t=once(t,old,constants)
    t=once(t,"Some(s) => (s as u32) << SYM1 | 1 << COUNT | zn << LEN1 | zn,",
           "Some(s) => (s as u32) << FIRST | u32::from(s as usize + 1 == alpha) * TERMINAL | zn << 4 | zn,")
    a=t.index("        // a second code")
    b=t.index("        h\n",a)
    t=t[:a]+(HERE/"extend.inc.rs").read_text()+t[b:]
    t=once(t,"if e >> COUNT & 3 != 0 {\n            r.skip_short(e >> LEN1 & 31);\n            return Ok((e >> SYM1 & 0x1ff) as u16);",
             "if e & 15 != 0 {\n            r.skip_short(e >> 4 & 15);\n            return Ok((e >> FIRST & 511) as u16);")
    a=t.index("                if e >> COUNT & 3 != 0 {")
    b=t.index("                } else {\n                    let (s, after)",a)
    t=t[:a]+(HERE/"emit.inc.rs").read_text()+t[b:]
    t=t.replace("left >= 2 * STEPS","left >= MAX_PHRASE * STEPS").replace("left < 2 * STEPS","left < MAX_PHRASE * STEPS")
    t=once(t,"while left >= 2 {","while left >= MAX_PHRASE {")
    t=once(t,"if left == 1 {","while left > 0 {")
    a=t.index("        while left > 0 {")
    t=t[:a]+once(t[a:],"            n += 1;\n            if s == eob", "            n += 1;\n            left -= 1;\n            if s == eob")
    return t
