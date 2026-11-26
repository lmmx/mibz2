fn emit_pair(edges:&[u32],spelling:&[u8],offsets:&[u32],n:usize,mark:u32,bits:u32,first:u32,out:&mut Vec<u8>){
    out.clear();out.reserve(n);let mask=(1<<bits)-1;let mut row=mark;let mut edge=edges[row as usize];
    loop{
        let code=edge>>bits;row=edge&mask;edge=edges[row as usize];
        if code<256{out.push(code as u8);}
        else if code>=first {let v=code-first;out.extend_from_slice(&[(v>>7) as u8,(v&127) as u8]);}
        else{let k=(code-256) as usize;out.extend_from_slice(&spelling[offsets[k] as usize..offsets[k+1] as usize]);}
        if row==mark{break;}
    }
    repeat(n,out);
}
