#include <algorithm>
#include <cassert>
#include <chrono>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <numeric>
#include <string>
#include <sys/resource.h>
#include <time.h>
#include <vector>

// Bounded by the bzip2 pre-RLE1 block limit (900,000). Python checks the
// arbitrary-width family. This diagnostic uses the standard allocator.
using U = uint32_t;
struct Run { U k,n; bool operator==(const Run&) const = default; };
struct Root { std::vector<Run> r; std::vector<U> bytes; U mark; };
struct Word { U base,n; int side; bool operator==(const Word&) const = default; };
struct Interval { U lo,hi,rank,k; };
struct Piece { U lo,hi,dst; };
struct Result { std::vector<Run> r; std::vector<Word> words; U mark=0, queries=0; };
struct Geo { std::vector<U> base; std::vector<Interval> ranges; U m=0,starts=0; };
void append(std::vector<Run>& r,U k,U n) {
    if (!r.empty() && r.back().k==k) r.back().n+=n;
    else r.push_back({k,n});
}
std::vector<U> bases(const Root& in) {
    std::vector<U> b(in.bytes.size()+1);
    for (auto [k,n]:in.r) b[k+1]+=n;
    std::partial_sum(b.begin(),b.end(),b.begin()); return b;
}
Root seed(const Root& in) {
    auto b=bases(in);
    U k=std::upper_bound(b.begin(),b.end(),in.mark)-b.begin()-1;
    U before=in.mark-b[k], after=b[k+1]-in.mark-1;
    std::vector<U> widths;
    if(before) widths.push_back(before);
    widths.push_back(1); if(after) widths.push_back(after);
    U extra=widths.size()-1;
    Root z; z.mark=k+(before!=0); // protected class id after seeding
    for(U t=0;t<in.bytes.size();++t)
        z.bytes.insert(z.bytes.end(),t==k?1+extra:1,in.bytes[t]);
    U j=0,used=0;
    for(auto [t,n]:in.r) {
        if(t!=k) {append(z.r,t+(t>k?extra:0),n);continue;}
        while(n) {
            U take=std::min(n,widths[j]-used);
            append(z.r,k+j,take); used+=take; n-=take;
            if(used==widths[j]) {++j;used=0;}
        }
    }
    return z;
}
Geo geometry(const Root& in) {
    Geo g; g.base=bases(in); g.m=g.base.back();
    U j=0,pos=0;
    for(auto [k,n]:in.r) while(n) {
        U w=std::min(n,g.base[j+1]-pos);
        if(k!=j) {
            g.ranges.push_back({pos,pos+w,g.starts,j});
            g.starts+=w;
        }
        n-=w; pos+=w;
        if(pos==g.base[j+1]) ++j;
    }
    return g;
}
std::vector<U> permutation(const Root& in,const std::vector<U>& b) {
    auto next=b; U pos=0;
    std::vector<U> dest(b.back());
    for(auto [k,n]:in.r) for(U i=0;i<n;++i) dest[next[k]++]=pos++;
    return dest;
}
template<class T>
const T& locate(const std::vector<T>& v,U x) {
    auto it=std::upper_bound(v.begin(),v.end(),x,[](U x,const auto& piece) { return x<piece.lo; });
    assert(it!=v.begin()); --it; assert(x<it->hi); return *it;
}
Result normalize(const Root& original,bool sparse) {
    auto in=seed(original); auto g=geometry(in);
    std::vector<U> dest,rank;
    std::vector<Piece> pieces;
    if(sparse) {
        std::vector<U> hits(in.bytes.size()+1);
        for(auto [k,n]:in.r) ++hits[k+1];
        std::partial_sum(hits.begin(),hits.end(),hits.begin());
        pieces.resize(in.r.size());
        auto cursor=g.base; U pos=0;
        for(auto [k,n]:in.r) {
            pieces[hits[k]++]={cursor[k],cursor[k]+n,pos};
            cursor[k]+=n; pos+=n;
        }
    } else {
        dest=permutation(in,g.base);
        rank.resize(g.m);
        for(auto s:g.ranges) for(U i=s.lo;i<s.hi;++i) rank[i]=s.rank+i-s.lo;
    }
    std::vector<U> column(g.starts);
    Result out; U last=UINT32_MAX,dprev=0; int sprev=0;
    for(auto s:g.ranges) for(U x=s.lo;x<s.hi;++x) {
        U k=s.k,y=x,d=0;
        U lo=g.base[k],hi=g.base[k+1];
        do {
            if(!sparse) {y=dest[y];++d;++out.queries;continue;}
            auto p=locate(pieces,y); ++out.queries;
            int delta=int(p.dst)-int(p.lo);
            int to=int(y)+delta;
            if(to<int(lo)||to>=int(hi)) { y=U(to);++d;break; }
            assert(delta!=0);
            int low=std::max(int(p.lo),int(lo)-delta);
            int high=std::min(int(p.hi),int(hi)-delta);
            int n=delta>0?(high-1-int(y))/delta+1:(int(y)-low)/(-delta)+1;
            y=U(int(y)+n*delta); d+=U(n);
        } while(y>=lo&&y<hi);
        int side=y<lo?-1:1;
        if(k!=last||d!=dprev||side!=sprev) {
            out.words.push_back({in.bytes[k],d,side}); last=k;dprev=d;sprev=side;
        }
        U a=out.words.size()-1;
        if(k==in.mark) out.mark=a;
        if(sparse) {auto t=locate(g.ranges,y);column[t.rank+y-t.lo]=a;}
        else column[rank[y]]=a;
    }
    for(U k:column) append(out.r,k,1);
    return out;
}
std::vector<uint8_t> direct(const Root& in) {
    auto b=bases(in);auto next=permutation(in,b);
    std::vector<uint8_t> bytes(b.back()),out(b.back());
    for(U k=0;k<in.bytes.size();++k)
        std::fill(bytes.begin()+b[k],bytes.begin()+b[k+1],in.bytes[k]);
    U x=in.mark;
    for(auto &v:out) {v=bytes[x];x=next[x];}
    return out;
}
Root load(const char* path) {
    Root out; std::ifstream input(path); U k,n; input>>out.mark;
    std::vector<U> freq(256),ids(256);
    while(input>>k>>n) {out.r.push_back({k,n});freq[k]+=n;}
    for(U i=0;i<256;++i) if(freq[i]) {ids[i]=out.bytes.size();out.bytes.push_back(i);}
    for(auto& r:out.r) r.k=ids[r.k];
    return out;
}
uint64_t cpu() {
    timespec t; clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t); return uint64_t(t.tv_sec)*1000000000+t.tv_nsec;
}
#include "interval.hpp"
long resident_peak() {
    std::ifstream status("/proc/self/status");
    std::string key,value;
    while(status>>key) {
        if(key=="VmHWM:") {long kib;status>>kib;return kib;}
        std::getline(status,value);
    }
    return -1;
}
#ifndef POWER_PHASE_LIBRARY
int main(int argc,char** argv) {
    assert(argc>=3); Root in=load(argv[1]);std::string mode=argv[2];
    if(mode=="check") {
        auto a=normalize(in,true),b=normalize(in,false);assert(a.words==b.words&&a.r==b.r&&a.mark==b.mark);
        auto c=interval(in);assert(c.words==b.words&&c.r==b.r&&c.mark==b.mark);
        std::filesystem::path p=argv[3];
        std::ofstream r(p/"native.rle"), w(p/"native.words"), raw(p/"direct.bin",std::ios::binary);
        for(auto [k,n]:a.r) r<<k<<' '<<n<<'\n';
        w<<a.mark<<'\n'; for(auto [k,n,side]:a.words) w<<k<<' '<<n<<'\n';
        auto data=direct(in);raw.write(reinterpret_cast<const char*>(data.data()),data.size());return 0;
    }
    int repetitions=std::stoi(argv[3]);
    std::cout<<"round,arm,thread_ns,result_size,lookups,maxrss_kib\n";
    for(int i=0;i<repetitions;++i) {
        for(int j=0;j<(mode=="panel"?4:1);++j) {
            const char* arms[]={"sparse","dense","interval","direct"};
            int slot=i%8<4?(i+j)%4:(i+3-j)%4;
            std::string arm=mode=="panel"?arms[slot]:mode;
            U size=0,lookups=0;uint64_t start=cpu();
            if(arm=="direct") {
                auto result=direct(in);size=result.size();asm volatile("" : : "g"(result.data()) : "memory");
            } else {
                auto result=arm=="interval"?interval(in):normalize(in,arm=="sparse");size=result.r.size();lookups=result.queries;
                asm volatile("" : : "g"(result.r.data()) : "memory");
            }
            uint64_t ns=cpu()-start;
            std::cout<<i<<','<<arm<<','<<ns<<','<<size<<','<<lookups<<','<<resident_peak()<<'\n';
        }
    }
}

#endif
