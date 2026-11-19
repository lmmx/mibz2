// Physical rank-only asynchronous first-return construction. Build with C++20.
#include <algorithm>
#include <array>
#include <bit>
#include <deque>
#include <cassert>
#include <cstdint>
#include <ctime>
#include <fstream>
#include <iostream>
#include <map>
#include <numeric>
#include <string>
#include <tuple>
#include <vector>

using U = uint32_t;
using N = uint64_t;
using Byte = uint8_t;
using Word = std::vector<Byte>;
using Section = std::vector<std::pair<U,U>>;
struct Run { U byte, len; };
constexpr U nil = UINT32_MAX;

// Count requested bytes, including the standard library's rebound map-node type.
struct Memory { N live=0, peak=0, allocs=0; };
static Memory mem;
template<class T> struct Paid {
    using value_type = T;
    Paid() = default;
    template<class V> Paid(const Paid<V>&) {}
    T* allocate(size_t n) {
        T* p = std::allocator<T>{}.allocate(n);
        mem.live += n*sizeof(T); mem.peak=std::max(mem.live,mem.peak); ++mem.allocs;
        return p;
    }
    void deallocate(T* p,size_t n) {
        mem.live -= n*sizeof(T); std::allocator<T>{}.deallocate(p,n);
    }
    template<class V> bool operator==(const Paid<V>&) const {return true;}
};
template<class T> using Vec=std::vector<T,Paid<T>>;
struct Piece { U lo,hi,dst; Byte c; };
using Edges = std::array<Vec<Piece>,2>;

// F-ordered Psi and L-ordered LF translations. Only original maximal runs are input.
Edges build(const std::vector<Run>& runs) {
    Edges e;
    std::array<U,256> p{}, slot{};
    for(auto [c,w]:runs){p[c]+=w;slot[c]++;}
    std::exclusive_scan(p.begin(),p.end(),p.begin(),0u);
    std::exclusive_scan(slot.begin(),slot.end(),slot.begin(),0u);
    e[0].resize(runs.size()); e[1].reserve(runs.size());
    U l=0;
    for(auto [c,w]:runs){
        e[0][slot[c]++]={p[c],p[c]+w,l,Byte(c)};
        e[1].push_back({l,l+w,p[c],Byte(c)});
        l+=w; p[c]+=w;
    }
    return e;
}

struct Stats {
    N states=0,pops=0,repush=0,shift=0,pieces=0,seeks=0,edge_cmp=0;
    N probes=0,tree_cmp=0,join=0,hit=0,gap=0,op_splits=0,coalesces=0;
    N fragment_inserts=0,fragment_erases=0,peak_map=0,peak_heap=0,sort_cmp=0,heap_cmp=0;
    N mask_reads=0,mask_writes=0,local_hops=0;
};
struct Seg { U lo,hi,owner,g; Seg *prev,*next; };
struct Group { Seg *head; U mass,depth,birth,parent; Byte side,c; };
struct Band { U lo,hi,dest,left,right; Byte c; };
struct Key { U mass,depth,side,birth,g; };
struct Priority {
    N* comparisons=nullptr;
    bool operator()(Key a, Key b) const {
        if(comparisons)++*comparisons;
        if(a.mass!=b.mass)return a.mass<b.mass;
        return std::tie(a.depth,a.side,a.birth)>std::tie(b.depth,b.side,b.birth);
    }
};
struct RowOrder {
    N* comparisons;
    bool operator()(U a,U b) const {++*comparisons;return a<b;}
};
using Map=std::map<U,Seg,RowOrder,Paid<std::pair<const U,Seg>>>;

class TreeIndex : public Map {
public:
    using Map::Map;
    void init(U,Stats&){}
};
#include "paged.hpp"
struct ChildEffect { U id,mass,birth; Byte c; };
struct Effect { U id,hits,hit_count,kids,kid_count; };
struct FreeTrace {
    std::vector<Effect> steps;
    std::vector<ChildEffect> births;
    std::vector<Band> matches;
};
template<class Index,bool Recording=false>
class BasicConstructor {
    const Edges& edges;
    std::array<Index,2> front;
    Vec<Key> heap;
    std::array<Vec<U>,2> page_first;
    U birth=0;
    U new_group(U side,U depth,U parent,Byte c){
        U id=groups.size();
        groups.push_back({nullptr,0,depth,nil,parent,Byte(side),c});
        return id;
    }
    void push(U id){
        auto g=groups[id];
        heap.push_back({g.mass,g.depth,g.side,g.birth,id});
        std::push_heap(heap.begin(),heap.end(),Priority{&stats.heap_cmp});
        stats.peak_heap=std::max<N>(stats.peak_heap,heap.size());
    }
    void attach(U side,U id,U lo,U hi,U owner){
        // Lower bound also finds the two possible affine coalescences.
        auto& m=front[side]; auto r=m.lower_bound(lo);
        bool has_left=r!=m.begin();
        auto l=has_left?std::prev(r):m.end();
        assert((!has_left || l->second.hi<=lo) && (r==m.end() || hi<=r->first));
        if(has_left && l->second.g==id && l->second.hi==lo &&
                l->second.owner+lo-l->first==owner){
            lo=l->first; owner=l->second.owner; remove(side,l); ++stats.coalesces;
        }
        if(r!=m.end() && r->second.g==id && r->first==hi &&
                r->second.owner==owner+hi-lo){
            hi=r->second.hi; r=remove(side,r); ++stats.coalesces;
        }
        auto& g=groups[id];
        auto it=m.emplace_hint(r,lo,Seg{lo,hi,owner,id,nullptr,g.head});
        Seg* p=&it->second;
        if(g.head)g.head->prev=p;
        g.head=p; ++stats.fragment_inserts;
        stats.peak_map=std::max<N>(stats.peak_map,front[0].size()+front[1].size());
    }
    typename Index::iterator remove(U side,typename Index::iterator it){
        auto p=it->second;
        if(p.prev)p.prev->next=p.next;
        else groups[p.g].head=p.next;
        if(p.next)p.next->prev=p.prev;
        ++stats.fragment_erases;
        return front[side].erase(it);
    }
    void advance(U id){
        U side=groups[id].side,opp=1-side,depth=groups[id].depth;
        auto& e=edges[side]; auto& m=front[opp];
        std::array<U,256> child; child.fill(nil);
        U paid=0;
        while(groups[id].head){
            Seg a=*groups[id].head;
            auto it=front[side].find(a.lo);
            remove(side,it);
            ++stats.seeks;
            auto k=e.begin();
            if(page_first[side].empty())
                k=std::prev(std::upper_bound(e.begin(),e.end(),a.lo,[&](U row,Piece p){
                    ++stats.edge_cmp;return row<p.lo;
                }));
            else {
                k+=page_first[side][a.lo/64];
                while(k->hi<=a.lo){++k;++stats.edge_cmp;}
            }
            for(U x=a.lo;x<a.hi;++k){
                U end=std::min(a.hi,k->hi),y=k->dst+(x-k->lo),stop=y+(end-x);
                U own=a.owner+(x-a.lo); Byte byte=k->c;
                ++stats.pieces; ++stats.probes;
                auto q=m.upper_bound(y);
                if(q!=m.begin() && std::prev(q)->second.hi>y)--q;
                while(y<stop){
                    ++stats.join;
                    if(q==m.end() || q->first>y){
                        U next=q==m.end()?stop:std::min(stop,q->first);
                        if(child[byte]==nil)child[byte]=new_group(side,depth+1,id,byte);
                        attach(side,child[byte],y,next,own);
                        groups[child[byte]].mass+=next-y;
                        ++stats.gap; own+=next-y; y=next;
                    }else{
                        Seg b=q->second;
                        U next=std::min(stop,b.hi), other=b.owner+(y-b.lo),w=next-y;
                        Group& g=groups[b.g];
                        assert(g.mass>=w);
                        g.mass-=w;
                        bands.push_back(side==0 ? Band{own,own+w,other,id,b.g,byte}
                                                : Band{other,other+w,own,b.g,id,byte});
                        q=remove(opp,q);
                        if(b.lo<y){attach(opp,b.g,b.lo,y,b.owner);++stats.op_splits;}
                        if(next<b.hi){attach(opp,b.g,next,b.hi,b.owner+(next-b.lo));++stats.op_splits;}
                        ++stats.hit; own+=w; y=next;
                    }
                }
                paid+=end-x; x=end;
            }
        }
        assert(paid==groups[id].mass);
        stats.shift+=paid; groups[id].mass=0;
        for(U c:child)if(c!=nil){groups[c].birth=birth++;push(c);}
    }
public:
    Stats stats;
    FreeTrace* trace=nullptr;
    Vec<Group> groups;
    Vec<Band> bands;
    BasicConstructor(const Edges& e): edges(e),front{Index(RowOrder{&stats.tree_cmp}),Index(RowOrder{&stats.tree_cmp})} {
        for(auto& m:front)m.init(e[0].back().hi,stats);
    }
    void index_edges(){
        for(U side:{0u,1u}){
            U i=0;auto& e=edges[side];
            for(U row=0;row<e.back().hi;row+=64){
                while(e[i].hi<=row)++i;
                page_first[side].push_back(i);
            }
        }
    }
    void run(const Section& section){
        for(U side:{0u,1u}){
            U id=new_group(side,0,nil,0);
            groups[id].birth=birth++;
            for(auto [lo,hi]:section){attach(side,id,lo,hi,lo);groups[id].mass+=hi-lo;}
            push(id);
        }
        while(!heap.empty()){
            std::pop_heap(heap.begin(),heap.end(),Priority{&stats.heap_cmp});
            Key k=heap.back();heap.pop_back();++stats.pops;
            U mass=groups[k.g].mass;
            if(mass==0)continue;
            if(k.mass!=mass){++stats.repush;push(k.g);continue;}
            ++stats.states;
            U before=groups.size(),start=bands.size();
            advance(k.g);
            if constexpr(Recording){
                trace->steps.push_back({k.g,U(trace->matches.size()),U(bands.size()-start),
                                          U(trace->births.size()),U(groups.size()-before)});
                trace->matches.insert(trace->matches.end(),bands.begin()+start,bands.end());
                for(U i=before;i<groups.size();++i){
                    auto g=groups[i];trace->births.push_back({i,g.mass,g.birth,g.c});
                }
            }
        }
        assert(front[0].empty() && front[1].empty());
        std::sort(bands.begin(),bands.end(),[&](auto a,auto b){++stats.sort_cmp;return a.lo<b.lo;});
    }
    template<bool Queue> void replay(const FreeTrace& plan,U endpoints){
        for(U side:{0u,1u}){
            U id=new_group(side,0,nil,0);
            groups[id].mass=endpoints;groups[id].birth=birth++;
            if constexpr(Queue)push(id);
        }
        auto next=[&](){
            while(!heap.empty()){
                std::pop_heap(heap.begin(),heap.end(),Priority{&stats.heap_cmp});
                auto k=heap.back();heap.pop_back();++stats.pops;
                if(!groups[k.g].mass)continue;
                if(groups[k.g].mass!=k.mass){push(k.g);continue;}
                return k.g;
            }
            return nil;
        };
        for(auto a:plan.steps){
            if constexpr(Queue)if(next()!=a.id)throw std::runtime_error("oracle queue order mismatch");
            ++stats.states;auto g=groups[a.id];
            auto lo=plan.matches.begin()+a.hits;
            bands.insert(bands.end(),lo,lo+a.hit_count);
            if constexpr(Queue){
                for(auto i=lo;i!=lo+a.hit_count;++i){
                    U opposite=g.side==0?i->right:i->left;
                    groups[opposite].mass-=i->hi-i->lo;
                }
            }
            for(U i=a.kids;i<a.kids+a.kid_count;++i){
                auto child=plan.births[i];
                U id=new_group(g.side,g.depth+1,a.id,child.c);
                if(id!=child.id)throw std::runtime_error("oracle birth mismatch");
                if constexpr(Queue){
                    groups[id].mass=child.mass;groups[id].birth=child.birth;push(id);
                }
            }
            groups[a.id].mass=0;
        }
        if constexpr(Queue)if(next()!=nil)throw std::runtime_error("oracle queue incomplete");
        std::sort(bands.begin(),bands.end(),[](const Band& a,const Band& b){return a.lo<b.lo;});
    }
    Vec<Byte> indexed_marked(U row,U n) const{
        struct Ref {U base,len;};
        Vec<Ref> refs;
        Vec<Byte> spellings;
        Vec<U> directory(bands.back().hi-bands[0].lo,nil);
        for(U id=0;id<bands.size();++id){
            Band b=bands[id];U width=groups[b.left].depth,base=spellings.size();
            spellings.resize(base+width);
            for(U g=b.left;groups[g].parent!=nil;g=groups[g].parent)
                spellings[base+groups[g].depth-1]=groups[g].c;
            spellings.push_back(b.c);
            for(U g=b.right;groups[g].parent!=nil;g=groups[g].parent)
                spellings.push_back(groups[g].c);
            refs.push_back({base,U(spellings.size()-base)});
            std::fill_n(directory.begin()+(b.lo-bands[0].lo),b.hi-b.lo,id);
        }
        Vec<Byte> out;out.reserve(n);
        while(out.size()<n){
            U offset=row-bands[0].lo;
            U id=offset<directory.size()?directory[offset]:nil;
            if(id==nil){
                auto& e=edges[0];
                U i;
                if(page_first[0].empty()){
                    auto k=std::prev(std::upper_bound(e.begin(),e.end(),row,[](U x,Piece z){return x<z.lo;}));
                    i=k-e.begin();
                }else{
                    i=page_first[0][row/64];while(e[i].hi<=row)++i;
                }
                auto k=e[i];out.push_back(k.c);row=k.dst+row-k.lo;continue;
            }
            auto r=refs[id];auto b=bands[id];
            U take=std::min<U>(n-out.size(),r.len);
            out.insert(out.end(),spellings.begin()+r.base,spellings.begin()+r.base+take);
            row=b.dest+row-b.lo;
        }
        return out;
    }
    Vec<Byte> marked(U row, U n) const{
        Vec<Byte> out;
        out.reserve(n);
        while(out.size()<n){
            auto it=std::upper_bound(bands.begin(),bands.end(),row,[](U x,const Band& b){return x<b.lo;});
            if(it==bands.begin() || row>=std::prev(it)->hi){
                auto& e=edges[0];
                auto k=std::prev(std::upper_bound(e.begin(),e.end(),row,[](U x,const Piece& p){return x<p.lo;}));
                out.push_back(k->c);row=k->dst+row-k->lo;continue;
            }
            Band b=*std::prev(it);
            U prefix=groups[b.left].depth, room=n-out.size();
            size_t begin=out.size();
            out.resize(begin+std::min(prefix,room));
            for(U g=b.left;groups[g].parent!=nil;g=groups[g].parent){
                U i=groups[g].depth-1;
                if(i<room)out[begin+i]=groups[g].c;
            }
            if(out.size()<n)out.push_back(b.c);
            for(U g=b.right;out.size()<n && groups[g].parent!=nil;g=groups[g].parent)
                out.push_back(groups[g].c);
            row=b.dest+row-b.lo;
        }
        return out;
    }
    Word word(const Band& b) const{
        Word w;
        for(U i=b.left;groups[i].parent!=nil;i=groups[i].parent)w.push_back(groups[i].c);
        std::reverse(w.begin(),w.end());
        w.push_back(b.c);
        for(U i=b.right;groups[i].parent!=nil;i=groups[i].parent)w.push_back(groups[i].c);
        return w;
    }
};

#ifdef ASTRA_PAGED
using Constructor=BasicConstructor<PagedIndex>;
#else
using Constructor=BasicConstructor<TreeIndex>;
#endif
struct Oracle {
    std::vector<U> psi;
    Word f;
    Oracle(std::vector<U> p,Word bytes):psi(std::move(p)),f(std::move(bytes)){}
    explicit Oracle(const std::vector<Run>& runs){
        Word l;for(auto [c,w]:runs)l.insert(l.end(),w,c);
        psi.resize(l.size());std::iota(psi.begin(),psi.end(),0u);
        std::stable_sort(psi.begin(),psi.end(),[&](U a,U b){return l[a]<l[b];});
        for(U p:psi)f.push_back(l[p]);
    }
    Word tape(U mark) const{
        Word w;for(U i=0;i<f.size();++i){w.push_back(f[mark]);mark=psi[mark];}return w;
    }
    Edges arbitrary_edges() const{
        Edges e;
        std::vector<U> lf(psi.size());
        for(U i=0;i<psi.size();++i)lf[psi[i]]=i;
        for(U side:{0u,1u})for(U i=0;i<psi.size();++i){
            U y=side?lf[i]:psi[i]; Byte c=side?f[y]:f[i];
            if(!e[side].empty()){
                Piece& p=e[side].back();
                if(p.dst+i-p.lo==y && p.c==c){p.hi++;continue;}
            }
            e[side].push_back({i,i+1,y,c});
        }
        return e;
    }
};

struct Audit {
    std::vector<U> band_by_row;
    std::vector<Word> words;
    std::map<Word,U> counts;
    U covered=0;
    Audit(const Oracle& o,const Constructor& c,const Section& s):band_by_row(o.f.size(),nil){
        std::vector<bool> in(o.f.size()),dest(o.f.size());
        for(auto [a,b]:s)std::fill(in.begin()+a,in.begin()+b,true);
        for(U id=0;id<c.bands.size();++id){
            Band b=c.bands[id]; Word w=c.word(b); words.push_back(w);
            counts[w]+=b.hi-b.lo;
            for(U x=b.lo;x<b.hi;++x){
                U y=x;
                assert(in[x] && band_by_row[x]==nil);band_by_row[x]=id;
                for(U j=0;j<w.size();++j){
                    assert((j==0 || !in[y]) && w[j]==o.f[y]);
                    ++covered;y=o.psi[y];
                }
                assert(in[y] && !dest[y] && y==b.dest+x-b.lo);dest[y]=true;
            }
        }
        assert(in==dest);
        for(U i=0;i<in.size();++i)assert(in[i]==(band_by_row[i]!=nil));
        assert(covered==c.stats.shift);
    }
    // Verification adapter: initial arc/fallback uses the dense oracle row step.
    // Once in section all bytes and links come solely from the constructor.
    Word marked(const Oracle& o,const Constructor& c,U mark) const{
        Word out;out.reserve(o.f.size());
        while(out.size()<o.f.size() && band_by_row[mark]==nil){
            out.push_back(o.f[mark]);mark=o.psi[mark];
        }
        while(out.size()<o.f.size()){
            U id=band_by_row[mark];assert(id!=nil);
            auto b=c.bands[id];auto& w=words[id];
            U take=std::min<size_t>(o.f.size()-out.size(),w.size());
            out.insert(out.end(),w.begin(),w.begin()+take);
            mark=b.dest+mark-b.lo;
        }
        return out;
    }
};

// Independent tiny model. Pick by global scan and read known words only here.
U model(const std::map<Word,U>& counts){
    struct C {U id,side,depth;std::vector<U> members;};
    std::vector<Word> words;std::vector<U> freq;
    for(auto& [w,n]:counts){words.push_back(w);freq.push_back(n);}
    std::vector<U> ids(words.size()), depth[2];
    std::iota(ids.begin(),ids.end(),0u);
    depth[0].resize(ids.size());depth[1].resize(ids.size());
    std::vector<C> active{{0,0,0,ids},{1,1,0,ids}};
    std::vector<bool> done(ids.size());
    U birth=2,states=0;
    for(;;){
        U best=nil;Key top{};
        for(U i=0;i<active.size();++i){
            auto& a=active[i];U m=0;
            for(U j:a.members)if(!done[j])m+=freq[j];
            Key k={m,a.depth,a.side,a.id,i};
            if(m && (best==nil || Priority{}(top,k))){best=i;top=k;}
        }
        if(best==nil)return states;
        ++states;
        C a=std::move(active[best]);active.erase(active.begin()+best);
        std::map<Byte,std::vector<U>> children;
        for(U j:a.members)if(!done[j]){
            U d=depth[a.side][j]++;
            if(depth[0][j]+depth[1][j]==words[j].size())done[j]=true;
            else children[words[j][a.side?words[j].size()-1-d:d]].push_back(j);
        }
        for(auto& [b,sub]:children)active.push_back({birth++,a.side,a.depth+1,std::move(sub)});
    }
}

Section section_mask(U n,U bits){
    Section s;
    for(U i=0;i<n;++i)if(bits>>i&1){
        if(s.empty() || s.back().second!=i)s.emplace_back(i,i);
        s.back().second=i+1;
    }
    return s;
}
void tiny(){
    N cases=0,marks=0,different=0;
    auto test=[&](const Oracle& oracle,const Edges& edges){
        U n=oracle.f.size();
        for(U bits=1;bits<(1u<<n);++bits){
            Section s=section_mask(n,bits);Constructor c(edges);c.run(s);
            Audit a(oracle,c,s);assert(model(a.counts)==c.stats.states);c.index_edges();
            for(U p=0;p<n;++p){auto expected=oracle.tape(p);auto actual=c.marked(p,n);
                auto indexed=c.indexed_marked(p,n);assert(std::equal(indexed.begin(),indexed.end(),expected.begin()));
                assert(a.marked(oracle,c,p)==expected && std::equal(actual.begin(),actual.end(),expected.begin()));++marks;}
            ++cases;different+=a.covered<n;
        }
    };
    for(U n=1;n<=5;++n){
        std::vector<U> perm(n);std::iota(perm.begin(),perm.end(),0u);
        do{
            for(U labels=0;labels<(1u<<n);++labels){
                Word f(n);for(U i=0;i<n;++i)f[i]=(labels>>i)&1;
                Oracle o(perm,f);test(o,o.arbitrary_edges());
            }
        }while(std::next_permutation(perm.begin(),perm.end()));
    }
    N arbitrary=cases;
    for(U n=1;n<=7;++n)for(U labels=0;labels<(1u<<n);++labels){
        std::vector<Run> runs;
        for(U i=0;i<n;++i){U b=(labels>>i)&1;
            if(runs.empty() || runs.back().byte!=b)runs.push_back({b,0});
            ++runs.back().len;
        }
        Oracle o(runs);test(o,build(runs));
    }
    assert(mem.live==0);
    std::cout<<"arbitrary_permutation_sections="<<arbitrary<<" stable_column_sections="<<cases-arbitrary
             <<" checked_marked_outputs="<<marks<<" sections_missing_at_least_one_cycle="<<different<<"\n";
}

#ifndef ASTRA_RANK_LIBRARY
int main(int argc,char** argv){
    if(argc==1){tiny();return 0;}
    assert(argc==4);
    std::ifstream in(argv[1]);U n,p,nruns,nsec;in>>n>>p>>nruns>>nsec;
    std::vector<Run> runs(nruns);
    for(auto& r:runs)in>>r.byte>>r.len;
    Oracle oracle(runs);assert(oracle.f.size()==n);
    std::ifstream gold(argv[2],std::ios::binary);
    Word expected((std::istreambuf_iterator<char>(gold)),{});
    assert(oracle.tape(p)==expected);
    for(U i=0;i<nsec;++i){
        std::string name;U k;in>>name>>k;Section s(k);for(auto& [a,b]:s)in>>a>>b;
        mem={};auto t=std::clock();auto e=build(runs);
        Constructor c(e);c.run(s);double cost=double(std::clock()-t)/CLOCKS_PER_SEC;
        N peak=mem.peak,allocs=mem.allocs;
        t=std::clock();auto marked=c.marked(p,n);double output_cost=double(std::clock()-t)/CLOCKS_PER_SEC;
        N output_peak=mem.peak;
        assert(std::equal(marked.begin(),marked.end(),expected.begin()));
        Audit a(oracle,c,s);assert(a.marked(oracle,c,p)==expected);
        U endpoints=0;for(auto [lo,hi]:s)endpoints+=hi-lo;
        auto z=c.stats;
        std::cout<<argv[3]<<','<<name<<','<<n<<','<<nruns<<','<<endpoints<<','<<a.counts.size()
          <<','<<z.states<<','<<z.pops<<','<<z.repush<<','<<z.shift
          <<','<<z.seeks<<','<<z.edge_cmp<<','<<z.pieces<<','<<z.probes<<','<<z.tree_cmp<<','<<z.join
          <<','<<z.gap<<','<<z.hit<<','<<z.op_splits<<','<<z.coalesces
          <<','<<z.fragment_inserts<<','<<z.fragment_erases<<','<<z.peak_map<<','<<z.peak_heap
          <<','<<c.groups.size()<<','<<c.bands.size()<<','<<peak<<','<<allocs<<','<<z.heap_cmp<<','<<z.sort_cmp<<','<<output_peak<<','<<cost<<','<<output_cost<<'\n';
    }
    assert(mem.live==0);
}
#endif
