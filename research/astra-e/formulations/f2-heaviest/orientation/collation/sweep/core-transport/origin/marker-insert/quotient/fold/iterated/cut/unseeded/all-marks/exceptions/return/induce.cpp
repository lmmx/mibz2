#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iostream>
#include <numeric>
#include <vector>
using U=uint32_t; using V=uint64_t;
constexpr U invalid=UINT32_MAX, stop=1u<<31;
struct Run { U x,w,y,c; };
struct Range { U lo,hi,base; };
struct Atom { U x,y; };
struct Packet { U src,x,w,time,atom; };
struct Return { U src,y,dt; };
struct Cost {
    V hops=0,rows=0,single=0,splits=0,run_splits=0,preimage_splits=0,lookups=0,comparisons=0,clock_comparisons=0;
    U anchors=0,roots=0,runs=0,raw_atoms=0,atoms=0,leaves=0,merged=0,depth=0,mark=0,lap=0,events=0,mark_visits=0;
    V overlay_bytes=0,walk_bytes=0;
};
template<class T> V cap(const std::vector<T>& a) { return a.capacity()*sizeof(T); }
U read32(std::istream& f) {
    unsigned char b[4]; f.read(reinterpret_cast<char*>(b),4);assert(f);
    return U(b[0])|(U(b[1])<<8)|(U(b[2])<<16)|(U(b[3])<<24);
}
struct Input {std::vector<uint8_t> l,mask; U primary;};
struct Physical {
    U n,mark=invalid; bool true_mark;
    std::vector<Run> runs; std::vector<Range> u;
    explicit Physical(const Input& in):n(in.l.size()),true_mark(in.mask[in.primary]) {
        assert(n&&n<stop&&in.mask.size()==n&&in.primary<n);
        std::array<U,256> freq{},offset{};
        for(U x=0;x<n;){
            U y=x+1;while(y<n&&in.l[y]==in.l[x])++y;
            runs.push_back({x,y-x,0,in.l[x]});freq[in.l[x]]+=y-x;x=y;
        }
        std::partial_sum(freq.begin(),freq.end()-1,offset.begin()+1);
        for(auto& r:runs){r.y=offset[r.c];offset[r.c]+=r.w;}
        U base=0;
        for(U x=0;x<n;){
            if(!in.mask[x]&&x!=in.primary){++x;continue;}
            U y=x+1;while(y<n&&(in.mask[y]||y==in.primary))++y;
            u.push_back({x,y,base});
            if(x<=in.primary&&in.primary<y)mark=base+in.primary-x;
            base+=y-x;x=y;
        }
        assert(mark!=invalid);
    }
};

// Atom.y is a physical LF coordinate when it continues, or stop|dense_U_rank on a return.
struct Map {
    std::vector<Atom> a;
    std::vector<uint8_t> run_cut;
    std::vector<U> page;
    Map(const Physical& p,bool merge,Cost& c) {
        const U R=p.runs.size();
        std::vector<U> image_order(R),starts(R+1);
        std::array<U,256> bins{},cur{};
        for(auto r:p.runs)++bins[r.c];
        std::partial_sum(bins.begin(),bins.end()-1,cur.begin()+1);
        for(U i=0;i<R;i++)image_order[cur[p.runs[i].c]++]=i;
        auto overlay=[&](auto part){
            U j=0,expected=0;
            for(U i:image_order){
                const auto& r=p.runs[i]; assert(r.y==expected);expected+=r.w;
                for(U y=r.y;y<r.y+r.w;){
                    while(j<p.u.size()&&p.u[j].hi<=y)++j;
                    const bool land=j<p.u.size()&&p.u[j].lo<=y;
                    const U edge=j==p.u.size()?p.n:(land?p.u[j].hi:p.u[j].lo);
                    U end=std::min(r.y+r.w,edge);assert(end>y);
                    U dest=land?(stop|(p.u[j].base+y-p.u[j].lo)):y;
                    part(i,r.x+(y-r.y),dest,y==r.y);
                    y=end;
                }
            }
            assert(expected==p.n);
        };
        overlay([&](U i,U,U,bool){++starts[i+1];});
        std::partial_sum(starts.begin(),starts.end(),starts.begin());
        c.raw_atoms=starts.back();
        a.resize(c.raw_atoms);run_cut.resize(c.raw_atoms);
        overlay([&](U i,U x,U y,bool first){U at=starts[i]++;a[at]={x,y};run_cut[at]=first;});
        assert(c.raw_atoms<=R+2*p.u.size());
        U used=0;
        for(U i=0;i<c.raw_atoms;i++){
            if(i)assert(a[i].x>a[i-1].x);
            if(merge&&used&&((a[used-1].y^a[i].y)&stop)==0
               &&a[used-1].y+(a[i].x-a[used-1].x)==a[i].y)continue;
            a[used]=a[i];run_cut[used++]=run_cut[i];
        }
        a.resize(used);run_cut.resize(used);c.atoms=used;
        page.resize((p.n+255)/256+1);
        U i=0;
        for(U j=0;j<page.size();j++){
            U x=std::min(j*256,p.n-1);while(i+1<a.size()&&a[i+1].x<=x)++i;page[j]=i;
        }
        c.overlay_bytes=cap(p.runs)+cap(p.u)+cap(a)+cap(run_cut)+cap(page)+cap(image_order)+cap(starts);
    }
    U find(U x,Cost& c)const{
        ++c.lookups;U i=page[x/256],end=page[x/256+1]+1;
        while(i+1<end){++c.comparisons;U mid=(i+end)/2;if(a[mid].x<=x)i=mid;else end=mid;}
        return i;
    }
};

std::vector<Return> induce(const Physical& p,const Map& map, Cost& c){
    std::vector<Return> out;
    std::vector<Packet> stack;
    c.runs=p.runs.size();c.roots=p.u.size();c.mark=p.mark;
    U covered=0;
    for(auto root:p.u){
        stack.push_back({root.base,root.lo,root.hi-root.lo,0,invalid});
        while(!stack.empty()){
            c.depth=std::max(c.depth,U(stack.size()));
            Packet s=stack.back();stack.pop_back();
            for(;;){
                U i=s.atom==invalid?map.find(s.x,c):s.atom;
                U end=i+1==map.a.size()?p.n:map.a[i+1].x;
                assert(map.a[i].x<=s.x&&s.x<end);
                U w=std::min(s.w,end-s.x);
                if(w<s.w){
                    stack.push_back({s.src+w,s.x+w,s.w-w,s.time,i+1});
                    ++c.splits;
                    if(map.run_cut[i+1])++c.run_splits;else ++c.preimage_splits;
                }
                U y=map.a[i].y+(s.x-map.a[i].x);
                ++c.hops;c.rows+=w;c.single+=w==1;++s.time;assert(s.time<=p.n);
                if(y&stop){
                    y^=stop;++c.leaves;
                    assert(s.src==covered);covered+=w;
                    if(out.empty()||out.back().dt!=s.time||out.back().y+s.src-out.back().src!=y)
                        out.push_back({s.src,y,s.time});
                    break;
                }
                s.x=y;s.w=w;s.atom=invalid;
            }
        }
    }
    c.anchors=covered;c.merged=out.size();
    assert(c.anchors==p.u.back().base+p.u.back().hi-p.u.back().lo);
    assert(c.roots+c.splits==c.leaves&&c.splits+1<=c.atoms&&c.leaves<=c.anchors&&c.hops<=c.rows&&c.rows<=p.n);
    c.walk_bytes=cap(p.runs)+cap(p.u)+cap(map.a)+cap(map.run_cut)+cap(map.page)+cap(out)+cap(stack);
    return out;
}

struct Clock {std::vector<U> times; U length;};
Clock marked(U mark,bool exceptional,const std::vector<Return>& returns,Cost& c) {
    Clock out{{},0};U x=mark;
    do{
        if(x!=mark||exceptional)out.times.push_back(out.length);
        ++c.mark_visits;
        U lo=0,hi=returns.size();
        while(lo+1<hi){++c.clock_comparisons;U m=(lo+hi)/2;if(returns[m].src<=x)lo=m;else hi=m;}
        auto r=returns[lo];
        out.length+=r.dt;x=r.y+x-r.src;
        assert(c.mark_visits<=c.anchors);
    }while(x!=mark);
    c.lap=out.length;c.events=out.times.size();return out;
}

void check(const Input& in,const Physical& p,const std::vector<Return>& returns,const Clock& clock,const Cost& c,const char* path){
    // Independent row-by-row oracle. Constructed after the clock; never an input to induce()/marked().
    const U n=p.n;
    std::vector<uint8_t> f=in.l;std::sort(f.begin(),f.end());
    std::array<std::vector<U>,256> occurrences;for(U i=0;i<n;i++)occurrences[f[i]].push_back(i);
    std::array<U,256> used{};std::vector<U> lf(n),rank(n,invalid),rows;
    for(U i=0;i<n;i++){
        lf[i]=occurrences[in.l[i]][used[in.l[i]]++];
        if(in.mask[i]||i==in.primary){rank[i]=rows.size();rows.push_back(i);}
    }
    assert(rows.size()==c.anchors&&rank[in.primary]==p.mark);
    U band=0; V visits=0;
    for(U s=0;s<rows.size();s++){
        while(band+1<returns.size()&&returns[band+1].src<=s)++band;
        U i=rows[s],dt=0;
        do{i=lf[i];++dt;}while(rank[i]==invalid);
        auto r=returns[band];assert(r.dt==dt&&r.y+s-r.src==rank[i]);visits+=dt;
    }
    assert(visits==c.rows);
    std::vector<U> expected;std::vector<uint8_t> bytes;
    U x=in.primary,t=0;
    do{if(in.mask[x]){expected.push_back(t);bytes.push_back(in.l[x]);}x=lf[x];++t;}while(x!=in.primary);
    assert(t==clock.length&&expected==clock.times);
    if(!path){
        std::vector<U> psi(n);
        for(U i=0;i<n;i++)psi[lf[i]]=i;
        for(U horizon=0;horizon<=2*n+2;horizon++){
            U endpoint=in.primary;
            for(U i=0;i<horizon;i++)endpoint=psi[endpoint];
            const U shift=(clock.length-horizon%clock.length)%clock.length;
            for(U i=0;i<horizon;i++){
                bool found=std::binary_search(clock.times.begin(),clock.times.end(),(shift+i)%clock.length);
                assert(found==bool(in.mask[endpoint]));endpoint=lf[endpoint];
            }
            assert(endpoint==in.primary);
        }
    }
    if(path){
        std::ifstream pred(path,std::ios::binary);char m[8];pred.read(m,8);
        assert(!std::memcmp(m,"PREDCLK1",8)&&read32(pred)==n&&clock.length==n);
        read32(pred);U states=read32(pred),events=read32(pred);pred.ignore(5ull*states);
        assert(events==expected.size());
        for(U i=0;i<events;i++){assert(read32(pred)==clock.times[i]);assert(pred.get()==bytes[i]);read32(pred);}
        assert(pred.peek()==EOF);
    }
}

void test(const Input& in,bool merge,const char* pred,bool print){
    Cost c;Physical p(in);Map map(p,merge,c);auto returns=induce(p,map,c);auto clock=marked(p.mark,p.true_mark,returns,c);
    check(in,p,returns,clock,c,pred);
    if(print)std::cout<<merge<<','<<p.n<<','<<c.runs<<','<<c.anchors<<','<<c.roots<<','<<c.raw_atoms<<','<<c.atoms<<','<<c.hops<<','<<c.rows<<','<<c.single<<','<<c.splits<<','<<c.run_splits<<','<<c.preimage_splits<<','<<c.leaves<<','<<c.merged<<','<<c.lookups<<','<<c.comparisons<<','<<c.depth<<','<<c.mark_visits<<','<<c.events<<','<<c.lap<<','<<c.clock_comparisons<<','<<c.overlay_bytes<<','<<c.walk_bytes<<'\n';
}
int main(int argc,char** argv){
    if(argc==1){
        V checked=0;
        for(U n=1;n<=6;n++)for(U l=0;l<(1u<<n);l++)for(U m=0;m<(1u<<n);m++)for(U p=0;p<n;p++){
            Input in{{},{},p};
            for(U i=0;i<n;i++){in.l.push_back((l>>i)&1);in.mask.push_back((m>>i)&1);}
            for(bool merge:{false,true}){test(in,merge,nullptr,false);++checked;}
        }
        std::cout<<"all_binary_columns_masks_marks_layouts="<<checked<<'\n';return 0;
    }
    assert(argc==3);std::ifstream f(argv[1],std::ios::binary);char magic[8];f.read(magic,8);
    assert(!std::memcmp(magic,"CLKMASK1",8));U n=read32(f),p=read32(f);
    Input in{std::vector<uint8_t>(n),std::vector<uint8_t>(n),p};
    f.read(reinterpret_cast<char*>(in.l.data()),n);f.read(reinterpret_cast<char*>(in.mask.data()),n);
    assert(f&&f.peek()==EOF&&std::all_of(in.mask.begin(),in.mask.end(),[](uint8_t v){return v<=1;}));
    for(bool merge:{false,true})test(in,merge,argv[2],true);
}
