#include "seeds.hpp"
#include <fstream>
#include <iostream>
#include <map>
#include <string>
U get32(std::istream& f){std::array<Byte,4>b;f.read((char*)b.data(),4);assert(f);return U(b[0])|(U(b[1])<<8)|(U(b[2])<<16)|(U(b[3])<<24);}
Geometry original(const char* path,U& primary){
    Geometry g;std::ifstream f(path);U b,w;assert(f>>primary);
    while(f>>b>>w){assert(b<256&&w);g.add(Byte(b),w);}g.finish();return g;
}
void verify(const Geometry& g,const Section& model,U k,const char* truth=nullptr){
    std::vector<Byte> last;std::vector<U> lf(g.n),physical(g.n);
    for(auto r:g.runs)for(U j=0;j<r.w;++j){last.push_back(r.b);lf[r.lo+j]=r.image+j;}
    std::vector<U> ends;std::vector<Byte> modes;
    if(truth){
        std::ifstream f(truth,std::ios::binary);char tag[8];f.read(tag,8);assert(std::string(tag,8)=="CTXMOD01"&&get32(f)==g.n);
        get32(f);assert(get32(f)==k);U C=get32(f);get32(f);assert(get32(f)==model.mass);get32(f);
        for(U i=0;i<C;++i){ends.push_back(get32(f));get32(f);modes.push_back(f.get());}
    }else{
        std::vector<Byte> F=last;std::sort(F.begin(),F.end());std::vector<U> psi(g.n);
        for(U i=0;i<g.n;++i)psi[lf[i]]=i;
        std::vector<Byte> old;
        for(U i=0;i<g.n;++i){std::vector<Byte> key;U x=i;
            for(U j=0;j<k;++j){key.push_back(F[x]);x=psi[x];}
            if(i&&key!=old)ends.push_back(i);
            old=std::move(key);
        }
        ends.push_back(g.n);
    }
    struct Ref {U lo,hi,owner;Byte b;};std::vector<Ref> rows;
    U left=0,mixed=0;
    for(U i=0;i<ends.size();++i){
        U end=ends[i];std::array<U,256> bins{};
        for(U x=left;x<end;++x)++bins[last[x]];
        Byte mode=std::max_element(bins.begin(),bins.end())-bins.begin();
        if(truth) assert(mode==modes[i]);
        if(bins[mode]!=end-left){
            assert(mixed<model.mixed.size());auto v=model.mixed[mixed];assert(v.lo==left&&v.hi==end&&v.mode==mode);
            std::map<Byte,Ref> a;
            for(U x=left;x<end;++x)if(last[x]!=mode){
                Byte b=last[x];auto [it,fresh]=a.try_emplace(b,Ref{lf[x],lf[x],mixed,b});
                assert(it->second.hi==lf[x]);++it->second.hi;(void)fresh;
            }
            for(const auto& kv:a)rows.push_back(kv.second);
            ++mixed;
        }
        left=end;
    }
    assert(left==g.n&&mixed==model.mixed.size()&&rows.size()==model.seeds.size());
    std::sort(rows.begin(),rows.end(),[](auto a,auto b){return a.lo<b.lo;});
    U mass=0;for(U i=0;i<rows.size();++i){auto a=rows[i];auto b=model.seeds[i];
        assert(a.lo==b.lo&&a.hi==b.hi&&a.owner==b.owner&&a.b==b.b&&b.base==mass);mass+=a.hi-a.lo;}
    assert(mass==model.mass);
}
int main(int argc,char** argv){
    if(argc==1){Q cases=0;
        for(U n=1,ways=3;n<=8;++n,ways*=3)for(U seq=0;seq<ways;++seq){
            U x=seq;Geometry g;for(U j=0;j<n;++j){g.add(x%3,1);x/=3;}g.finish();
            for(U k:{1u,2u,3u,8u,16u}){Stats t;auto s=classify<true>(g,k,t);verify(g,s,k);++cases;}
        }
        std::cout<<"all_ternary_models="<<cases<<'\n';return 0;
    }
    assert(argc==4);U p,k=std::stoul(argv[2]);auto g=original(argv[1],p);Stats t;auto s=classify<true>(g,k,t);verify(g,s,k,argv[3]);
    std::cout<<g.n<<','<<g.runs.size()<<','<<t.seams<<','<<t.tested<<','<<t.skipped<<','<<t.pair_steps<<','<<s.mixed.size()
             <<','<<s.seeds.size()<<','<<s.mass<<','<<t.fm_ranks<<','<<t.fm_cmps<<','<<t.hist_slices<<','<<t.image_queries<<','<<t.image_cmps;
    for(U j=0;j<k;++j){std::cout<<','<<t.reject[j];}std::cout<<'\n';
}
