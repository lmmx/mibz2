#include <algorithm>
#include <array>
#include <cassert>
#include <chrono>
#include <cstdint>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <numeric>
#include <string>
#include <time.h>
#include <vector>
using U=uint32_t;using Q=uint64_t;
constexpr U invalid=UINT32_MAX,term=1u<<31,mask20=(1u<<20)-1;
struct Pair{U x,y;bool operator==(const Pair&)const=default;};
struct Range{U lo,hi,base;};
struct Run{U x,w,y,c;};
struct Packet{U orig,x,w,t,i;};
struct Input{U n,primary;std::vector<uint16_t> caps;std::vector<Range> exc;};
template<class T>Q bytes(const std::vector<T>& a){return a.capacity()*sizeof(T);}
template<class T>void get(std::istream& f,T& v){f.read(reinterpret_cast<char*>(&v),sizeof(v));if(!f)std::abort();}
template<class T>void read_vec(std::istream& f,std::vector<T>& a){U n;get(f,n);a.resize(n);f.read(reinterpret_cast<char*>(a.data()),a.size()*sizeof(T));if(!f)std::abort();}
Input load(const std::string& dir,U block,U k){
    Input a{};std::ifstream f(dir+"/"+std::to_string(block)+"."+std::to_string(k),std::ios::binary);
    get(f,a.n);get(f,a.primary);read_vec(f,a.caps);read_vec(f,a.exc);
    if(f.peek()!=EOF||a.n>mask20)std::abort();
    return a;
}
struct Output{
    U lap;std::vector<Pair> events;
    bool operator==(const Output&)const=default;
    void init(const Input& in){events.clear();lap=0;if(!in.exc.empty())events.reserve(in.exc.back().base+in.exc.back().hi-in.exc.back().lo);}
};
struct Direct{
    std::vector<U> lf,base;Output out{};
    void build(const Input& a){
        std::array<U,256> hist{},cur{};
        for(U rec:a.caps)hist[rec>>8]+=rec&255;
        std::partial_sum(hist.begin(),hist.end()-1,cur.begin()+1);
        lf.resize(a.n);U x=0;
        for(U rec:a.caps){U len=rec&255;auto& y=cur[rec>>8];std::iota(lf.begin()+x,lf.begin()+x+len,y);y+=len;x+=len;}
        base.resize((a.n+255)/256);
        U next=0;
        for(auto [lo,hi,first]:a.exc){
            while(next<base.size()&&next*256<=lo)base[next++]=first;
            for(U i=lo;i<hi;i++){
                U ordinal=first+i-lo;
                if(i==next*256)base[next++]=ordinal;
                lf[i]|=(1+ordinal-base[i>>8])<<20;
            }
        }
        out.init(a);U i=a.primary,t=0;
        do{U e=lf[i],local=e>>20;if(local)out.events.push_back({t,base[i>>8]+local-1});
            i=e&mask20;++t;
        }while(i!=a.primary);
        out.lap=t;
    }
    Q capacity()const{return bytes(lf)+bytes(base)+bytes(out.events);}
};
struct Induced{
    std::vector<Run> runs;std::vector<Range> roots;std::vector<Pair> atoms,returns;
    std::vector<U> order,starts,page;std::vector<Packet> stack;Output out{};
    U mark=invalid;bool is_event=false;
    void rootset(const Input& a){
        roots.clear();roots.reserve(a.exc.size()+1);U base=0;
        auto add=[&](U lo,U hi){
            if(!roots.empty()&&roots.back().hi>=lo){U more=std::max(hi,roots.back().hi)-roots.back().hi;
                roots.back().hi+=more;base+=more;
            }else{roots.push_back({lo,hi,base});base+=hi-lo;}
        };
        bool inserted=false;is_event=false;
        for(auto r:a.exc){if(!inserted&&a.primary<=r.lo){add(a.primary,a.primary+1);inserted=true;}
            add(r.lo,r.hi);
            if(r.lo<=a.primary&&a.primary<r.hi){inserted=true;is_event=true;}
        }
        if(!inserted)add(a.primary,a.primary+1);
        auto it=std::upper_bound(roots.begin(),roots.end(),a.primary,[](U p,Range r){return p<r.lo;})-1;
        mark=it->base+a.primary-it->lo;
        returns.resize(base);
    }
    void map(const Input& a){
        std::array<U,256> hist{},cur{},count{},slot{};
        runs.clear();runs.reserve(a.caps.size());U x=0;
        for(U rec:a.caps){
            U c=rec>>8,w=rec&255;hist[c]+=w;
            if(!runs.empty()&&runs.back().c==c)runs.back().w+=w;
            else {runs.push_back({x,w,0,c});++count[c];}
            x+=w;
        }
        std::partial_sum(hist.begin(),hist.end()-1,cur.begin()+1);
        std::partial_sum(count.begin(),count.end()-1,slot.begin()+1);
        order.resize(runs.size());starts.assign(1+runs.size(),0);
        for(U j=0;j<runs.size();j++){auto& r=runs[j];r.y=cur[r.c];cur[r.c]+=r.w;order[slot[r.c]++]=j;}
        auto overlay=[&](auto f){
            U k=0;
            for(U j:order){
                auto r=runs[j];U end=r.y+r.w;
                for(U y=r.y;y<end;){
                    while(k<roots.size()&&roots[k].hi<=y)++k;
                    bool hit=k<roots.size()&&roots[k].lo<=y;
                    U hi=k==roots.size()?end:std::min(end,hit?roots[k].hi:roots[k].lo);
                    f(j,r.x+y-r.y,hit?(term|(roots[k].base+y-roots[k].lo)):y);y=hi;
                }
            }
        };
        overlay([&](U j,U,U){++starts[j+1];});
        std::partial_sum(starts.begin(),starts.end(),starts.begin());atoms.resize(starts.back()+1);
        overlay([&](U j,U pos,U y){atoms[starts[j]++]={pos,y};});
        atoms.back()={a.n,0};page.resize((a.n+255)/256+1);
        U j=0;for(U pg=0;pg<page.size();++pg){
            U pos=std::min(pg*256,a.n-1);while(atoms[j+1].x<=pos)++j;page[pg]=j;
        }
    }
    U locate(U pos)const{
        U i=page[pos>>8],end=page[(pos>>8)+1]+1;
        while(i+1<end){U mid=(i+end)/2;if(atoms[mid].x<=pos)i=mid;else end=mid;}
        return i;
    }
    void build(const Input& a){
        rootset(a);map(a);stack.clear();
        for(auto root:roots){
            stack.push_back({root.base,root.lo,root.hi-root.lo,0,invalid});
            while(!stack.empty()){
                Packet p=stack.back();stack.pop_back();
                for(;;){
                    U j=p.i==invalid?locate(p.x):p.i,w=std::min(p.w,atoms[j+1].x-p.x);
                    if(w<p.w)stack.push_back({p.orig+w,p.x+w,p.w-w,p.t,j+1});
                    U y=atoms[j].y+p.x-atoms[j].x;++p.t;
                    if(y&term){
                        y^=term;for(U k=0;k<w;k++)returns[p.orig+k]={y+k,p.t};break;
                    }
                    p.x=y;p.w=w;p.i=invalid;
                }
            }
        }
        out.init(a);U x=mark,t=0;
        do{
            if(is_event||x!=mark)out.events.push_back({t,x-U(!is_event&&x>mark)});
            auto e=returns[x];t+=e.y;x=e.x;
        }while(x!=mark);
        out.lap=t;
    }
    Q capacity()const{return bytes(runs)+bytes(roots)+bytes(atoms)+bytes(returns)+bytes(order)+bytes(starts)+bytes(page)+bytes(stack)+bytes(out.events);}
};
Q cpu(){timespec t;if(clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t))std::abort();return Q(t.tv_sec)*1000000000+t.tv_nsec;}
U hwm(){std::ifstream st("/proc/self/status");std::string field;while(st>>field){if(field=="VmHWM:"){U k;st>>k;return k;}std::getline(st,field);}std::abort();}
void oracle(const Input& a,const Output& out,const std::string& pred){
    std::vector<U> rows;for(auto [lo,hi,base]:a.exc){assert(base==rows.size());for(U i=lo;i<hi;i++)rows.push_back(i);}
    std::vector<uint8_t> last;for(U r:a.caps)last.insert(last.end(),r&255,r>>8);
    std::array<std::vector<U>,256> ranks;
    auto sorted=last;std::sort(sorted.begin(),sorted.end());for(U i=0;i<sorted.size();i++)ranks[sorted[i]].push_back(i);
    std::vector<U> lf;std::array<U,256> used{};
    for(auto b:last)lf.push_back(ranks[b][used[b]++]);
    std::ifstream f(pred,std::ios::binary);char magic[8];f.read(magic,8);U n,state,c,e;get(f,n);get(f,state);get(f,c);get(f,e);f.ignore(5ull*c);
    assert(n==a.n&&e==out.events.size()&&out.lap==n);
    U x=a.primary,k=0;
    for(U t=0;t<n;t++){
        if(k<e&&out.events[k].x==t){U dt,next;uint8_t byte;get(f,dt);get(f,byte);get(f,next);
            assert(dt==t&&rows[out.events[k].y]==x&&byte==last[x]);++k;
        }
        x=lf[x];
    }
    assert(k==e&&x==a.primary&&f.peek()==EOF);
}
int main(int argc,char** argv){
    assert(argc>=5);
    std::string mode=argv[1],dir=argv[2],wires=argv[3];
    if(mode=="rss"){
        U b=std::stoul(argv[4]),k=std::stoul(argv[5]);std::string arm=argv[6];
        auto a=load(dir,b,k);U pre=hwm(),post=0;Q retained=0;
        if(arm=="direct"){Direct d;d.build(a);post=hwm();retained=d.capacity();}
        else{Induced d;d.build(a);post=hwm();retained=d.capacity();}
        std::cout<<b<<','<<k<<','<<arm<<','<<pre<<','<<post<<','<<retained<<'\n';return 0;
    }
    std::vector<U> ids;for(int i=4;i<argc;i++)ids.push_back(std::stoul(argv[i]));
    std::array<U,3> depths{4,8,16};
    if(mode=="check"){
        for(U b:ids)for(U k:depths){auto a=load(dir,b,k);Direct d;Induced i;d.build(a);i.build(a);
            assert(d.out==i.out);oracle(a,i.out,wires+"/"+std::to_string(b)+".pred"+std::to_string(k));
            std::cout<<b<<','<<k<<','<<d.out.lap<<','<<d.out.events.size()<<'\n';}return 0;
    }
    assert(mode=="time");std::vector<std::vector<Input>> inputs(3);
    for(U i=0;i<3;i++)for(U b:ids)inputs[i].push_back(load(dir,b,depths[i]));
    Direct direct;Induced induced;
    for(auto& blocks:inputs)for(auto& a:blocks){direct.build(a);induced.build(a);assert(direct.out==induced.out);}
    for(U r=0;r<24;r++)for(U j=0;j<3;j++){
        U i=(r+j)%3,k=depths[i];
        for(U p=0;p<2;p++){
            bool treatment=(r+i+p)%2;Q lap=0;
            for(U b=0;b<ids.size();b++){
                auto& a=inputs[i][b];Q start=cpu();
                if(treatment)induced.build(a);else direct.build(a);
                Q elapsed=cpu()-start;lap+=elapsed;
                auto& out=treatment?induced.out:direct.out;
                std::cout<<r<<','<<k<<','<<p<<','<<(treatment?"induced":"direct")<<','<<ids[b]<<','<<elapsed<<','<<out.events.size()<<','<<out.lap<<'\n';
            }
        }
    }
}
