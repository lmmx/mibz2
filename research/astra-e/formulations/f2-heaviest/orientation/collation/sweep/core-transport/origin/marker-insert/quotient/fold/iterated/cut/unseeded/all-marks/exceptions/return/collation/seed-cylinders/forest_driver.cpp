#include "forest.hpp"
#define main seed_driver_main
#include "small-geometry.hpp"
#undef main
#include <set>
struct Inverse {std::vector<U> psi,owner;std::vector<Byte> bytes;};
Inverse expand(const Section& s,const Child& child,U n){
    Inverse a;a.psi.resize(s.mass);a.owner.resize(s.mass);std::vector<U> ptr;
    for(U i=0;i<child.classes.size();++i){auto v=child.classes[i];ptr.push_back(v.base);
        std::fill_n(a.owner.begin()+v.base,v.width,i);}
    U y=0;for(auto r:child.incoming){std::iota(a.psi.begin()+ptr[r.id],a.psi.begin()+ptr[r.id]+r.w,y);ptr[r.id]+=r.w;y+=r.w;}
    assert(y==s.mass);a.bytes.reserve(n);
    if(child.origin==none){assert(!child.pure.empty());for(U i=0;i<n;++i)a.bytes.push_back(child.pure[i%child.pure.size()]);}
    else {
        U x=child.origin,lo=child.skip;
        while(a.bytes.size()<n){
            auto c=child.classes[a.owner[x]];assert(lo<=c.weight);
            U w=std::min(U(n-a.bytes.size()),c.weight-lo);
            a.bytes.insert(a.bytes.end(),child.dictionary.begin()+c.offset+lo,child.dictionary.begin()+c.offset+lo+w);
            x=a.psi[x];lo=0;
        }
    }
    return a;
}
void tiny(const Geometry& g,U k,U primary){
    Stats t;auto s=classify<true>(g,k,t);auto c=form<true>(g,s,primary,t);auto got=expand(s,c,g.n);
    std::vector<U> rank(g.n,none),seed(g.n,none),row,psi(g.n);std::vector<Byte> f(g.n);
    for(U j=0;j<s.seeds.size();++j)for(U x=s.seeds[j].lo;x<s.seeds[j].hi;++x){rank[x]=row.size();seed[x]=j;row.push_back(x);}
    for(auto r:g.runs)for(U h=0;h<r.w;++h){f[r.image+h]=r.b;psi[r.image+h]=r.lo+h;}
    U x=primary;for(auto b:got.bytes){assert(b==f[x]);x=psi[x];}
    std::map<std::pair<U,std::vector<Byte>>,U> used;
    for(U i=0;i<row.size();++i){std::vector<Byte> w;U z=row[i];
        do{w.push_back(f[z]);z=psi[z];}while(rank[z]==none);
        U id=got.owner[i];auto cl=c.classes[id];
        assert(cl.weight==w.size()&&std::equal(w.begin(),w.end(),c.dictionary.begin()+cl.offset));
        assert(got.psi[i]==rank[z]);
        auto [it,inserted]=used.try_emplace(std::make_pair(seed[z],w),id);assert(inserted||it->second==id);
    }
    assert(used.size()==c.classes.size());
}
int main(int argc,char** argv){
    if(argc==1){Q models=0,marks=0;
        for(U n=1,ways=3;n<=7;++n,ways*=3)for(U seq=0;seq<ways;++seq){
            U x=seq;Geometry g;for(U j=0;j<n;++j){g.add(x%3,1);x/=3;}g.finish();
            for(U k:{1u,2u,3u,8u}){++models;for(U p=0;p<n;++p){tiny(g,k,p);++marks;}}
        }
        std::cout<<"models="<<models<<" marks="<<marks<<'\n';return 0;
    }
    assert(argc==6);U p,k=std::stoul(argv[2]);auto g=original(argv[1],p);Stats t;
    auto s=classify<true>(g,k,t);auto seed_img=t.image_queries;
    auto child=form<true>(g,s,p,t);auto out=expand(s,child,g.n);
    std::ifstream model(argv[3],std::ios::binary),truth(argv[4],std::ios::binary),want(argv[5],std::ios::binary);
    char tag[8];model.read(tag,8);assert(std::string(tag,8)=="CTXMOD01"&&get32(model)==g.n);
    get32(model);assert(get32(model)==k);U C=get32(model);get32(model);assert(get32(model)==s.mass);get32(model);
    std::vector<U> def;for(U i=0;i<C;++i){get32(model);U post=get32(model);Byte b=model.get();def.push_back((post<<8)|b);}
    truth.read(tag,8);assert(std::string(tag,8)=="EXCCOL01");assert(get32(truth)==child.classes.size());assert(get32(truth)==child.incoming.size());
    U oldmark=get32(truth),oldskip=get32(truth);assert(get32(truth)==child.default_lap);
    bool identical_phase=oldmark==child.origin&&oldskip==child.skip;
    if(!identical_phase){assert(!child.skip&&oldmark!=none&&oldskip==child.classes[out.owner[oldmark]].weight&&out.psi[oldmark]==child.origin);}
    for(U i=0;i<child.classes.size();++i){
        U base=get32(truth),width=get32(truth),weight=get32(truth);get32(truth);Byte first=truth.get();truth.get();U state=get32(truth);
        auto cl=child.classes[out.owner[base]];assert(cl.base==base&&cl.width==width&&cl.weight==weight&&child.dictionary[cl.offset]==first);
        for(U d=weight;d>1;){U e=def[state];assert(child.dictionary[cl.offset+--d]==Byte(e));state=e>>8;}
    }
    for(auto r:child.incoming){assert(get32(truth)==child.classes[r.id].base);assert(get32(truth)==r.w);}
    assert(truth.peek()==EOF);std::vector<Byte> expected(g.n);want.read((char*)expected.data(),g.n);assert(want&&want.peek()==EOF&&expected==out.bytes);
    std::cout<<g.n<<','<<s.mass<<','<<s.seeds.size()<<','<<t.nodes<<','<<t.unary<<','<<t.branch<<','<<t.leaves<<','<<t.covered_rows
             <<','<<t.slices<<','<<t.seed_queries<<','<<t.seed_cmps<<','<<t.spelling<<','<<t.merge_claims<<','<<t.merge_records
             <<','<<t.peak_live<<','<<child.incoming.size()<<','<<t.phase_steps<<','<<t.fallback<<','<<identical_phase<<','<<t.image_queries-seed_img<<'\n';
}
