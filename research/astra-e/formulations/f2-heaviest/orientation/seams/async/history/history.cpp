// The pinned control supplies the edge tables, mass priority, tracked allocator,
// marked writer, independent oracle and exhaustive model. Its live maps stay empty here.
#include "control.hpp"
#include <chrono>
#include <deque>
#include <stdexcept>

struct HistoryStats {
    N bitmap_reads=0, bitmap_writes=0, new_rows=0, interior_hits=0, root_hits=0;
    N descriptors=0, hist_seeks=0, desc_examined=0, history_pages=0;
    N lazy_ranges=0, lazy_skip_rows=0, zero_band_drops=0, reused_packets=0;
    N packet_pool_peak=0, queued_packets=0, peak_queued=0, root_tests=0;
    N owner_bytes=0, bitmap_bytes=0, page_heads_bytes=0;
};

class Bitmap {
    Vec<N> bits;
public:
    Bitmap()=default;
    void init(U n){bits.assign((n+63)/64,0);}
    N bytes() const{return bits.capacity()*sizeof(N);}
    std::pair<bool,U> span(U x,U end,HistoryStats& s) const {
        U page=x/64;
        N raw=bits[page];++s.bitmap_reads;
        bool on=(raw>>(x%64))&1;
        N changed=(on?~raw:raw)&(~N(0)<<(x%64));
        while(!changed && 64*(page+1)<end) {
            raw=bits[++page];++s.bitmap_reads;
            changed=on?~raw:raw;
        }
        return {on,changed?std::min<U>(end,64*page+std::countr_zero(changed)):end};
    }
    void set(U x,U end,HistoryStats& s){
        while(x<end) {
            U stop=std::min(end,(x/64+1)*64);
            N mask=(~N(0)<<(x%64)) & (~N(0)>>(63-(stop-1)%64));
            assert((bits[x/64]&mask)==0);
            bits[x/64]|=mask;++s.bitmap_reads;++s.bitmap_writes;
            x=stop;
        }
    }
};

struct Record { U owner, group, previous; uint16_t bounds; };
static_assert(sizeof(Record)==16);
struct Known {U lo,hi,owner,g;};

template<U Page>
class History : public Constructor {
    static_assert(Page>=1 && Page<=256 && std::has_single_bit(Page));
    const Edges& e;
    const Section& roots;
    Vec<U> root_offset,page_head;
    Vec<Record> archive;
    Bitmap seen, collisions;
    std::array<Bitmap,2> root_dead;
    Vec<Key> queue;
    std::deque<Seg,Paid<Seg>> packet_pool;
    Seg* free_packet=nullptr;
    U birth=0;

    U component(U row) {
        ++hs.root_tests;
        if(roots.size()==1) return row<roots[0].second?0:1;
        return std::lower_bound(roots.begin(),roots.end(),row,
             [](auto p,U y){return p.second<=y;})-roots.begin();
    }
    U add_group(U side,U depth,U parent,Byte c) {
        U id=groups.size();
        groups.push_back({nullptr,0,depth,nil,parent,Byte(side),c});return id;
    }
    void enqueue(U id) {
        auto g=groups[id];
        queue.push_back({g.mass,g.depth,g.side,g.birth,id});
        std::push_heap(queue.begin(),queue.end(),Priority{&stats.heap_cmp});
        stats.peak_heap=std::max<N>(stats.peak_heap,queue.size());
    }
    void attach(U id,U lo,U hi,U owner) {
        Seg* p;
        if (free_packet) {p=free_packet;free_packet=p->next;++hs.reused_packets;}
        else {packet_pool.push_back({});p=&packet_pool.back();}
        auto& g=groups[id];*p={lo,hi,owner,id,nullptr,g.head};g.head=p;
        ++stats.fragment_inserts;++hs.queued_packets;
        hs.peak_queued=std::max(hs.peak_queued,hs.queued_packets);
        hs.packet_pool_peak=std::max<N>(hs.packet_pool_peak,packet_pool.size());
    }
    Seg take(U id) {
        auto& g=groups[id];auto* p=g.head;auto v=*p;
        g.head=v.next;p->next=free_packet;free_packet=p;
        --hs.queued_packets;++stats.fragment_erases;
        return v;
    }
    void remember(U lo,U hi,U owner,U group) {
        while(lo<hi) {
            U page=lo/Page, last=std::min(hi,(page+1)*Page);
            uint16_t bounds=uint16_t((lo%Page) | ((last-lo-1)<<8));
            U index=archive.size();
            archive.push_back({owner,group,page_head[page],bounds});
            page_head[page]=index;
            ++hs.descriptors;owner+=last-lo;lo=last;
        }
    }
    Known lookup(U y) {
        ++hs.hist_seeks;
        U page=y/Page;
        for(U id=page_head[page];id!=nil;id=archive[id].previous) {
            ++hs.desc_examined;
            const auto r=archive[id];
            U lo=page*Page+(r.bounds&255),hi=lo+(r.bounds>>8)+1;
            if(lo<=y && y<hi) return {lo,hi,r.owner,r.group};
        }
        throw std::runtime_error("missing first-arrival descriptor");
    }
    void meet(U side,U id,U own,U other_id,U other,U width,Byte c) {
        assert(groups[other_id].side!=side && groups[other_id].mass>=width);
        groups[other_id].mass-=width;
        bands.push_back(side==0?Band{own,own+width,other,id,other_id,c}:
                                Band{other,other+width,own,other_id,id,c});
        ++stats.hit;
    }
    void advance_piece(U id,U x,U end,U original,std::array<U,256>& child) {
        auto side=groups[id].side;
        auto& edges=e[side];++stats.seeks;
        auto k=std::prev(std::upper_bound(edges.begin(),edges.end(),x,[&](U y,Piece p){
            ++stats.edge_cmp;return y<p.lo;
        }));
        while(x<end) {
            U last=std::min(end,k->hi),y=k->dst+x-k->lo,stop=y+last-x;
            Byte c=k->c;
            ++stats.pieces;
            U owner=original;
            while(y<stop) {
                U comp=component(y);
                if(comp<roots.size() && roots[comp].first<=y) {
                    U hi=std::min(stop,roots[comp].second), offset=root_offset[comp]+y-roots[comp].first;
                    U w=hi-y;
                    root_dead[1-side].set(offset,offset+w,hs);
                    meet(side,id,owner,1-side,y,w,c);
                    hs.root_hits+=w;owner+=w;y=hi;
                    continue;
                }
                U boundary=comp<roots.size()?std::min(stop,roots[comp].first):stop;
                auto [visited,hi]=seen.span(y,boundary,hs);
                assert(y<hi);
                if(!visited) {
                    if(child[c]==nil) child[c]=add_group(side,groups[id].depth+1,id,c);
                    U g=child[c],w=hi-y;
                    seen.set(y,hi,hs);remember(y,hi,owner,g);attach(g,y,hi,owner);
                    groups[g].mass+=w;hs.new_rows+=w;++stats.gap;owner+=w;y=hi;
                } else {
                    while(y<hi) {
                        Known r=lookup(y);U last_hit=std::min(hi,r.hi),w=last_hit-y;
                        meet(side,id,owner,r.g,r.owner+y-r.lo,w,c);
                        collisions.set(y,last_hit,hs);hs.interior_hits+=w;
                        y=last_hit;owner+=w;
                    }
                }
            }
            original+=last-x;x=last;++k;
        }
    }
    void move(U id) {
        std::array<U,256> child;child.fill(nil);
        U consumed=0;const U side=groups[id].side;
        while(groups[id].head) {
            Seg a=take(id);U x=a.lo;
            U base=0,off=0;
            if(id<2) {
                U ci=component(x);assert(ci<roots.size() && a.hi<=roots[ci].second);
                base=roots[ci].first;off=root_offset[ci];
            }
            while(x<a.hi) {
                auto [dead,lim]=id<2?root_dead[side].span(off+x-base,off+a.hi-base,hs):
                                     collisions.span(x,a.hi,hs);
                U hi=id<2?(lim-off+base):lim;
                if(dead) {hs.lazy_skip_rows+=hi-x;++hs.lazy_ranges;}
                else {advance_piece(id,x,hi,a.owner+x-a.lo,child);consumed+=hi-x;}
                x=hi;
            }
        }
        assert(consumed==groups[id].mass);
        stats.shift+=consumed;groups[id].mass=0;
        for(U g:child) if(g!=nil) {groups[g].birth=birth++;enqueue(g);}
    }
public:
    HistoryStats hs;
    History(const Edges& edges,const Section& s):Constructor(edges),e(edges),roots(s) {}
    void run() {
        U n=e[0].back().hi,m=0;
        seen.init(n);collisions.init(n);
        for(auto [lo,hi]:roots){root_offset.push_back(m);m+=hi-lo;}
        for(auto& b:root_dead)b.init(m);
        page_head.assign((n+Page-1)/Page,nil);
        for(U side=0;side<2;++side){
            U id=add_group(side,0,nil,0);groups[id].birth=birth++;
            for(auto [lo,hi]:roots){attach(id,lo,hi,lo);groups[id].mass+=hi-lo;}
            enqueue(id);
        }
        while(!queue.empty()){
            std::pop_heap(queue.begin(),queue.end(),Priority{&stats.heap_cmp});
            auto k=queue.back();queue.pop_back();++stats.pops;
            U g=k.g;
            if(groups[g].mass==0) {
                while(groups[g].head){take(g);++hs.zero_band_drops;}
                continue;
            }
            if(groups[g].mass!=k.mass){++stats.repush;enqueue(g);continue;}
            ++stats.states;move(g);
        }
        assert(hs.queued_packets==0 && hs.new_rows+m==stats.shift);
        assert(hs.interior_hits+hs.root_hits==m);
        std::sort(bands.begin(),bands.end(),[&](const Band& a,const Band& b){
            ++stats.sort_cmp;return a.lo<b.lo;
        });
        hs.bitmap_bytes=seen.bytes()+collisions.bytes()+root_dead[0].bytes()+root_dead[1].bytes();
        hs.owner_bytes=archive.capacity()*sizeof(Record);
        hs.page_heads_bytes=page_head.capacity()*sizeof(U);
        hs.history_pages=page_head.size();
    }
};

template<class C>
std::vector<std::array<U,4>> seams(const C& c,U n) {
    std::vector<std::array<U,4>> a(n,{nil,nil,nil,nil});
    for(auto b:c.bands)for(U x=b.lo;x<b.hi;++x)
        a[x]={b.dest+x-b.lo,c.groups[b.left].depth,c.groups[b.right].depth,b.c};
    return a;
}

template<U Page>
void check(const Edges& e,const Oracle& o,const Section& s) {
    Constructor base(e);base.run(s);History<Page> h(e,s);h.run();
    assert(seams(base,o.f.size())==seams(h,o.f.size()));
    assert(h.stats.states==base.stats.states && h.stats.shift==base.stats.shift);
    Audit audit(o,h,s);
    assert(model(audit.counts)==h.stats.states);
    for(U mark=0;mark<o.f.size();++mark) {
        auto out=h.marked(mark,o.f.size());auto gold=o.tape(mark);
        assert(std::equal(out.begin(),out.end(),gold.begin(),gold.end()));
    }
}

void exhaustive(){
    U cases=0;N marks=0;
    auto each=[&](const Oracle& o,const Edges& e) {
        U n=o.f.size();
        for(U bits=1;bits<(1u<<n);++bits){
            auto s=section_mask(n,bits);check<16>(e,o,s);++cases;marks+=n;
        }
    };
    for(U n=1;n<=5;++n) {
        std::vector<U> f(n);std::iota(f.begin(),f.end(),0u);
        do {
            for(U mask=0;mask<(1u<<n);++mask) {
                Word bytes(n);for(U i=0;i<n;++i)bytes[i]=(mask>>i)&1;
                Oracle o(f,bytes);each(o,o.arbitrary_edges());
            }
        }while(std::next_permutation(f.begin(),f.end()));
    }
    for(U n=1;n<=7;++n)for(U bits=0;bits<(1u<<n);++bits) {
        std::vector<Run> runs;
        for(U i=0;i<n;++i){U c=(bits>>i)&1;if(runs.empty()||runs.back().byte!=c)runs.push_back({c,0});++runs.back().len;}
        Oracle o(runs);each(o,build(runs));
    }
    assert(mem.live==0);
    std::cout<<"sections "<<cases<<" marked_outputs "<<marks<<" matched exact seam owners/depths/byte against 8f1be9d and output against independent dense cycles\n";
}

template<U Page>
void report(const std::vector<Run>& runs,const Section& section,const Word& gold,U mark,
            const std::string& ordinal,const std::string& name) {
    assert(mem.live==0);mem={};
    auto t=std::clock();auto e=build(runs);History<Page> c(e,section);c.run();
    double construction=double(std::clock()-t)/CLOCKS_PER_SEC;N peak=mem.peak;
    auto time=std::clock();auto out=c.marked(mark,gold.size());double output=double(std::clock()-time)/CLOCKS_PER_SEC;
    assert(std::equal(out.begin(),out.end(),gold.begin(),gold.end()));
    auto z=c.stats;auto h=c.hs;
    std::cout<<ordinal<<','<<name<<','<<Page<<','<<z.states<<','<<z.pops<<','<<z.repush<<','<<z.shift<<','<<z.seeks
      <<','<<z.pieces<<','<<c.groups.size()<<','<<c.bands.size()<<','<<h.new_rows<<','<<h.interior_hits<<','<<h.root_hits
      <<','<<h.descriptors<<','<<h.hist_seeks<<','<<h.desc_examined<<','<<h.bitmap_reads<<','<<h.bitmap_writes
      <<','<<h.root_tests<<','<<h.lazy_ranges<<','<<h.lazy_skip_rows<<','<<h.zero_band_drops
      <<','<<h.packet_pool_peak<<','<<h.reused_packets<<','<<h.owner_bytes<<','<<h.page_heads_bytes<<','<<h.bitmap_bytes
      <<','<<peak<<','<<mem.peak<<','<<construction<<','<<output<<'\n';
    {
        // The control and dense audit are outside both measured regions.
        Constructor baseline(e);baseline.run(section);
        assert(seams(baseline,gold.size())==seams(c,gold.size()));
        assert(baseline.stats.pops==z.pops && baseline.stats.repush==z.repush && baseline.stats.states==z.states);
        Oracle o(runs);Audit verify(o,c,section);
        assert(verify.marked(o,c,mark)==gold);
    }
}

int main(int argc,char** argv) {
    if(argc==1) {exhaustive();return 0;}
    assert(argc==4);
    std::ifstream in(argv[1]);U n,mark,r,s;in>>n>>mark>>r>>s;
    std::vector<Run> runs(r);for(auto& v:runs)in>>v.byte>>v.len;
    std::ifstream gf(argv[2],std::ios::binary);Word gold((std::istreambuf_iterator<char>(gf)),{});
    assert(gold.size()==n);
    while(s--){
        std::string name;U k;in>>name>>k;Section sec(k);
        for(auto& v:sec)in>>v.first>>v.second;
        report<16>(runs,sec,gold,mark,argv[3],name);
        report<64>(runs,sec,gold,mark,argv[3],name);
        report<256>(runs,sec,gold,mark,argv[3],name);
    }
}
