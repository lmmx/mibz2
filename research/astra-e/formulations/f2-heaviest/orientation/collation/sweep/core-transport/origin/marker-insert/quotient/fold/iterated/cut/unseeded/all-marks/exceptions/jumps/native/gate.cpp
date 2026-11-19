#include "model.hpp"
#include <filesystem>
#include <iostream>
#include <iterator>
#include <random>
struct Truth{
    std::vector<Byte> last,first;
    std::vector<U> lf,psi;
    explicit Truth(const Input& raw):first(raw.n),lf(raw.n),psi(raw.n){
        std::array<U,256> cursor{};::Geometry geom(raw);std::copy_n(geom.base.begin(),256,cursor.begin());
        for(auto r:raw.runs)last.insert(last.end(),r.width,r.symbol);
        for(U x=0;x<raw.n;x++){Byte a=last[x];U y=cursor[a]++;lf[x]=y;psi[y]=x;first[y]=a;}
    }
    std::vector<Byte> forward(U primary)const{
        std::vector<Byte> out(last.size());U x=primary;
        for(auto& a:out){a=first[x];x=psi[x];}return out;
    }
};
struct Stats{uint64_t columns=0,checks=0,outputs=0,repairs=0,jumps=0;} stats;
void check(Input raw,bool variable,const std::vector<Byte>* gold=nullptr){
    Truth t(raw);auto g=contexts(raw,variable);auto idx=jump::build(raw,g);
    auto two=jump::twice(idx.one,1,raw.n,g);auto four=jump::twice(two,2,raw.n,g);
    std::vector<U> owner(raw.n);
    for(U s=0,left=0;s<g.ends.size();left=g.ends[s++])std::fill(owner.begin()+left,owner.begin()+g.ends[s],s);
    const jump::Layer* layers[]{&idx.one,&two,&four,&idx.eight};
    for(U x=0;x<raw.n;x++){
        U y=x;uint64_t w=0;bool eligible=true;
        for(U j=1;j<=8;j++){
            Byte a=t.last[y];eligible&=a==g.modes[owner[y]];w=(w<<8)|a;y=t.lf[y];
            if(std::has_single_bit(j)){
                auto& layer=*layers[std::countr_zero(j)];assert(layer.mask.test(x)==eligible);
                if(eligible){auto l=layer.link[owner[x]];assert(l.bias+layer.mask.before(x)==y&&l.post==owner[y]&&l.spelling==w);}
                ++stats.checks;
            }
        }
        jump::CursorState p{x,owner[x]};assert(jump::unit(idx,p)==t.last[x]&&p.x==t.lf[x]&&p.c==owner[t.lf[x]]);
    }
    bool all=raw.n<=12;for(U p=all?0:raw.primary;p<(all?raw.n:raw.primary+1);p++){
        idx.primary_state=owner[p];std::vector<Byte> out(raw.n);
        auto repaired=jump::emit(idx,p,out.data());auto want=t.forward(p);
        assert(out==want);if(gold)assert(*gold==out);
        ++stats.outputs;stats.repairs+=repaired!=0;
    }
    ++stats.columns;
    if(gold){
        U row=raw.primary,left=raw.n,units=0,jumps=0,exceptions=0;
        while(left){if(left>=8&&idx.eight.mask.test(row)){
                auto link=idx.eight.link[owner[row]];row=link.bias+idx.eight.mask.before(row);left-=8;jumps++;
            }else{exceptions+=!idx.one.mask.test(row);row=t.lf[row];left--;units++;}}
        std::cout<<','<<raw.n<<','<<g.ends.size()<<','<<g.exceptions<<','<<idx.eight.mask.before(raw.n)<<','<<jumps<<','<<units<<','<<exceptions<<','
            <<idx.bytes()<<','<<idx.construction_peak<<','<<g.bytes()<<','<<raw.runs.capacity()*sizeof(Run)<<'\n';
    }
}
template<class T> std::vector<T> file(std::filesystem::path p){
    std::ifstream f(p,std::ios::binary|std::ios::ate);assert(f);auto bytes=size_t(f.tellg());assert(bytes%sizeof(T)==0);
    std::vector<T> v(bytes/sizeof(T));f.seekg(0);f.read(reinterpret_cast<char*>(v.data()),bytes);assert(f);return v;
}
int main(int argc,char**argv){
    if(argc==1){
        for(U n=1,ways=3;n<=8;n++,ways*=3)for(U code=0;code<ways;code++){
            std::vector<uint16_t> tape;U r=code;
            for(U j=0;j<n;j++,r/=3)tape.push_back(((r%3)<<8)|1);
            auto raw=from_caps(tape.data(),tape.size(),n,0);check(raw,false);check(raw,true);
        }
        std::mt19937 rng(60911);
        for(U i=0;i<380;i++){U n=63+(i%3==0?1U:U(rng()%127));std::vector<uint16_t> tape;
            U period=1+rng()%17;std::vector<U> pattern(period);
            for(auto& a:pattern)a=rng()%8;
            for(U j=0;j<n;j++)tape.push_back(((i%2?pattern[j%period]:rng()%4)<<8)|1);
            auto raw=from_caps(tape.data(),tape.size(),n,rng()%n);check(raw,false);check(raw,true);}
    }else{
        assert(argc==5);auto dir=std::filesystem::path(argv[1]);std::string block=argv[2];U mark=std::stoul(argv[3]);
        auto caps=file<uint16_t>(dir/("block-"+block+".caps"));auto truth=file<Byte>(dir/("block-"+block+".raw"));
        auto raw=from_caps(caps.data(),caps.size(),truth.size(),mark);
        bool var=std::stoi(argv[4])!=0;std::cout<<block<<','<<var;check(raw,var,&truth);
    }
    std::cerr<<"columns "<<stats.columns<<" tested row/horizons "<<stats.checks<<" complete marked outputs "<<stats.outputs<<" repaired "<<stats.repairs<<'\n';
}
