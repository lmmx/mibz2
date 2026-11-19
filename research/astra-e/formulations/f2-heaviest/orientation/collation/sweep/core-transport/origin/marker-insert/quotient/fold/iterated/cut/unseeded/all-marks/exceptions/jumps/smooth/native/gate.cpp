#include "smooth.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <numeric>
#include <random>
#include <vector>

namespace gate {
using Count = uint64_t;

template<class T>
std::vector<T> read_file(const std::filesystem::path& path) {
    std::ifstream f(path, std::ios::binary | std::ios::ate);
    assert(f);
    size_t length = f.tellg();
    assert(length % sizeof(T) == 0);
    std::vector<T> out(length / sizeof(T));
    f.seekg(0);
    f.read(reinterpret_cast<char*>(out.data()), length);
    assert(f);
    return out;
}

struct Truth {
    std::vector<Byte> last, first;
    std::vector<U> lf, psi;

    explicit Truth(const std::vector<uint16_t>& caps) {
        for (auto record : caps) {
            U width = record & 255;
            assert(width && width <= 127);
            last.insert(last.end(), width, record >> 8);
        }
        U n = last.size();
        assert(n);
        psi.resize(n);
        std::iota(psi.begin(), psi.end(), 0);
        std::stable_sort(psi.begin(), psi.end(), [&](U a, U b) { return last[a] < last[b]; });
        lf.resize(n);
        first.resize(n);
        for (U i=0; i<n; ++i) {
            lf[psi[i]] = i;
            first[i] = last[psi[i]];
        }
    }

    // Independent prefix-doubling on (F, Ψ). Equality of tuple components is exact.
    std::vector<U> classes(U depth) const {
        U n = last.size();
        assert(depth && (depth & (depth-1)) == 0);
        std::vector<U> cls(n), next(n), step=psi, doubled(n);
        for (U x=1; x<n; ++x) cls[x] = cls[x-1] + (first[x] != first[x-1]);
        for (U w=1; w<depth; w*=2) {
            for (U x=1; x<n; ++x) {
                auto key = std::pair(cls[x], cls[step[x]]);
                auto previous = std::pair(cls[x-1], cls[step[x-1]]);
                assert(previous <= key);
                next[x] = next[x-1] + (key != previous);
            }
            for (U x=0; x<n; ++x) doubled[x] = step[step[x]];
            step.swap(doubled);
            cls.swap(next);
        }
        return cls;
    }

    std::vector<Byte> forward(U primary) const {
        std::vector<Byte> output(last.size());
        U x=primary;
        for (auto& b : output) { b=first[x]; x=psi[x]; }
        return output;
    }

    U lap(U primary) const {
        U x=primary, count=0;
        do {x=lf[x]; ++count;} while (x!=primary);
        return count;
    }
};

struct Counts {
    Count inputs=0, outputs=0, bytes=0, row_checks=0, smooth=0, branch=0, entirely_smooth=0;
    Count nondividing=0, qualified=0, readonly_bytes=0;
    void add(const Counts& o) {
        inputs+=o.inputs; outputs+=o.outputs; bytes+=o.bytes; row_checks+=o.row_checks;
        smooth+=o.smooth; branch+=o.branch; entirely_smooth+=o.entirely_smooth;
        nondividing+=o.nondividing; qualified+=o.qualified; readonly_bytes+=o.readonly_bytes;
    }
};

struct Classes {
    std::vector<U> id;
    std::vector<bool> branching;
    explicit Classes(const Truth& t,U k) : id(t.classes(k)),branching(id.back()+1,false) {
        for (U x=1; x<id.size(); ++x)
            if (id[x]==id[x-1] && t.last[x]!=t.last[x-1]) branching[id[x]]=true;
    }
};

void every_row(const smooth::Index& measured,const std::vector<uint16_t>& records,
               const Truth& t,const Classes& c,U depth,U mark,Counts& sums) {
    U n=t.last.size();
    auto input=smooth::decode(records.data(),records.size(),n,mark);
    Charge charge;
    auto model=construct(input,depth,charge);
    std::vector<U> new_state;
    auto exposed=smooth::build(input,model.states,&new_state);
    assert(measured.bytes()==exposed.bytes() && new_state.size()==c.branching.size());
    U row=0,state=0;
    for(auto& group:model.states.end) for(U end:group) {
        assert(row<end && end<=n);
        for(; row<end; ++row) assert(c.id[row]==state);
        ++state;
    }
    assert(row==n && state==c.branching.size());
    auto owner=[&](U x){return new_state[c.id[x]];};
    assert(measured.primary.x==mark && measured.primary.state==owner(mark));
    std::vector<Byte> backing(n+64,0xd3);
    Byte* end=backing.data()+32+n;
    for(U x=0; x<n; ++x) {
        smooth::Position step{x,owner(x)};
        auto one=smooth::unit(measured,step);
        assert(one==t.last[x] && step.x==t.lf[x] && step.state==owner(t.lf[x]));
        ++sums.row_checks;
        if(c.branching[c.id[x]]) ++sums.branch; else ++sums.smooth;
        auto check=[&](U cap){
            smooth::Position start{x,owner(x)};
            U used=smooth::group(measured,start,end,cap);
            assert(used && used<=cap);
            U y=x;
            for(U j=0;j<used;++j){
                assert(end[-1-static_cast<int>(j)]==t.last[y]);
                y=t.lf[y];
            }
            assert(start.x==y && start.state==owner(y));
        };
        check(n);
        if(n<=8) for(U cut=1;cut<n;++cut) check(cut);
        else check(std::min(n,1+x%17));
    }
    assert(std::all_of(backing.begin(),backing.begin()+32,[](Byte b){return b==0xd3;}));
    assert(std::all_of(backing.end()-32,backing.end(),[](Byte b){return b==0xd3;}));
}


Counts test(const std::vector<uint16_t>& caps,U k,const std::vector<U>& marks,
            const std::vector<Byte>* gold=nullptr) {
    Truth t(caps);
    Classes classes(t,k);
    U n=t.lf.size();
    Counts c;
    c.inputs=1;
    for (U p : marks) {
        auto idx=smooth::prepare(caps.data(),caps.size(),n,p,k);
        if (!c.outputs) {
            c.readonly_bytes=idx.bytes();
            every_row(idx,caps,t,classes,k,p,c);
        }
        auto want=t.forward(p);
        std::vector<Byte> guarded(n+64,0xa5);
        auto repaired=smooth::emit(idx,guarded.data()+32);
        assert(std::equal(want.begin(),want.end(),guarded.begin()+32));
        assert(std::all_of(guarded.begin(),guarded.begin()+32,[](Byte b){return b==0xa5;}));
        assert(std::all_of(guarded.end()-32,guarded.end(),[](Byte b){return b==0xa5;}));
        if(gold) assert(*gold==want);
        U lap=t.lap(p);
        
        bool all=true, varying=false;
        for (U x=p,remain=lap;remain;--remain,x=t.lf[x]) {
            all &= !classes.branching[classes.id[x]];
            varying |= t.last[x]!=t.last[p];
        }
        assert(repaired==(n%lap && !all ? lap : 0));
        c.nondividing+=n%lap!=0;
        c.entirely_smooth+=all;
        c.qualified+=all && varying && n%lap!=0;
        ++c.outputs; c.bytes+=n;
    }
    return c;
}

std::vector<uint16_t> caps(const std::vector<Byte>& l,U limit) {
    std::vector<uint16_t> out;
    for(Byte b:l) {
        if(!out.empty() && (out.back()>>8)==b && (out.back()&255)<limit) ++out.back();
        else out.push_back((uint16_t(b)<<8)|1);
    }
    return out;
}

void print(const Counts& c) {
    std::cout<<c.inputs<<','<<c.outputs<<','<<c.bytes<<','<<c.row_checks<<','<<c.smooth<<','<<c.branch
        <<','<<c.entirely_smooth<<','<<c.nondividing<<','<<c.qualified<<','<<c.readonly_bytes<<'\n';
}

int main(int argc,char** argv) {
    if(argc==5){
        std::filesystem::path path=argv[1];
        std::string id=argv[2];
        U primary=std::stoul(argv[3]), k=std::stoul(argv[4]);
        auto records=read_file<uint16_t>(path/("block-"+id+".caps"));
        auto gold=read_file<Byte>(path/("block-"+id+".raw"));
        auto c=test(records,k,{primary},&gold);
        std::cout<<id<<','<<k<<',';
        print(c);
        return 0;
    }
    assert(argc==1);
    std::array<U,4> depths{1,4,8,16};
    for(U alphabet:{2u,3u}){
        U ways=1;
        Counts counts;
        for (U n=1;n<=(alphabet==2?8:6);++n) {
            ways*=alphabet;
            for (U code=0;code<ways;++code) {
                std::vector<Byte> l(n);
                U v=code;
                for(auto& b:l){b=std::array<Byte,3>{0,128,255}[v%alphabet];v/=alphabet;}
                std::vector<U> marks(n);
                std::iota(marks.begin(),marks.end(),0);
                auto raw=caps(l,1+(code%127));
                for (U k:depths) counts.add(test(raw,k,marks));
            }
        }
        assert(counts.branch && counts.smooth && counts.qualified);
        std::cout<<"exhaustive-"<<alphabet<<",0,";
        print(counts);
    }
    // Long cap-boundary repeats, high-byte and period-seam cases.
    std::mt19937 rng(2049871);
    Counts varied;
    for(U s=0;s<54;++s) {
        U n=std::array<U,9>{17,63,127,128,129,254,255,256,509}[s%9];
        std::vector<Byte> l(n);
        for(U x=0;x<n;++x) {
            switch(s/9){
                case 0: l[x]=255; break;
                case 1: l[x]=x<n/2?0:255; break;
                case 2: l[x]=std::array<Byte,5>{255,128,0,128,9}[x%5];break;
                case 3: l[x]=static_cast<Byte>(rng());break;
                case 4: l[x]=x==n-2?255:0;break;
                default: l[x]=static_cast<Byte>(x/8);break;
            }
        }
        for(U k:depths) varied.add(test(caps(l,127),k,{0,n/2,n-1}));
    }
    assert(varied.qualified && varied.nondividing && varied.branch);
    std::cout<<"targeted,0,";
    print(varied);
    return 0;
}
} // namespace gate

int main(int argc,char** argv){ return gate::main(argc,argv); }
