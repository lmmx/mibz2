#define ASTRA_RANK_LIBRARY
#include "../ranks.cpp"
#include <chrono>
#include <cstring>
#include <iomanip>
#include <optional>
#include <stdexcept>
using Capture=BasicConstructor<PagedIndex,true>;
struct Pair{U id,width;};
struct Owned{Byte *data;size_t len,cap;};
struct Object{
    const Pair *root;size_t r;const Pair *words;const U* bases;size_t k;
    const Byte *dict;size_t bytes;U mark,skip,tokens,n;
};
extern "C" Owned full(const uint16_t*,size_t,const U*,U,U);
extern "C" Owned consume(const Object*,U*);
extern "C" void compare(const Object*,const uint16_t*,size_t,const U*,U);
extern "C" void release(Owned);
double now(){timespec t;clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t);return t.tv_sec+t.tv_nsec*1e-9;}

struct SectionRanks{
    const Section& s;Vec<U> bases;U width=0;
    explicit SectionRanks(const Section& intervals):s(intervals){
        for(auto [a,b]:s){bases.push_back(width);width+=b-a;}
    }
    U rank(U x) const{
        if(s.size()==1)return x-s[0].first<s[0].second-s[0].first?x-s[0].first:nil;
        auto it=std::upper_bound(s.begin(),s.end(),x,[](U v,auto p){return v<p.first;});
        if(it==s.begin())return nil;
        --it;return x<it->second?bases[it-s.begin()]+x-it->first:nil;
    }
};
struct Mark {U rank=nil,skip=0;};
Mark transfer(const Edges& e,const SectionRanks& sec,U p,bool rotate){
    U x=p,skip=0;
    if(!rotate && sec.rank(p)!=nil)return {sec.rank(p),0};
    do{
        auto k=std::prev(std::upper_bound(e[1].begin(),e[1].end(),x,[](U r,Piece a){return r<a.lo;}));
        x=k->dst+x-k->lo;++skip;
        U z=sec.rank(x);if(z!=nil)return {z,skip-U(rotate)};
    }while(x!=p);
    return {};
}

struct TokenObject{
    Vec<Pair> root,words;
    Vec<U> bases;
    Vec<Byte> dict;
    U probes=0, comparisons=0;
    Mark mark;U tokens=0,n=0;
    Object view() const{return {root.data(),root.size(),words.data(),bases.data(),bases.size(),
                           dict.data(),dict.size(),mark.rank,mark.skip,tokens,n};}
};
struct Triple{U left,right;Byte c;};
struct KeySlot{Triple key;U tag=0;};
U hash(Triple t){
    uint64_t v=(uint64_t(t.left)<<32)|t.right;
    v^=uint64_t(t.c)*0x9e3779b97f4a7c15ULL;
    v^=v>>30;v*=0xbf58476d1ce4e5b9ULL;v^=v>>27;v*=0x94d049bb133111ebULL;
    return U(v^(v>>31));
}
struct Published{U dest,source,width,id;};
enum class Assemble{sort,scatter,rank};
#include "rank.hpp"

// Beginning with unsorted physical meetings, charge all dictionary and ordering work.
bool form(const Edges& e,const Capture& c,const std::vector<Band>& bands,
          const SectionRanks& s,U n,U p,bool rotate,Assemble mode,TokenObject& out){
    if(mode==Assemble::rank && s.s.size()==1)
        return rank_form(e,c,bands,s,n,p,rotate,false,out);
    if(mode==Assemble::rank)mode=Assemble::scatter;
    out.tokens=s.width;out.n=n;
    out.mark=transfer(e,s,p,rotate);
    if(out.mark.rank==nil)return false;
    Vec<KeySlot> keys(std::bit_ceil(std::max<size_t>(2,2*bands.size())));
    Vec<Published> parts;
    parts.reserve(bands.size());
    Vec<U> by_dest;
    if(mode==Assemble::scatter)by_dest.resize(s.width,nil);
    int common=-1;
    for(auto b:bands){
        U src=s.rank(b.lo),dst=s.rank(b.dest),w=b.hi-b.lo;
        if(src==nil || dst==nil || s.rank(b.hi-1)!=src+w-1 || s.rank(b.dest+w-1)!=dst+w-1)return false;
        Triple t{b.left,b.right,b.c};U i=hash(t)&(keys.size()-1);
        while(keys[i].tag && !(keys[i].key.left==t.left && keys[i].key.right==t.right && keys[i].key.c==t.c)){
            ++out.probes;i=(i+1)&(keys.size()-1);
        }
        ++out.probes;
        U id;
        if(keys[i].tag)id=keys[i].tag-1;
        else{
            id=out.words.size();keys[i]={t,id+1};
            U base=out.dict.size(),f=c.groups[b.left].depth;
            out.dict.resize(base+f);
            for(U j=b.left;c.groups[j].parent!=nil;j=c.groups[j].parent)
                out.dict[base+c.groups[j].depth-1]=c.groups[j].c;
            out.dict.push_back(b.c);
            for(U j=b.right;c.groups[j].parent!=nil;j=c.groups[j].parent)out.dict.push_back(c.groups[j].c);
            if(rotate){
                Byte first=out.dict[base];
                if(common<0)common=first;
                if(first!=common)return false;
                std::rotate(out.dict.begin()+base,out.dict.begin()+base+1,out.dict.end());
            }
            out.words.push_back({base,U(out.dict.size()-base)});out.bases.push_back(src);
        }
        out.bases[id]=std::min(out.bases[id],src);
        if(mode==Assemble::scatter)by_dest[dst]=parts.size();
        parts.push_back({dst,src,w,id});
    }
    if(mode==Assemble::sort)std::sort(parts.begin(),parts.end(),[&](auto a,auto b){++out.comparisons;return a.dest<b.dest;});
    Vec<U> next=out.bases;
    U row=0,k=0;
    while(row<s.width){
        U i=mode==Assemble::scatter?by_dest[row]:k++;
        if(i>=parts.size())return false;
        auto a=parts[i];
        // Certificate of occurrence owner and induced stable LF at every rank.
        if(a.dest!=row || a.source!=next[a.id])return false;
        next[a.id]+=a.width;
        if(!out.root.empty() && out.root.back().id==a.id)out.root.back().width+=a.width;
        else out.root.push_back({a.id,a.width});
        row+=a.width;
    }
    return row==s.width;
}
void require(bool ok,const char* what){if(!ok)throw std::runtime_error(what);}
void validate(Owned b,const Word& expected){
    require(b.len==expected.size() && std::equal(expected.begin(),expected.end(),b.data),"marked output mismatch");
    release(b);
}
void small(){
    N attempts=0,accepted=0,second_used=0,declined=0,disjoint=0,outputs=0;
    for(U n=1;n<=6;++n)for(U code=0;code<1u<<n;++code){
        std::vector<Run> runs;
        for(U i=0;i<n;++i){
            U b=34+((code>>i)&1);
            if(runs.empty() || runs.back().byte!=b)runs.push_back({b,0});
            ++runs.back().len;
        }
        auto e=build(runs);Oracle ref(runs);
        for(U mask=1;mask<(1u<<n);++mask){
            auto section=section_mask(n,mask);SectionRanks s(section);FreeTrace plan;
            Capture c(e);c.trace=&plan;c.index_edges();c.run(section);
            for(U p=0;p<n;++p){
                auto want=ref.tape(p);
                for(bool rotate:{false,true})for(auto mode:{Assemble::sort,Assemble::scatter,Assemble::rank}){
                    TokenObject data;++attempts;
                    if(form(e,c,plan.matches,s,n,p,rotate,mode,data)){
                        ++accepted;U used=0;auto o=data.view();
                        auto got=consume(&o,&used);validate(got,want);second_used+=used;
                    }else{
                        ++declined;disjoint+=data.mark.rank==nil;
                        auto got=c.marked(p,n);
                        require(std::equal(got.begin(),got.end(),want.begin()),"fallback"); 
                    }
                    ++outputs;
                }
            }
        }
    }
    std::cout<<"marked_objects_tested="<<attempts<<" accepted="<<accepted<<" second_used="<<second_used
             <<" token_fallback="<<accepted-second_used<<" declined="<<declined<<" disjoint="<<disjoint
             <<" verified_outputs="<<outputs<<'\n';
}
#ifndef ASTRA_TOKEN_LIBRARY
int main(int argc,char** argv){
    if(argc==1){small();return 0;}
    require(argc==6,"writer RUNS EXPECTED ORDINAL REPS CSV");
    std::ifstream in(argv[1]);U n,p,r,ns;in>>n>>p>>r>>ns;
    std::vector<Run> runs;std::vector<uint16_t> caps;std::array<U,256> freq{};
    for(U i=0;i<r;++i){
        U c,w;in>>c>>w;runs.push_back({c,w});freq[c]+=w;
        while(w){U l=std::min(w,127u);caps.push_back((c<<8)+l);w-=l;}
    }
    std::ifstream gold(argv[2],std::ios::binary);
    Word expected((std::istreambuf_iterator<char>(gold)),{});
    require(expected.size()==n,"bad expected input");
    auto e=build(runs);
    std::ofstream csv(argv[5],std::ios::app);csv<<std::setprecision(9);
    for(U i=0;i<ns;++i){
        std::string name;U cnt;in>>name>>cnt;Section section(cnt);
        for(auto&[lo,hi]:section)in>>lo>>hi;
        SectionRanks s(section);FreeTrace plan;
        Capture c(e);c.trace=&plan;c.index_edges();c.run(section);
        TokenObject frozen;
        require(form(e,c,plan.matches,s,n,p,true,Assemble::sort,frozen),"frozen not eligible");
        auto frozen_view=frozen.view();
        if(name=="quote")compare(&frozen_view,caps.data(),caps.size(),freq.data(),p);
        {TokenObject raw;require(form(e,c,plan.matches,s,n,p,false,Assemble::scatter,raw),"raw not eligible");
         auto o=raw.view();U used=0;validate(consume(&o,&used),expected);}
        constexpr std::array<int,5> pattern{0,1,4,2,3};
        #ifdef ASTRA_RANK_PANEL
#if defined(ASTRA_ENVELOPE_PANEL)
        const char* names[]={"rank","scatter","sort","compact","second"};
#else
        const char* names[]={"rank","scatter","paid_rank","compact","second"};
#endif
#else
        const char* names[]={"sort","scatter","old","compact","second"};
#endif
        for(int rep=0;rep<std::stoi(argv[4]);++rep)for(int pos=0;pos<5;++pos){
            int arm=(pattern[(rep/5)%2 ? 4-pos:pos]+rep)%5;
            U used=0,probes=0,comparisons=0;N added=0;
            double start=now(),assembly=0,second=0;
            Owned out{};Vec<Byte> old;
            #ifdef ASTRA_RANK_PANEL
            Assemble first_mode=Assemble::rank;
#else
            Assemble first_mode=Assemble::sort;
#endif
            if(arm<=1){
                auto base=mem.live;mem.peak=base;
                TokenObject w;
                require(form(e,c,plan.matches,s,n,p,true,arm==0?first_mode:Assemble::scatter,w),"interchange failed");
                auto o=w.view();assembly=now()-start;
                added=mem.peak-base;probes=w.probes;comparisons=w.comparisons;
                double t=now();out=consume(&o,&used);second=now()-t;
            }else if(arm==2){
#ifdef ASTRA_RANK_PANEL
                auto base=mem.live;mem.peak=base;
#ifdef ASTRA_ENVELOPE_PANEL
                TokenObject w;
                require(form(e,c,plan.matches,s,n,p,true,Assemble::sort,w),"sort interop declined");
                auto o=w.view();assembly=now()-start;added=mem.peak-base;
                probes=w.probes;comparisons=w.comparisons;
                double t=now();out=consume(&o,&used);second=now()-t;
#else
                auto own_e=build(runs);BasicConstructor<PagedIndex> own(own_e);
                own.index_edges();own.run(section);
                TokenObject w;
                require(rank_form(own_e,own,own.bands,s,n,p,true,true,w),"paid interop declined");
                auto o=w.view();assembly=now()-start;added=mem.peak-base;
                double t=now();out=consume(&o,&used);second=now()-t;
#endif
#else
                old=c.indexed_marked(p,n);
#endif
            }
            else if(arm==3)out=full(caps.data(),caps.size(),freq.data(),n,p);
            else {out=consume(&frozen_view,&used);second=now()-start;}
            double elapsed=now()-start;
            if(out.data)validate(out,expected);
            else require(old.size()==n && std::equal(old.begin(),old.end(),expected.begin()),"old writer mismatch");
            csv<<argv[3]<<','<<name<<','<<rep<<','<<names[arm]<<','<<n<<','<<s.width<<','<<plan.matches.size()
               <<','<<frozen_view.k<<','<<frozen_view.r<<','<<frozen_view.bytes<<','<<frozen_view.mark<<','<<frozen_view.skip
               <<','<<used<<','<<probes<<','<<comparisons<<','<<added<<','<<assembly<<','<<second<<','<<elapsed<<'\n';
        }
    }
}
#endif
