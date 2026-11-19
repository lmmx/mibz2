#pragma once
#include "../../arc.hpp"
#include <cstring>

namespace arc_output {
using arc_cut::U;
using arc_cut::none;
using arc_cut::Grammar;
struct Counts {
    U events=0,literals=0,memo_calls=0,memo_bytes=0,power_calls=0,power_bytes=0,lap_bytes=0;
};
struct Memo {
    struct Task {U node,start;};
    std::vector<U> completed;
    std::vector<Task> stack;
    size_t bytes()const{return completed.capacity()*sizeof(U)+stack.capacity()*sizeof(Task);}
    static void repeat(std::vector<uint8_t>& output,U start,U made,U total) {
        assert(made&&made<=total&&uint64_t(start)+total<=output.size());
        while(made<total){
            U take=std::min(made,total-made);
            std::memcpy(output.data()+start+made,output.data()+start,take);made+=take;
        }
    }
    Counts emit(const Grammar& g,U root,std::vector<uint8_t>& output,U horizon) {
        Counts c;completed.assign(g.nodes.size(),none);stack.clear();stack.push_back({root,none});
        output.resize(horizon);U pos=0;
        while(!stack.empty()){
            auto [id,start]=stack.back();stack.pop_back();++c.events;
            if(id<256){assert(pos<horizon);output[pos++]=uint8_t(id);++c.literals;continue;}
            U slot=id-256;auto node=g.nodes[slot];
            if(start!=none){
                assert(completed[slot]==none);
                if(node.right==none){
                    U base=pos-start;assert(base==g.len(node.left));
                    repeat(output,start,base,node.length);pos=start+node.length;
                    c.power_bytes+=node.length-base;++c.power_calls;
                }
                assert(pos-start==node.length);completed[slot]=start;continue;
            }
            U src=completed[slot];
            if(src!=none){
                assert(src+node.length<=pos&&pos+node.length<=horizon);
                std::memcpy(output.data()+pos,output.data()+src,node.length);
                pos+=node.length;c.memo_bytes+=node.length;++c.memo_calls;continue;
            }
            stack.push_back({id,pos});stack.push_back({node.left,none});
            if(node.right!=none)stack.push_back({node.right,none});
        }
        assert(pos==g.len(root));repeat(output,0,pos,horizon);c.lap_bytes=horizon-pos;
        assert(c.memo_bytes+c.power_bytes+c.literals+c.lap_bytes==horizon);return c;
    }
};
}
