#include "induce.hpp"
#include <fstream>
#include <iostream>
#include <set>
#include <string>
using namespace induction;
struct Oracle{
    std::vector<U> last,lf,psi,f;
    std::vector<Run> runs;
    explicit Oracle(std::vector<U> bytes):last(std::move(bytes)),lf(last.size()),psi(last.size()),f(last.size()){
        std::array<U,257> base{};
        for(U b:last)++base[b+1];
        std::partial_sum(base.begin(),base.end(),base.begin());
        for(U x=0;x<last.size();){
            U y=x+1,b=last[x];while(y<last.size()&&last[y]==b)++y;
            runs.push_back({x,y,base[b],b});
            for(;x<y;++x){lf[x]=base[b]++;psi[lf[x]]=x;f[lf[x]]=b;}
        }
    }
};
std::vector<Span> spans(const std::vector<unsigned char>& hit){
    std::vector<Span> out;
    for(U x=0;x<hit.size();++x){
        if(x+1==hit.size()||hit[x+1]!=hit[x])out.push_back({x+1,bool(hit[x])});
    }
    return out;
}
struct Audit {U cap_bytes=0,rule_visits=0,distinct_rules=0,ordinary_runs=0,canon=0,mark_bytes=0;};
Audit verify(const Oracle& p,std::vector<unsigned char> hit,U mark,bool all_marks,std::string policy){
    hit[mark]=1;
    auto S=spans(hit);
    auto q=induce(p.runs,S);U n=p.last.size(),m=q.kept;
    std::vector<U> rank(n,none),rows;
    for(U x=0;x<n;x++)if(hit[x]){rank[x]=rows.size();rows.push_back(x);}
    assert(rows.size()==m);
    std::vector<U> back(m),front(m),rule(m),in(m),base(q.records.size()),buffer,stack;
    std::vector<bool> assigned(m),used(q.grammar.size());
    std::vector<U> ids(q.records.size());std::iota(ids.begin(),ids.end(),0);
    std::sort(ids.begin(),ids.end(),[&](U i,U j){return q.records[i].target<q.records[j].target;});
    Audit z;
    for(U c=0;c<ids.size();++c){
        auto r=q.records[ids[c]];
        base[c]=r.target;
        buffer.clear();stack={r.rule};
        while(!stack.empty()){
            U b=stack.back();stack.pop_back();z.rule_visits++;
            if(b<256){buffer.push_back(b);continue;}
            used[b-256]=true;auto node=q.grammar[b-256];stack.push_back(node.right);stack.push_back(node.left);
        }
        for(U j=0;j<r.width;++j){
            U i=r.source+j,k=r.target+j,x=rows[i];
            for(U b:buffer){assert(b==p.last[x]);x=p.lf[x];}
            assert(x==rows[k]&&!assigned[k]);assigned[k]=true;
            U y=rows[i];do{y=p.lf[y];}while(!hit[y]);assert(y==x);
            back[i]=k;front[k]=i;in[i]=c;rule[k]=r.rule;
        }
        z.cap_bytes+=buffer.size()*r.width;
    }
    assert(z.cap_bytes<=n);
    // Distinct forward source intervals get increasing F IDs, with the ID
    // stored at the forward destination. Check ordinary stable LF directly.
    auto ptr=base;
    for(U i=0;i<m;++i){
        assert(ptr[in[i]]++==back[i]);
        z.ordinary_runs+=i==0||in[i]!=in[i-1];
        z.canon+=i==0||back[i]!=back[i-1]+1||rule[back[i]]!=rule[back[i-1]];
    }
    z.distinct_rules=std::count(used.begin(),used.end(),true);
    for(U probe=0;probe<(all_marks?m:1);++probe){
        U i=all_marks?probe:rank[mark],x=rows[i],wrote=0;
        while(wrote<n){
            stack={rule[i]};
            while(!stack.empty()&&wrote<n){
                U b=stack.back();stack.pop_back();
                if(b<256){assert(b==p.f[x]);x=p.psi[x];++wrote;}
                else{auto node=q.grammar[b-256];stack.push_back(node.left);stack.push_back(node.right);}
            }
            i=front[i];
        }
        z.mark_bytes+=wrote;
    }
    if(!policy.empty()){
        std::cout<<policy<<','<<n<<','<<p.runs.size()<<','<<m<<','<<q.seams<<','<<q.initial<<','
          <<q.strict<<','<<q.equal<<','<<q.self<<','<<q.partial<<','<<q.grammar.size()<<','
          <<z.distinct_rules<<','<<q.records.size()<<','<<z.canon<<','<<z.rule_visits<<','
          <<z.cap_bytes<<','<<z.mark_bytes<<'\n';
    }
    return z;
}
U u32(std::istream& f){unsigned char b[4];f.read(reinterpret_cast<char*>(b),4);assert(f);return U(b[0])|(U(b[1])<<8)|(U(b[2])<<16)|(U(b[3])<<24);}
int main(int argc,char** argv){
    if(argc==1){
        uint64_t cases=0,bytes=0;
        for(U alphabet:{2,3})for(U n=1,ways=alphabet;n<=(alphabet==2?7:5);++n,ways*=alphabet){
            for(U code=0;code<ways;++code){
                U value=code;std::vector<U> l(n);
                for(auto& b:l){b=value%alphabet;value/=alphabet;}
                Oracle p(l);
                for(U bits=1;bits<(1u<<n);++bits){
                    std::vector<unsigned char> hit(n);U mark=0;
                    for(U x=0;x<n;++x)if(bits>>x&1)hit[x]=1,mark=x;
                    auto z=verify(p,hit,mark,true,"");++cases;bytes+=z.mark_bytes;
                }
            }
        }
        std::cout<<"nonempty_sections="<<cases<<" marked_oracle_bytes="<<bytes<<'\n';
        return 0;
    }
    std::ifstream f(argv[1],std::ios::binary);char magic[8];f.read(magic,8);assert(std::string(magic,8)=="CLKMASK1");
    U n=u32(f),mark=u32(f);std::vector<U> l(n);for(auto& a:l)a=static_cast<unsigned char>(f.get());
    Oracle p(l);std::vector<unsigned char> selected(n);U errors=0;
    for(U x=0;x<n;++x){int b=f.get();assert(b==0||b==1);selected[p.lf[x]]=b;errors+=b;}
    verify(p,selected,mark,false,"seed8");
    std::fill(selected.begin(),selected.end(),0);
    for(auto r:p.runs)selected[r.begin]=1;
    verify(p,selected,mark,false,"heads");
    for(auto r:p.runs)std::fill(selected.begin()+r.begin,selected.begin()+r.end,r.end-r.begin<=4);
    verify(p,selected,mark,false,"minor4");
    std::fill(selected.begin(),selected.end(),0);std::fill_n(selected.begin(),errors,1);
    verify(p,selected,mark,false,"prefix");
    std::array<U,256> count{};for(U b:p.last)count[b]++;
    std::vector<std::pair<U,U>> order;for(U b=0;b<256;++b)if(count[b])order.emplace_back(count[b],b);
    std::sort(order.begin(),order.end());std::array<bool,256> rare{};
    for(U j=0;j<std::min<size_t>(16,order.size());++j)rare[order[j].second]=true;
    for(U x=0;x<n;++x)selected[x]=rare[p.f[x]];
    verify(p,selected,mark,false,"rareF16");
    return 0;
}
