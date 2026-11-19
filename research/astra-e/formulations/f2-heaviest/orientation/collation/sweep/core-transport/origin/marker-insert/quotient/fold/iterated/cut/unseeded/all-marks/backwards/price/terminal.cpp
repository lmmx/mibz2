#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <ctime>
#include <filesystem>
#include <fstream>
#include <map>
#include <span>
#include <vector>
#include "ids.inc"
using U=uint32_t;
using Q=uint64_t;
Q cpu(){timespec t;int ret=clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t);assert(!ret);return Q(t.tv_sec)*1000000000+t.tv_nsec;}
std::filesystem::path wire_path(unsigned id){
    auto dir=std::getenv("POSONLY_WIRES");assert(dir);
    return std::filesystem::path(dir)/(std::to_string(id)+".pos");
}
std::vector<uint8_t> read_file(const std::filesystem::path& p){
    std::ifstream f(p,std::ios::binary|std::ios::ate);assert(f);
    std::vector<uint8_t> b(f.tellg());f.seekg(0);f.read(reinterpret_cast<char*>(b.data()),b.size());assert(f);return b;
}
struct Reader{
    std::span<const uint8_t> wire;size_t at=0;
    U word(){assert(at+4<=wire.size());auto p=wire.data()+at;at+=4;
        return U(p[0])|(U(p[1])<<8)|(U(p[2])<<16)|(U(p[3])<<24);}
    uint8_t byte(){assert(at<wire.size());return wire[at++];}
};
struct Piece{U x;int32_t delta;U add;};
static_assert(sizeof(Piece)==12);
struct Level{U old,next;std::vector<Piece> rec;};
struct Tree{
    U n=0,mark=0;std::vector<U> boundaries;std::vector<uint8_t> letters;std::vector<Level> levels;
    Tree()=default;
    explicit Tree(std::span<const uint8_t> wire){
        Reader r{wire,8};assert(wire.size()>=24&&!std::memcmp(wire.data(),"POSONLY1",8));
        n=r.word();mark=r.word();U k=r.word(),h=r.word();assert(n&&mark<n&&k<=256);
        boundaries.reserve(k+1);letters.reserve(k);levels.reserve(h);boundaries.push_back(0);
        for(U i=0;i<k;++i){boundaries.push_back(boundaries.back()+r.word());letters.push_back(r.byte());}
        assert(boundaries.back()==n);
        for(U i=0;i<h;++i){
            Level z{r.word(),r.word(),{}};U p=r.word();z.rec.reserve(p+1);
            for(U j=0;j<p;++j){U x=r.word();int32_t d=int32_t(r.word());U a=r.word();z.rec.push_back({x,d,a});}
            assert(p&&z.rec[0].x==0&&z.old==(i?levels.back().next:n));z.rec.push_back({z.old,0,0});
            levels.push_back(std::move(z));
        }
        assert(r.at==wire.size()&&(levels.empty()?n==1:levels.back().next==1));
    }
    size_t heap()const{
        size_t total=boundaries.capacity()*sizeof(U)+letters.capacity()+levels.capacity()*sizeof(Level);
        for(auto& z:levels)total+=z.rec.capacity()*sizeof(Piece);
        return total;
    }
};
struct Stamp{U id=0,arm=0;Q parse=0,provision=0,reverse=0,scatter=0,repeat=0,once=0;size_t tree=0,offset=0,output=0,wire=0;};
struct Journal{std::vector<Stamp> rows;
    ~Journal(){if(const char* p=std::getenv("POSONLY_STAMPS")){std::ofstream f(p);
        f<<"block,arm,parse_ns,provision_ns,reverse_ns,scatter_ns,rehearsal_ns,free_tree_load_ns,tree_capacity,offset_capacity,output_capacity,resident_wire_bytes\n";
        for(auto& q:rows)f<<q.id<<','<<q.arm<<','<<q.parse<<','<<q.provision<<','<<q.reverse<<','<<q.scatter<<','<<q.repeat<<','<<q.once<<','<<q.tree<<','<<q.offset<<','<<q.output<<','<<q.wire<<'\n';
    }}
};
static Journal journal;
struct Scratch{std::vector<U> offset;std::vector<uint8_t> output;};
void reverse_scatter(const Tree& t,Scratch& s,Stamp& q){
    Q start=cpu();auto& a=s.offset;auto& out=s.output;
    a.resize(t.levels.empty()?1:t.levels[0].next);out.resize(t.n);a[0]=0;
    Q ready=cpu();
    // Both assignments below use the affine invariants of the frozen map.
    // Only the terminal coordinate a[0] is an input; reused slots are fully set.
    for(size_t z=t.levels.size();z-->1;){
        const auto& pieces=t.levels[z].rec;
        for(size_t i=pieces.size()-1;i-->0;){
            auto [x,shift,add]=pieces[i];if(add||!shift)continue;
            auto w=pieces[i+1].x-x;
            std::memmove(a.data()+x,a.data()+int64_t(x)+shift,w*sizeof(U));
        }
        for(size_t i=0;i+1<pieces.size();++i){
            auto [x,shift,add]=pieces[i];if(!add)continue;
            U end=pieces[i+1].x;
            for(U j=x;j<end;++j)a[j]=a[int64_t(j)+shift]+add;
        }
    }
    Q expanded=cpu();
    if(t.levels.empty())out[0]=t.letters[0];
    else{
        const auto& r=t.levels[0].rec;
        auto it=std::upper_bound(r.begin(),r.end(),t.mark,[](U x,Piece p){return x<p.x;});--it;
        U origin=a[int64_t(t.mark)+it->delta]+it->add;U owner=0;
        for(size_t i=0;i+1<r.size();++i){
            auto [x,shift,add]=r[i];U end=r[i+1].x;
            while(x<end){
                while(x>=t.boundaries[owner+1])++owner;
                U stop=std::min(end,t.boundaries[owner+1]);uint8_t val=t.letters[owner];
                for(;x<stop;++x){U pos=a[int64_t(x)+shift]+add;
                    out[pos>=origin?pos-origin:t.n-origin+pos]=val;}
            }
        }
    }
    Q finish=cpu();q.provision=ready-start;q.reverse=expanded-ready;q.scatter=finish-expanded;
    q.offset=a.capacity()*sizeof(U);q.output=out.capacity();
}
struct Entry{Tree tree;std::vector<uint8_t> wire,expected;};
struct State{std::map<U,Entry> entries;Scratch scratch;std::vector<uint8_t> fresh_out;};
extern "C" State* grammar_new(){return new State;}
extern "C" void grammar_delete(State* s){delete s;}
extern "C" const uint8_t* grammar_run(State* s,const uint16_t* caps,U records,U n,U mark,U arm){
    assert(caps&&arm<3);auto id=lookup(n,mark,records);Stamp q{};q.id=id;q.arm=arm;
    auto [f,new_entry]=s->entries.try_emplace(id);Entry& e=f->second;
    if(new_entry){Q start=cpu();auto data=read_file(wire_path(id));
        if(arm==2)e.wire=std::move(data);else e.tree=Tree(data);
        if(arm==1){
            const char* r=std::getenv("POSONLY_REFS");assert(r);
            e.expected=read_file(std::filesystem::path(r)/("block-"+std::to_string(id)+".raw"));
        }
        q.once=cpu()-start;
    }
    const uint8_t* out;
    if(arm==2){
        Q start=cpu();Tree tree(e.wire);q.parse=cpu()-start;q.tree=tree.heap();q.wire=e.wire.capacity();
        assert(tree.n==n&&tree.mark==mark);Scratch tmp;reverse_scatter(tree,tmp,q);
        s->fresh_out=std::move(tmp.output);out=s->fresh_out.data();
    }else{
        auto& t=e.tree;assert(t.n==n&&t.mark==mark);q.tree=t.heap();
        if(arm==1){Q start=cpu();Stamp h;reverse_scatter(t,s->scratch,h);
            assert(e.expected==s->scratch.output);
            q.repeat=cpu()-start;
        }
        reverse_scatter(t,s->scratch,q);out=s->scratch.output.data();
    }
    if(std::getenv("POSONLY_STAMPS"))journal.rows.push_back(q);
    return out;
}
