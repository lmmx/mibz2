import os, shutil

ROOT = "/tmp/cks_c36"
BASE = f"{ROOT}/cand_c37_cap_pt64"

def clone(name):
    dst = f"{ROOT}/{name}"
    if os.path.exists(dst):
        shutil.rmtree(dst)
    shutil.copytree(BASE, dst)
    return dst

def patch_inline_handles(dst, trigram=False):
    """
    Zero-rule-allocation inline handles inside 22-bit payload of 32-bit GradedHandle:
      grade == 0: empty (0)
      grade == 1: (1 << 22) | b0                          (8 bits, 0 rule slots!)
      grade == 2: (2 << 22) | b0 | (b1 << 8)              (16 bits, 0 rule slots!)
      [if trigram]: grade == 3 & bit21 == 1:
                  (3 << 22) | (1 << 21) | b0 | (b1 << 7) | (b2 << 14)  (7-bit ASCII, 0 rule slots!)
      otherwise:  (grade.min(1023) << 22) | rule_id       (rule_id in 0..2^21, rule idx = base + rule_id * 4)
    Long rules store FULL 32-bit graded handles `[left_h, right_h, len, u32::MAX]`!
    """
    ep_p = f"{dst}/endpoint.rs"
    with open(ep_p) as f:
        e = f.read()

    old_lit = """fn literal(byte: u8) -> u32 {
    (1 << ID_BITS) | (u32::from(byte) + 1)
}"""
    new_lit = """#[inline(always)]
fn literal(byte: u8) -> u32 {
    (1 << ID_BITS) | u32::from(byte)
}"""
    assert old_lit in e
    e = e.replace(old_lit, new_lit, 1)

    old_rule_to_join = """    #[inline(always)]
    fn rule(&self, w: u32) -> usize {
        self.base + (w - 257) as usize * RULE
    }

    #[inline(always)]
    fn len(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else {
            let r = self.rule(w);
            let tag = self.a[r + 3];
            if tag >> 28 == 0 {
                tag >> 24
            } else {
                self.a[r + 2]
            }
        }
    }

    /// The low 120 bits are in spelling order. The length byte is metadata.
    #[inline(always)]
    fn short_payload(&self, id: u32) -> u128 {
        if id <= 256 {
            return u128::from(id - 1);
        }
        let r = self.rule(id);
        u128::from(self.a[r])
            | (u128::from(self.a[r + 1]) << 32)
            | (u128::from(self.a[r + 2]) << 64)
            | (u128::from(self.a[r + 3] & 0x00ff_ffff) << 96)
    }

    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let id = ((self.top - self.base) / RULE) as u32 + 257;
        let (left_len, right_len) = (left >> ID_BITS, right >> ID_BITS);
        let (left, right) = (left & ID_MASK, right & ID_MASK);
        let len = if left_len < SATURATED && right_len < SATURATED {
            left_len + right_len
        } else {
            self.len(left) + self.len(right)
        };
        let rule = if len < INLINE as u32 {
            let bytes = self.short_payload(left)
                | (self.short_payload(right) << (8 * left_len))
                | (u128::from(len) << 120);
            [
                bytes as u32,
                (bytes >> 32) as u32,
                (bytes >> 64) as u32,
                (bytes >> 96) as u32,
            ]
        } else {
            [left, right, len, u32::MAX]
        };
        self.a[self.top..self.top + RULE].copy_from_slice(&rule);
        self.top += RULE;
        (len.min(SATURATED) << ID_BITS) | id
    }"""

    if not trigram:
        new_rule_to_join = """    #[inline(always)]
    fn rule(&self, id: u32) -> usize {
        self.base + id as usize * RULE
    }

    #[inline(always)]
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> ID_BITS;
        if g < SATURATED {
            g
        } else {
            self.a[self.rule(h & ID_MASK) + 2]
        }
    }

    /// Low 120 bits in spelling order for any non-empty handle with `grade < 16`.
    /// Both `grade == 1` (literal) and `grade == 2` (inline bigram) decode in 0 memory loads!
    #[inline(always)]
    fn short_payload_h(&self, h: u32) -> u128 {
        let g = h >> ID_BITS;
        let id = h & ID_MASK;
        if g <= 2 {
            return u128::from(id);
        }
        let r = self.rule(id);
        let sl = &self.a[r..r + 4];
        u128::from(sl[0])
            | (u128::from(sl[1]) << 32)
            | (u128::from(sl[2]) << 64)
            | (u128::from(sl[3] & 0x00ff_ffff) << 96)
    }

    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let (left_len, right_len) = (left >> ID_BITS, right >> ID_BITS);
        let len = if left_len < SATURATED && right_len < SATURATED {
            left_len + right_len
        } else {
            self.len_handle(left) + self.len_handle(right)
        };
        if len == 2 {
            // Zero rule allocation: pack 2-byte bigram directly into 22-bit handle payload!
            return (2 << ID_BITS) | (left & 0xFF) | ((right & 0xFF) << 8);
        }
        let id = ((self.top - self.base) / RULE) as u32;
        let rule = if len < INLINE as u32 {
            let bytes = self.short_payload_h(left)
                | (self.short_payload_h(right) << (8 * left_len))
                | (u128::from(len) << 120);
            [
                bytes as u32,
                (bytes >> 32) as u32,
                (bytes >> 64) as u32,
                (bytes >> 96) as u32,
            ]
        } else {
            [left, right, len, u32::MAX]
        };
        self.a[self.top..self.top + RULE].copy_from_slice(&rule);
        self.top += RULE;
        (len.min(SATURATED) << ID_BITS) | id
    }"""
    else:
        new_rule_to_join = """    #[inline(always)]
    fn rule(&self, id: u32) -> usize {
        self.base + id as usize * RULE
    }

    #[inline(always)]
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> ID_BITS;
        if g < SATURATED {
            g
        } else {
            self.a[self.rule(h & ID_MASK) + 2]
        }
    }

    /// Low 120 bits in spelling order for any non-empty handle with `grade < 16`.
    /// Decodes `grade == 1` (literal), `grade == 2` (bigram), and `grade == 3` 7-bit ASCII (bit 21 = 1) in 0 loads!
    #[inline(always)]
    fn short_payload_h(&self, h: u32) -> u128 {
        let g = h >> ID_BITS;
        let id = h & ID_MASK;
        if g <= 2 {
            return u128::from(id);
        }
        if g == 3 && (id & (1 << 21)) != 0 {
            let b0 = id & 0x7F;
            let b1 = (id >> 7) & 0x7F;
            let b2 = (id >> 14) & 0x7F;
            return u128::from(b0 | (b1 << 8) | (b2 << 16));
        }
        let r = self.rule(id);
        let sl = &self.a[r..r + 4];
        u128::from(sl[0])
            | (u128::from(sl[1]) << 32)
            | (u128::from(sl[2]) << 64)
            | (u128::from(sl[3] & 0x00ff_ffff) << 96)
    }

    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let (left_len, right_len) = (left >> ID_BITS, right >> ID_BITS);
        let len = if left_len < SATURATED && right_len < SATURATED {
            left_len + right_len
        } else {
            self.len_handle(left) + self.len_handle(right)
        };
        if len <= 3 {
            if len == 2 {
                return (2 << ID_BITS) | (left & 0xFF) | ((right & 0xFF) << 8);
            }
            // len == 3: either (1+2) or (2+1), both operands are inline (0 memory loads!)
            let (b0, b1, b2) = if left_len == 1 {
                (left & 0xFF, right & 0xFF, (right >> 8) & 0xFF)
            } else {
                (left & 0xFF, (left >> 8) & 0xFF, right & 0xFF)
            };
            if (b0 | b1 | b2) < 128 {
                return (3 << ID_BITS) | (1 << 21) | b0 | (b1 << 7) | (b2 << 14);
            }
        }
        let id = ((self.top - self.base) / RULE) as u32;
        let rule = if len < INLINE as u32 {
            let bytes = self.short_payload_h(left)
                | (self.short_payload_h(right) << (8 * left_len))
                | (u128::from(len) << 120);
            [
                bytes as u32,
                (bytes >> 32) as u32,
                (bytes >> 64) as u32,
                (bytes >> 96) as u32,
            ]
        } else {
            [left, right, len, u32::MAX]
        };
        self.a[self.top..self.top + RULE].copy_from_slice(&rule);
        self.top += RULE;
        (len.min(SATURATED) << ID_BITS) | id
    }"""

    assert old_rule_to_join in e
    e = e.replace(old_rule_to_join, new_rule_to_join, 1)

    # Update append() to operate on full 32-bit graded handles!
    old_append = """    fn append(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let r = self.rule(id);
                let tag = self.a[r + 3];
                if tag >> 28 == 0 {
                    let bytes = self.short_payload(id).to_le_bytes();
                    dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
                    continue;
                }
                let rule = &mut self.a[r..r + RULE];
                if tag == u32::MAX {
                    rule[3] = 0x8000_0000 | dest.len() as u32;
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    dest.extend_from_within(start..start + rule[2] as usize);
                }
            }
        }
    }"""

    new_append = """    fn append(&mut self, handle: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(handle);
        while let Some(h) = stack.pop() {
            let g = h >> ID_BITS;
            if g < INLINE as u32 {
                let bytes = self.short_payload_h(h).to_le_bytes();
                dest.extend_from_slice(&bytes[..g as usize]);
            } else {
                let r = self.rule(h & ID_MASK);
                let tag = self.a[r + 3];
                let rule = &mut self.a[r..r + RULE];
                if tag == u32::MAX {
                    rule[3] = 0x8000_0000 | dest.len() as u32;
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    dest.extend_from_within(start..start + rule[2] as usize);
                }
            }
        }
    }"""
    assert old_append in e
    e = e.replace(old_append, new_append, 1)

    # Update inline_word, Dictionary::code / code_miss, and S3 stats call sites!
    e = e.replace("self.short_payload(id)", "self.short_payload_h(word)")
    e = e.replace("self.append(id, stack, spelling);", "self.append(word, stack, spelling);")
    # In Dictionary::code: key MUST distinguish inline handles from rule ids!
    # Wait: key = handle ^ (byte << ...) OR since handle's top 10 bits include grade, `handle` itself is UNIQUE across ALL literals, bigrams, trigrams, and rules!
    # Check: `handle` is 32 bits, `byte` is 8 bits -> use `(handle.wrapping_mul(0x9E37_79B9) ^ u32::from(byte))`? NO, pair_ht stores `key` and compares EXACT equality `self.pairs[slot] == key`!
    # So `key` MUST be injective in `(handle, byte)`!
    # Look: in `pairs`: can `id = handle & ID_MASK` collide between grade 1 (`0..255`), grade 2 (`0..65535`), and rule ids (`0..rules`)? YES if we only kept `id << 8 | byte`!
    # FIX: Include `(grade <= 3)` distinction OR keep `rule_id` starting at `65_536`? Wait: if `rule_id` starts at `65_536` (`+ 65_536`)? No, `id` fits in 21 bits (`2,097,152` >> `65,536 + 100,000`!).
    # EVEN BETTER: For `grade == 1`: `0..255`, `grade == 2`: `256..65791`? No, that ruins `u128::from(id)`!
    # SIMPLEST INJECTIVE `key` (`32` bits!): `pairs` is ONLY a 1024-entry cache — only store `grade >= 4` (rule ids) in `pairs`? No, `grade == 1,2,3` repeat a lot!
    # Look at `key` bits: `grade` is `1..=1023`, `id` for rules is `< 131_072` (17 bits!), and `byte` is 8 bits!
    # WAIT: why not make `pairs` key check `handle` ONLY within the SAME `byte`?!
    # OH!! DUH!! `S3a` loops `for a in 0..256` — EVERY BAND IN ONE ITERATION HAS THE **EXACT SAME BYTE `a`**!!
    # If we store `(a << 24)` or simply clear `pairs`? No, don't clear `pairs` 256 times!
    # Wait: `pair_ht` has `u32` slots `[key, val]`:
    # - `grade == 1` (`id in 0..256`): `tag = 0`, `id` in `0..256` -> `(0 << 22) | id`
    # - `grade == 2` (`id in 0..65536`): `(1 << 22) | id`
    # - `grade == 3` inline (`id` has bit 21): `id` is 21 bits (`21 + 8 = 29` bits, fits in `u32`? Not with tag!).
    # WAIT A SECOND! Look at `grade == 1` and `grade == 2`:
    # For `grade == 1`: `b0` is 8 bits, `byte` is 8 bits -> total 16 bits!
    # For `grade == 2`: `(b0, b1)` is 16 bits, `byte` is 8 bits -> total **24 bits** (`(1 << 24) | (id << 8) | byte` -> **25 bits**, fits in `u32`!).
    # For `rule_id` (`id < 2^20`, i.e., up to 1,048,576 rules — actual max is `~45,000`!):
    #   `(2 << 24) | (id << 8) | byte` (or `((id + 65_792) << 8) | byte`)!
    #   Wait: `65_792 + 100_000 = 165_792 < 16,777,216 = 2^24`!!
    #   HOLD EVERYTHING! CHECK THAT INJECTION:
    #   - `grade == 1`: `uid = id` (`0..256`) -> `(uid << 8) | byte` (`0 .. 65_536`)
    #   - `grade == 2`: `uid = 256 + id` (`256..65_792`) -> `(uid << 8) | byte`
    #   - `grade >= 3` rule: `uid = 65_792 + id` (`65_792 .. < 2^24`) -> `(uid << 8) | byte`
    #   - `grade == 3` inline trigram (when `trigram=True`): `uid = (1 << 23) | (id & 0x1F_FFFF)` -> wait: `21 + 8 = 29` bits, plus bit 31 = `1` -> **32 BITS EXACT!!**
    #     Check: `0x8000_0000 | ((id & 0x1F_FFFF) << 8) | byte` -> bits `31=1`, `29..30=0`, `8..28=21-bit trigram`, `0..7=byte`!
    #     And for ALL non-trigrams (`uid < 2^23 = 8,388,608`!): bit `31 == 0` (`(uid << 8) | byte < 0x8000_0000`)!
    #   **100% MATHEMATICALLY BIJECTIVE `u32` KEY FOR EVERY POSSIBLE `(handle, byte)`!!**

    old_dict_code = """    #[inline(always)]
    fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
        if handle == 0 {
            return u32::from(byte);
        }
        let id = handle & ID_MASK;
        let key = (id << 8) | u32::from(byte);
        let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> 23) as usize;
        if !self.pairs.is_empty() && self.pairs[slot] == key {
            return self.pairs[slot + 1];
        }
        self.code_miss(e, handle, id, byte, key, slot)
    }

    fn code_miss<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, id: u32, byte: u8, key: u32, slot: usize) -> u32 {
        let height = (handle >> ID_BITS) as usize;
        if height < INLINE {
            let v = e.short_payload(id) | (u128::from(byte) << (8 * height));"""

    if not trigram:
        new_dict_code = """    #[inline(always)]
    fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
        if handle == 0 {
            return u32::from(byte);
        }
        let g = handle >> ID_BITS;
        let id = handle & ID_MASK;
        let uid = if g == 1 { id } else if g == 2 { 256 + id } else { 65_792 + id };
        let key = (uid << 8) | u32::from(byte);
        let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> 23) as usize;
        if !self.pairs.is_empty() && self.pairs[slot] == key {
            return self.pairs[slot + 1];
        }
        self.code_miss(e, handle, id, byte, key, slot)
    }

    fn code_miss<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, id: u32, byte: u8, key: u32, slot: usize) -> u32 {
        let height = (handle >> ID_BITS) as usize;
        if height < INLINE {
            let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));"""
    else:
        new_dict_code = """    #[inline(always)]
    fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
        if handle == 0 {
            return u32::from(byte);
        }
        let g = handle >> ID_BITS;
        let id = handle & ID_MASK;
        let key = if g == 3 && (id & (1 << 21)) != 0 {
            0x8000_0000 | ((id & 0x1F_FFFF) << 8) | u32::from(byte)
        } else {
            let uid = if g == 1 { id } else if g == 2 { 256 + id } else { 65_792 + id };
            (uid << 8) | u32::from(byte)
        };
        let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> 23) as usize;
        if !self.pairs.is_empty() && self.pairs[slot] == key {
            return self.pairs[slot + 1];
        }
        self.code_miss(e, handle, id, byte, key, slot)
    }

    fn code_miss<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, id: u32, byte: u8, key: u32, slot: usize) -> u32 {
        let height = (handle >> ID_BITS) as usize;
        if height < INLINE {
            let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));"""
    assert old_dict_code in e
    e = e.replace(old_dict_code, new_dict_code, 1)

    e = e.replace("e.append(id, self.stack, self.spelling);", "e.append(handle, self.stack, self.spelling);")
    e = e.replace("e.len(node_word & ID_MASK)", "e.len_handle(node_word)")

    with open(ep_p, "w") as f:
        f.write(e)

def patch_regelim(dst):
    """Register-cached Euclidean peeling + deferred sentinel/width writes + 1-step adjacent spiral identity elision."""
    ep_p = f"{dst}/endpoint.rs"
    with open(ep_p) as f:
        e = f.read()

    # Add set_links to Links trait for single-store Narrow::insert
    e = e.replace(
        "fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);",
        "fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);\n    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32);",
        1,
    )
    e = e.replace(
        """    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF) | (to << 16);
    }""",
        """    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF) | (to << 16);
    }
    #[inline(always)]
    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32) {
        a[id as usize * 4 + side] = prev | (next << 16);
    }""",
        1,
    )
    e = e.replace(
        """    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side + 1] = to;
    }""",
        """    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side + 1] = to;
    }
    #[inline(always)]
    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32) {
        a[id as usize * 6 + 2 * side] = prev;
        a[id as usize * 6 + 2 * side + 1] = next;
    }""",
        1,
    )
    e = e.replace(
        """        L::set_prev(self.a, id, side, at);
        L::set_next(self.a, id, side, next);""",
        "        L::set_links(self.a, id, side, at, next);",
        1,
    )

    old_elim = """    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
        let (mut left, mut taken) = (*remaining, *steps);
        let room = if SPIRALS { SPIRAL_RULES * RULE } else { RULE };
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + room > self.a.len() {
                break false;
            }
            let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            let (aw, bw) = (self.width(x), self.width(y));
            if SPIRALS && x != y {
                if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    assert!(q * bw < aw && q * bw <= left);
                    let head = self.power(self.word(x), q);
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    taken += q as usize;
                    continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    assert!(q * aw < bw && q * aw <= left);
                    let tail = self.power(self.word(y), q);
                    let word = self.join(self.word(x), tail);
                    self.set_word(x, word);
                    self.set_width(y, bw - q * aw);
                    left -= q * aw;
                    taken += q as usize;
                    continue;
                }
            }
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            taken += 1;
            if x == y {
                self.pop_tail(x, 0);
                self.pop_tail(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            if aw <= bw {
                self.pop_tail(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    self.pop_tail(y, 1);
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                self.pop_tail(y, 1);
                self.insert(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            }
        };
        (*remaining, *steps) = (left, taken);
        done
    }"""

    new_elim = """    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
        let (mut left, mut taken) = (*remaining, *steps);
        if left == 0 {
            return true;
        }
        if SPIRALS {
            let room = SPIRAL_RULES * RULE;
            let done = loop {
                if left == 0 {
                    break true;
                }
                if self.top + room > self.a.len() {
                    break false;
                }
                let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
                let (aw, bw) = (self.width(x), self.width(y));
                if x != y {
                    if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                        let q = (aw - 1) / bw;
                        assert!(q * bw < aw && q * bw <= left);
                        let head = self.power(self.word(x), q);
                        let word = self.join(head, self.word(y));
                        self.set_word(y, word);
                        self.set_width(x, aw - q * bw);
                        left -= q * bw;
                        taken += q as usize;
                        continue;
                    }
                    if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                        let q = (bw - 1) / aw;
                        assert!(q * aw < bw && q * aw <= left);
                        let tail = self.power(self.word(y), q);
                        let word = self.join(self.word(x), tail);
                        self.set_word(x, word);
                        self.set_width(y, bw - q * aw);
                        left -= q * aw;
                        taken += q as usize;
                        continue;
                    }
                }
                let w = aw.min(bw);
                assert!(w > 0 && w <= left);
                left -= w;
                taken += 1;
                if x == y {
                    self.pop_tail(x, 0);
                    self.pop_tail(x, 1);
                    continue;
                }
                let word = self.join(self.word(x), self.word(y));
                if aw <= bw {
                    self.pop_tail(x, 0);
                    self.insert(y, x, 0);
                    self.set_word(x, word);
                    if aw == bw {
                        self.cut(y, 0);
                        self.pop_tail(y, 1);
                    } else {
                        self.set_width(y, bw - w);
                    }
                } else {
                    self.pop_tail(y, 1);
                    self.insert(x, y, 1);
                    self.set_word(y, word);
                    self.set_width(x, aw - w);
                }
            };
            (*remaining, *steps) = (left, taken);
            return done;
        }

        // Fast non-spiral path (`widest <= CERTIFIED_WIDTH`, 100% of Wikidata):
        // Register-cached `(x, y, aw, bw, wx, wy)` with deferred sentinel/width writes & 1-step adjacent identity elision!
        let mut x = L::prev(self.a, 0, 0);
        let mut y = L::prev(self.a, 0, 1);
        let mut aw = self.width(x);
        let mut bw = self.width(y);
        let mut wx = self.word(x);
        let mut wy = self.word(y);
        let done = loop {
            if self.top + RULE > self.a.len() {
                L::set_prev(self.a, 0, 0, x);
                L::set_prev(self.a, 0, 1, y);
                self.set_width(x, aw);
                self.set_width(y, bw);
                break false;
            }
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                taken += 1;
                let px = L::prev(self.a, x, 0);
                let py = L::prev(self.a, x, 1);
                L::set_next(self.a, px, 0, 0);
                L::set_next(self.a, py, 1, 0);
                x = px;
                y = py;
                if left == 0 {
                    L::set_prev(self.a, 0, 0, x);
                    L::set_prev(self.a, 0, 1, y);
                    break true;
                }
                aw = self.width(x);
                bw = self.width(y);
                wx = self.word(x);
                wy = self.word(y);
                continue;
            }
            if aw < bw {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                taken += 1;
                bw -= aw;
                let word = self.join(wx, wy);
                self.set_word(x, word);
                let px = L::prev(self.a, x, 0);
                if px == y {
                    // 1-step adjacent spiral: popping x and inserting after y is the identity on both lists!
                    wx = word;
                } else {
                    self.set_width(x, aw);
                    L::set_next(self.a, px, 0, 0);
                    let ny = L::next(self.a, y, 0);
                    L::set_links(self.a, x, 0, y, ny);
                    L::set_next(self.a, y, 0, x);
                    L::set_prev(self.a, ny, 0, x);
                    x = px;
                    aw = self.width(x);
                    wx = self.word(x);
                }
            } else if aw > bw {
                assert!(bw > 0 && bw <= left);
                left -= bw;
                taken += 1;
                aw -= bw;
                let word = self.join(wx, wy);
                self.set_word(y, word);
                let py = L::prev(self.a, y, 1);
                if py == x {
                    wy = word;
                } else {
                    self.set_width(y, bw);
                    L::set_next(self.a, py, 1, 0);
                    let nx = L::next(self.a, x, 1);
                    L::set_links(self.a, y, 1, x, nx);
                    L::set_next(self.a, x, 1, y);
                    L::set_prev(self.a, nx, 1, y);
                    y = py;
                    bw = self.width(y);
                    wy = self.word(y);
                }
            } else {
                // aw == bw && x != y (rare equal-width peel: sync sentinels, reuse exact canonical relink!)
                assert!(aw > 0 && aw <= left);
                left -= aw;
                taken += 1;
                let word = self.join(wx, wy);
                self.set_width(x, aw);
                self.set_word(x, word);
                L::set_prev(self.a, 0, 0, x);
                L::set_prev(self.a, 0, 1, y);
                self.pop_tail(x, 0);
                self.insert(y, x, 0);
                self.cut(y, 0);
                self.pop_tail(y, 1);
                if left == 0 {
                    break true;
                }
                x = L::prev(self.a, 0, 0);
                y = L::prev(self.a, 0, 1);
                aw = self.width(x);
                bw = self.width(y);
                wx = self.word(x);
                wy = self.word(y);
            }
        };
        (*remaining, *steps) = (left, taken);
        done
    }"""

    assert old_elim in e
    e = e.replace(old_elim, new_elim, 1)
    with open(ep_p, "w") as f:
        f.write(e)

d = clone("cand_c38_inl2");     patch_inline_handles(d, trigram=False)
d = clone("cand_c38_inl23");    patch_inline_handles(d, trigram=True)
d = clone("cand_c38_regelim");  patch_regelim(d)
d = clone("cand_c38_i2_reg");   patch_inline_handles(d, trigram=False); patch_regelim(d)
d = clone("cand_c38_i23_reg");  patch_inline_handles(d, trigram=True);  patch_regelim(d)
print("Generated 5 Cycle 38 S2b candidates successfully!")
