"""A 64-slot exact (grammar handle, terminal byte) cache; a conflict appends normally."""
def apply(endpoint):
    anchor='''        let mut cursors = [0; 256];'''
    assert endpoint.count(anchor)==1
    endpoint=endpoint.replace(anchor,'''        let mut owners = [(0u32, 0u32); 64];
'''+anchor)
    first='''                    stats.words += 1;'''
    assert endpoint.count(first)==1
    endpoint=endpoint.replace(first,'')
    old='''                    e.append(node.word, &mut self.spelling);
                    self.spelling.push(a as u8);
                    self.offsets.push(self.spelling.len() as u32);
                    254 + self.offsets.len() as u32'''
    assert endpoint.count(old)==1
    return endpoint.replace(old,'''                    let slot = (node.word.wrapping_mul(2_654_435_761) >> 26) as usize;
                    let (owner, packed) = owners[slot];
                    if owner == node.word && (packed & 255) == a as u32 {
                        packed >> 8
                    } else {
                        e.append(node.word, &mut self.spelling);
                        self.spelling.push(a as u8);
                        self.offsets.push(self.spelling.len() as u32);
                        let code = 254 + self.offsets.len() as u32;
                        owners[slot] = (node.word, (code << 8) | a as u32);
                        stats.words += 1;
                        code
                    }''')
