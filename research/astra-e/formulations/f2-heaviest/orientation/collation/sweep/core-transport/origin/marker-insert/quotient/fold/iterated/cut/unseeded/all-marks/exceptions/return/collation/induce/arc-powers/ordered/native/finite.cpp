#include "terminal.hpp"
#include "../ordered.hpp"
#include <iostream>
using namespace arc_cut;
using Bytes=std::vector<uint8_t>;
void same(Grammar& g,U root,U horizon,arc_output::Memo& memo){
    Bytes plain,fast;g.emit(root,plain,horizon);auto c=memo.emit(g,root,fast,horizon);
    assert(plain==fast&&c.literals+c.memo_bytes+c.power_bytes+c.lap_bytes==horizon);
}
int main(){
    arc_output::Memo memo;Grammar g;U a=g.join('a','b'),b=g.repeat(a,3),c=g.join(b,b),d=g.repeat(c,4),root=g.join(d,c);
    same(g,root,g.len(root)*2+7,memo);Bytes out;auto counts=memo.emit(g,root,out,g.len(root)*2+7);
    assert(counts.power_calls==2&&counts.memo_calls&&counts.lap_bytes==g.len(root)+7);
    U cases=0,powers=0,nondividing=0;
    for(U n=1;n<=6;++n)for(U split=0;split<(1u<<(n-1));++split){
        Exchange e;U start=0;
        for(U i=1;i<=n;++i)if(i==n||((split>>(i-1))&1)){
            e.arcs.push_back({i-start,U(e.arcs.size())%3});start=i;
        }
        e.bottom.resize(e.arcs.size());std::iota(e.bottom.begin(),e.bottom.end(),0);
        do{
            auto dst=bases(e);std::vector<U> right(n),label(n);
            for(U i=0,x=0;i<e.arcs.size();++i)for(U j=0;j<e.arcs[i].width;++j,++x){
                right[dst[i]+j]=x;label[x]=e.arcs[i].rule;
            }
            for(U mark=0;mark<n;++mark){
                Bytes expected;U x=mark;
                for(U j=0;j<n;++j){x=right[x];expected.push_back(label[x]);}
                for(U arm=0;arm<2;++arm){
                    Grammar tree;std::vector<Layer> layers;
                    U rule=arm?arc_order::solve(e,mark,tree,layers):arc_cut::solve(e,mark,tree,layers);
                    U lap=tree.len(rule);assert(lap&&lap<=n);nondividing+=(n%lap!=0);
                    auto stats=memo.emit(tree,rule,out,n);powers+=(stats.power_calls!=0);
                    same(tree,rule,n,memo);assert(out==expected);++cases;
                }
            }
        }while(std::next_permutation(e.bottom.begin(),e.bottom.end()));
    }
    assert(cases&&powers&&nondividing);
    std::cout<<"exchange_arm_marks="<<cases<<" outputs_with_reached_power="<<powers<<" nondividing="<<nondividing
        <<" shared_power_memo_calls="<<counts.memo_calls<<" shared_power_copied="<<counts.power_bytes<<" shared_lap_repair="<<counts.lap_bytes<<"\n";
}
