#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <iterator>
#include <list>
#include <numeric>
#include <queue>
#include <limits>
#include <string>
#include <tuple>
#include <vector>
using U=uint32_t;
using Bytes=std::vector<uint8_t>;
struct Span {U begin,end;};
struct Input { U n,mark,exception_count;Bytes last,selected;std::vector<U> lf; };
U word(std::istream& f){
    unsigned char b[4];f.read(reinterpret_cast<char*>(b),4);assert(f);
    return U(b[0])|(U(b[1])<<8)|(U(b[2])<<16)|(U(b[3])<<24);
}
Input load(const char* path){
    std::ifstream f(path,std::ios::binary);char name[8];f.read(name,8);assert(std::string(name,8)=="CLKMASK1");
    Input in{};in.n=word(f);in.mark=word(f);in.last.resize(in.n);in.selected.resize(in.n);
    f.read(reinterpret_cast<char*>(in.last.data()),in.n);
    Bytes exceptions(in.n);f.read(reinterpret_cast<char*>(exceptions.data()),in.n);assert(f&&f.peek()==EOF);
    std::array<U,256> counts{},base{};for(auto c:in.last)counts[c]++;
    std::exclusive_scan(counts.begin(),counts.end(),base.begin(),0U);
    for(U i=0;i<in.n;i++) {
        U y=base[in.last[i]]++;in.lf.push_back(y);
        assert(exceptions[i]<=1);in.selected[y]=exceptions[i];in.exception_count+=exceptions[i];
    }
    in.selected[in.mark]=1;
    U along=in.mark,visited=0;do{along=in.lf[along];++visited;}while(along!=in.mark);
    assert(visited==in.n); // This real panel can count every complement bigram.
    return in;
}
// Store the two orderings and widths independently of the main pointer implementation.
using List=std::list<U>;
struct Exchange {
    List source,target;
    std::vector<List::iterator> src,dst;
    std::vector<U>w;
    Exchange(std::vector<U> width,const std::vector<U>& order):src(width.size()),dst(width.size()),w(std::move(width)){
        for(U x=0;x<w.size();x++)src[x]=source.insert(source.end(),x);
        for(U x:order)dst[x]=target.insert(target.end(),x);
    }
    std::array<U,3> eliminate(U remaining) {
        U steps=0,clips=0;
        while(remaining){
            U a=source.back(),b=target.back(),x=w[a],y=w[b],k=std::min({x,y,remaining});assert(k);
            remaining-=k;steps++;
            if(a==b){
                if(k<x){w[a]-=k;clips++;}
                else{source.pop_back();target.pop_back();}
            }else if(k<x&&k<y){
                U c=w.size();w[a]-=k;w[b]-=k;w.push_back(k);clips++;
                src.push_back(source.insert(std::next(src[b]),c));
                dst.push_back(target.insert(std::next(dst[a]),c));
            }else if(x==y){
                source.pop_back();target.pop_back();*dst[a]=b;dst[b]=dst[a];
            }else if(x<y){
                w[b]-=x;source.splice(std::next(src[b]),source,src[a]);
            }else{
                w[a]-=y;target.splice(std::next(dst[a]),target,dst[b]);
            }
        }
        assert(source.size()==target.size());
        return {steps,clips,U(source.size())};
    }
};
std::vector<U> layout(const Bytes& selected,const std::string& policy) {
    std::vector<Span> outside;std::vector<U> order;
    for(U x=0;x<selected.size();){
        if(selected[x]){order.push_back(x++);continue;}
        U a=x;while(x<selected.size()&&!selected[x])x++;outside.push_back({a,x});
    }
    if(policy=="reflected"){std::reverse(order.begin(),order.end());std::reverse(outside.begin(),outside.end());}
    if(policy=="reverse_spans")std::reverse(outside.begin(),outside.end());
    if(policy=="long_first"||policy=="short_first"){
        bool longer=policy=="long_first";
        std::stable_sort(outside.begin(),outside.end(),[longer](Span a,Span b){
            return longer ? a.end-a.begin>b.end-b.begin : a.end-a.begin<b.end-b.begin;});
    }
    for(auto s:outside){
        if(policy=="reflected"){for(U x=s.end;x>s.begin;)order.push_back(--x);}
        else{for(U x=s.begin;x<s.end;x++)order.push_back(x);}
    }
    return order;
}
struct Prepared {std::vector<U>map,inv,order;U m=0,seams=0,adj=0,stab=0;int sign;};
Prepared coordinates(const Input& in,const Bytes& selected,const std::string& policy){
    Prepared q;q.m=std::accumulate(selected.begin(),selected.end(),0U);q.sign=policy=="reflected"?-1:1;
    q.order=layout(selected,policy);std::vector<U> pos(in.n);
    for(U i=0;i<in.n;i++)pos[q.order[i]]=i;
    q.map.resize(in.n);q.inv.resize(in.n);
    for(U i=0;i<in.n;i++){q.map[i]=pos[in.lf[q.order[i]]];q.inv[q.map[i]]=i;
        if(i&&(i==q.m||int(q.order[i])-int(q.order[i-1])!=q.sign))q.seams++;
    }
    U min_left=q.m;
    for(U i=q.m+1;i<in.n;i++){
        U f=q.map[i],b=q.inv[i];
        if(f==i-1||b==i-1)q.adj++;
        if((min_left<=f&&f<i)||(min_left<=b&&b<i)){
            q.stab++;min_left=i;
        }
    }
    return q;
}
struct Family {U lo,hi,len;};
U stabbing(std::vector<Family>& families,U m) {
    if(families.empty())return 0;
    std::sort(families.begin(),families.end(),[](auto a,auto b){return a.lo<b.lo;});
    U inf=std::numeric_limits<U>::max();
    std::vector<U> future(families.size()+1,inf);
    for(U j=families.size();j>0;j--){
        const auto& f=families[j-1];future[j-1]=std::min(future[j],f.lo+f.len-1);
    }
    using Entry=std::pair<U,U>;
    std::priority_queue<Entry,std::vector<Entry>,std::greater<Entry>> active;
    U next=m,picks=0,i=0;
    while(true){
        while(i<families.size()&&families[i].lo<=next){auto f=families[i++];active.push({f.len,f.hi});}
        while(!active.empty()&&active.top().second<next)active.pop();
        U right=future[i];
        if(!active.empty())right=std::min(right,next+active.top().first-1);
        if(right==inf)break;
        ++picks;next=right+1;
    }
    return picks;
}
void price(const Input& in,U block,const std::string& section,const std::string& policy,const Prepared& q,bool merge){
    std::vector<Family> families;std::vector<U> widths,reverse;std::vector<U> start(in.n,in.n);U a=0;
    for(U i=1;i<=in.n;i++){
        bool split=i==in.n;
        if(!split){
            auto t=q.map[i],prev=q.map[i-1];
            split=(t!=prev+1 || in.last[q.order[i]]!=in.last[q.order[i-1]]);
            if(!merge && (i==q.m || t==q.m || int(q.order[i])-int(q.order[i-1])!=q.sign))split=true;
        }
        if(split){
            U y=q.map[a],w=i-a,lo=std::min(a,y),hi=lo+w-1;
            if(a!=y&&hi>=q.m)families.push_back({std::max(lo,q.m),hi,std::max(a,y)-lo});
            start[y]=widths.size();widths.push_back(w);a=i;
        }
    }
    assert(stabbing(families,q.m)==q.stab);
    U pairs=0;
    std::vector<U> source_cut(in.n,0);
    for(U first=0,i=0;i<widths.size();++i){source_cut[first]=1;first+=widths[i];}
    for(U x=q.m;x<in.n;++x)
        pairs+=U(x==q.m||source_cut[x]||start[x]!=in.n);
    U atoms=widths.size();
    for(U id:start)if(id!=in.n)reverse.push_back(id);
    assert(reverse.size()==atoms);
    Exchange e(std::move(widths),reverse);auto [steps,clips,final]=e.eliminate(in.n-q.m);
    U floor=U(q.m<in.n)+q.stab;
    assert(steps>=floor&&steps+final>=atoms);
    assert(steps>=pairs);
    std::cout<<block<<','<<section<<','<<policy<<','<<merge<<','<<in.n<<','<<q.m<<','<<q.seams<<','<<atoms<<','
             <<q.adj<<','<<floor<<','<<steps<<','<<clips<<','<<final<<','<<pairs<<"\n";
}
int main(int argc,char**argv){
    assert(argc==3);U block=std::stoul(argv[1]);auto in=load(argv[2]);
    for(std::string seed:{"seed8","heads","prefix"}){
        Bytes section=in.selected;
        if(seed=="heads"){
            for(U i=0;i<in.n;i++)section[i]=(i==0||in.last[i]!=in.last[i-1]);
        }else if(seed=="prefix"){
            U m=in.exception_count;
            std::fill(section.begin(),section.end(),0);std::fill_n(section.begin(),m,1);
        }
        section[in.mark]=1;
        for(std::string p:{"stable","reverse_spans","long_first","short_first","reflected"}){
            auto q=coordinates(in,section,p);
            for(bool merge:{false,true})price(in,block,seed,p,q,merge);
        }
    }
}
