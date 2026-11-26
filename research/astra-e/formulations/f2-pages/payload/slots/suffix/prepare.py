"""Reuse the contiguous spelling at a previously published terminal ancestor."""
from pathlib import Path
from hashlib import sha256

here = Path(__file__).resolve().parent
first = (here.parent / 'candidate.rs').read_text()
second = (here.parents[3] / 'f2c/second.rs').read_text()
assert sha256(first.encode()).hexdigest() == '84cc39ea5322f37e5a3da7383ea342da4ed7f14ffd4957496b1de2f92a1cbe06'
assert sha256(second.encode()).hexdigest() == '877dd1870572c4c5237cbdd1362b8d7124c4393abdcbcebb5194e1fadcffc992'
first=first.replace('../../../f2c/second.rs','second.rs').replace('pub type Candidate = Dispatch<2>;', 'pub type Candidate = Dispatch<2,2,3>;')
first=first.replace('const SCHEDULE:u8=1>', 'const SCHEDULE:u8=2,const SPELL:u8=3>').replace('impl<const MODE:u8,const SCHEDULE:u8>', 'impl<const MODE:u8,const SCHEDULE:u8,const SPELL:u8>').replace('Dispatch<MODE,SCHEDULE>','Dispatch<MODE,SCHEDULE,SPELL>').replace('second: second::Second,', 'second: second::Second<SPELL>,\n    #[cfg(spell_counts)]\n    copied: [u64; 4],')
second=second.replace('struct Second {','struct Second<const SPELL:u8> {\n    #[cfg(spell_counts)]\n    pub(super) copied: [u64;4],').replace('impl Second {','impl<const SPELL:u8> Second<SPELL> {')
def materialize(s, flag, variable, store):
    anchor = f'let {variable} = self.nodes[j as usize];'
    assert s.count(anchor)==1
    line_start=s.rindex('\n',0,s.index(anchor))+1
    indent=s[line_start:s.index(anchor)]
    replacement=f'''
if SPELL & {flag} != 0 && ({variable}.word as usize) < self.words.len() {{
    let spelled = self.words[{variable}.word as usize];
    self.{store}.extend_from_within(spelled.id as usize..(spelled.id + spelled.width) as usize);
    #[cfg(spell_counts)]
    {{
        self.copied[2] += 1;
        self.copied[3] += u64::from(spelled.width);
    }}
    break;
}}'''
    return s.replace(anchor,anchor+'\n'+'\n'.join(indent+x for x in replacement.strip().splitlines()))
first=materialize(first,1,'n','dict')
second=materialize(second,2,'node','dictionary')
first=first.replace('                                self.dict.extend(','''                                    #[cfg(spell_counts)]
                                    {
                                        self.copied[0] += 1;
                                        self.copied[1] += u64::from(n.end - n.edge);
                                    }
                                    self.dict.extend(''')
second=second.replace('let w = first_words[sym as usize];\n                                self.dictionary.extend_from_slice(','''let w = first_words[sym as usize];
                                #[cfg(spell_counts)]
                                {
                                    self.copied[0] += 1;
                                    self.copied[1] += u64::from(w.width);
                                }
                                self.dictionary.extend_from_slice(''')
assert 'self.copied[0] += 1;' in first and 'self.copied[0] += 1;' in second
(here/'family.rs').write_text(first)
(here/'second.rs').write_text(second)
