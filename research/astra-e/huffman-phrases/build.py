"""Build isolated historical scanner modules; the normal corpus producer is frozen."""
import argparse
import hashlib
from pathlib import Path
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
PIN="530ed404a014a414ef0b62adfd947fa14d4533e6"

def source(name):
    return subprocess.check_output(["git","-C",ROOT,"show",f"{PIN}:src/{name}.rs"],text=True)


def once(text,old,new):
    if text.count(old)!=1: raise ValueError(f"expected one: {old[:60]}")
    return text.replace(old,new)


def build(dest,mode):
    dest.mkdir(exist_ok=True,parents=True)
    lib=source("lib")
    defs=[]
    for name in ("pub enum ErrorKind {","pub struct Error {"):
        start=lib.index(name)
        end=lib.index("\n}",start)+2
        defs.append("#[derive(Clone,Debug,PartialEq,Eq)]\n"+lib[start:end]+"\n")
    (dest/"errors.rs").write_text("".join(defs))
    (dest/"bits.rs").write_text(source("bits"))
    (dest/"stream.rs").write_text(source("stream"))
    stock=source("block")
    (dest/"stock.rs").write_text(stock)
    modules=["stock"]
    if mode=="census":
        text=once(stock,"    let mut tables = Vec::with_capacity(n_groups);","    let mut observed_lens=Vec::new();\n    let mut tables = Vec::with_capacity(n_groups);")
        text=once(text,"        tables.push(Huff::new(&lens[..alpha]));","        observed_lens.push(lens[..alpha].to_vec());\n        tables.push(Huff::new(&lens[..alpha]));")
        text=once(text,"    unsafe { syms.set_len(n) };","    unsafe { syms.set_len(n) };\n    audit(&syms,&selectors,&observed_lens,&tables,eob);")
        (dest/"census.rs").write_text(text+(HERE/"census.rs").read_text())
        modules.append("census")
    else:
        from native import transform
        for name,width in (("triple",3),("quad",4)):
            (dest/(name+".rs")).write_text(transform(stock,width))
            modules.append(name)
    driver='#![allow(dead_code)]\ninclude!("errors.rs");\nmod bits;\n'
    for name in modules:
        driver+=f'pub mod {name} {{ pub use crate::{{ErrorKind,Error}}; use crate::bits; #[path="{dest}/{name}.rs"] pub mod block; #[path="{dest}/stream.rs"] pub mod stream; }}\n'
    if mode=="census":
        driver+='''fn main(){
            let data=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
            let mut scanner=census::stream::Scanner::new(census::stream::SliceSource(&data));
            let mut reuse=Vec::new();
            println!("block,mode,tables,symbols,body,tail,tail_single,long,one,two,three,four,table_long,table_one,table_two,table_three,table_four,groups");
            while let census::stream::Event::Block{symbols,..}=scanner.next(reuse).unwrap(){reuse=symbols.syms;}
        }'''
    else:
        driver+=(HERE/"native_driver.rs").read_text()
    (dest/"main.rs").write_text(driver)
    flags=["rustc","--edition=2021","-C","opt-level=3","-C","codegen-units=1"]
    subprocess.run([*flags,str(dest/"main.rs"),"-o",str(dest/mode)],check=True)
    print(f"{mode} {hashlib.sha256((dest/mode).read_bytes()).hexdigest()}")

if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("mode",choices=["census","native"])
    p.add_argument("destination",type=lambda s:Path(s).resolve())
    a=p.parse_args()
    build(a.destination,a.mode)
