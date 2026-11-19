#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <map>
#include <vector>
using U=uint32_t;
using Pair=std::pair<U,U>;
using Key=std::array<U,6>;
struct Source{
    struct Run{U begin,end,base,byte;};
    std::vector<U> last,f,lf,psi;
    std::vector<Run> runs;
    explicit Source(std::vector<U> raw):last(std::move(raw)),f(last.size()),lf(last.size()),psi(last.size()){
        std::array<U,257> base{};
        for(auto b:last)base[b+1]++;
        for(U i=0;i<256;i++)base[i+1]+=base[i];
        for(U x=0;x<last.size();){
            U y=x+1,b=last[x];while(y<last.size()&&last[y]==b)y++;
            runs.push_back({x,y,base[b],b});
            for(;x<y;x++){lf[x]=base[b]++;psi[lf[x]]=x;f[lf[x]]=b;}
        }
    }
};
struct Model{
    std::vector<U> end;
    std::vector<Pair> mode;
    U atoms=0,edges=0;
    U state(U x)const{return std::upper_bound(end.begin(),end.end(),x)-end.begin();}
    Model(const Source& p,std::vector<U> bounds):end(std::move(bounds)),mode(end.size()){
        // Only runs and physical interval ends enter histogram construction.
        U s=0;std::map<Pair,U> hist;
        for(auto r:p.runs){
            U x=r.begin;
            while(x<r.end){
                U y=r.base+x-r.begin,j=state(y);
                U width=std::min({r.end-x,end[s]-x,end[j]-y});
                assert(width);hist[{r.byte,j}]+=width;atoms++;x+=width;
                if(x==end[s]){
                    auto best=hist.begin();
                    for(auto it=hist.begin();it!=hist.end();++it)if(it->second>best->second)best=it;
                    mode[s++]=best->first;edges+=hist.size();hist.clear();
                }
            }
        }
        assert(s==end.size()&&atoms<=p.runs.size()+2*(s-1));
    }
};
struct Result{U n,c,r,atoms,edges,e,cl,runs,g8,covered,marked_phase,omissions;};
Result test(const Source& p,const std::vector<U>& ends,U primary,bool finite){
    Model m(p,ends);
    U n=p.last.size();
    std::vector<U> state(n),rank(n,n),rows;
    std::vector<bool> bad(n),shifted(n);
    for(U x=0;x<n;x++)state[x]=m.state(x);
    for(U x=0;x<n;x++){
        bad[x]=Pair(p.last[x],state[p.lf[x]])!=m.mode[state[x]];
        shifted[p.lf[x]]=bad[x];
    }
    for(U x=0;x<n;x++)if(shifted[x]){rank[x]=rows.size();rows.push_back(x);}
    U num=rows.size();
    std::vector<U> src(num),actual(num),col(num),starts;
    std::map<Key,U> known;std::vector<Key> allkeys;
    struct Rule{U len,b,post;};std::vector<Rule> rules;
    U traversed=0;
    for(U i=0;i<num;i++){
        U x=rows[i],z=x,len=0;
        do{z=p.psi[z];++len;}while(rank[z]==n);
        Key key={len,p.f[x],state[x],p.f[z],state[z],state[p.psi[z]]};
        if(finite)allkeys.push_back(key);
        auto [it,ins]=known.try_emplace(key,known.size());U cl=it->second;
        if(ins){starts.push_back(i);rules.push_back({len,p.f[x],state[z]});}
        else{
            assert(i&&src[i-1]==cl&&rows[i-1]+1==x);
            assert(actual[i-1]<rank[z]);
        }
        // Spell W using only its leading symbol, length and final state.
        U s=state[z],q=z;
        for(U j=1;j<len;j++){
            assert(m.mode[s].first==p.last[q]);s=m.mode[s].second;q=p.lf[q];assert(s==state[q]);
        }
        assert(p.lf[q]==x);
        src[i]=cl;actual[i]=rank[z];col[rank[z]]=cl;traversed+=len;
    }
    auto ptr=starts;std::vector<U> sparse(num);
    U runs=0;
    for(U i=0;i<num;i++){sparse[ptr[col[i]]++]=i;runs+=!i||col[i]!=col[i-1];}
    assert(sparse==actual);
    U omitted=0;
    if(finite){
        for(U field=1;field<6;field++){
            std::map<Key,U> previous;bool conflict=false;
            for(U i=0;i<num;i++){
                auto short_key=allkeys[i];short_key[field]=0;
                auto [it,ins]=previous.try_emplace(short_key,i);
                if(!ins){
                    auto j=it->second;
                    U x=rows[i],y=rows[j];
                    for(U step=0;step<short_key[0];step++){
                        if(p.f[x]!=p.f[y])conflict=true;
                        x=p.psi[x];y=p.psi[y];
                    }
                    if(rows[i]!=rows[j]+1||actual[i]<=actual[j])conflict=true;
                    it->second=i;
                }
            }
            if(conflict)omitted|=1u<<field;
        }
    }
    U mark_steps=0;
    for(U mark=finite?0:primary;mark<(finite?n:primary+1);mark++){
        U x=mark,phase=0;while(rank[x]==n&&phase<n){x=p.psi[x];++phase;}
        mark_steps=std::max(mark_steps,phase);
        U checked=0,q=mark;
        for(;checked<phase;checked++)q=p.psi[q];
        assert(q==x);
        if(phase==n)continue;
        U i=rank[x];
        while(checked<n){
            auto rule=rules[src[i]];
            assert(rule.b==p.f[q]);
            std::vector<U> w(rule.len);w[0]=rule.b;U s=rule.post;
            for(U j=rule.len-1;j;j--){w[j]=m.mode[s].first;s=m.mode[s].second;}
            for(U b:w){if(checked==n)break;assert(b==p.f[q]);q=p.psi[q];++checked;}
            i=sparse[i];
        }
    }
    U x=primary,gap=0,g8=0;
    for(U i=0;i<n;i++){
        if(!bad[x])gap++;else{g8+=gap/8;gap=0;}
        x=p.lf[x];
    }
    g8+=gap/8;
    return {n,U(ends.size()),U(p.runs.size()),m.atoms,m.edges,num,U(starts.size()),runs,g8*8,traversed,mark_steps,omitted};
}
U word(std::istream& f){unsigned char b[4];f.read(reinterpret_cast<char*>(b),4);assert(f);return U(b[0])|(U(b[1])<<8)|(U(b[2])<<16)|(U(b[3])<<24);}
int main(int argc,char** argv){
    if(argc==1){
        uint64_t cases=0,marks=0;U omit=0;
        for(U sigma:{2,3})for(U n=1,ways=sigma;n<=(sigma==2?7:5);n++,ways*=sigma){
            for(U code=0;code<ways;code++){
                U q=code;std::vector<U> l(n);for(auto& v:l){v=q%sigma;q/=sigma;}
                Source p(l);
                for(U mask=0;mask<(1u<<(n-1));mask++){
                    std::vector<U> ends;for(U i=1;i<n;i++)if((mask>>(i-1))&1)ends.push_back(i);
                    ends.push_back(n);auto r=test(p,ends,0,true);
                    U added=r.omissions&~omit;
                    if(added){
                        std::cout<<"witness lost_fields_mask="<<added<<" last=";for(auto b:l)std::cout<<b;
                        std::cout<<" ends=";for(auto e:ends)std::cout<<e<<' ';std::cout<<"\n";
                    }
                    omit|=r.omissions;cases++;marks+=n;
                }
            }
        }
        std::cout<<"models="<<cases<<" marks="<<marks<<" failed_omission_mask="<<omit<<"\n";
    }else{
        std::ifstream f(argv[1],std::ios::binary);char magic[8];f.read(magic,8);
        assert(std::string(magic,8)=="CLKMASK1");U n=word(f),primary=word(f);
        std::vector<U> l(n);for(auto& b:l)b=static_cast<unsigned char>(f.get());assert(f);
        Source p(std::move(l));
        for(U w:{16,32,64,128,256,512,1024}){
            std::vector<U> ends;for(U x=0;x<n;x+=w)ends.push_back(std::min(x+w,n));
            auto r=test(p,ends,primary,false);
            std::cout<<w<<','<<r.n<<','<<r.c<<','<<r.r<<','<<r.atoms<<','<<r.edges<<','<<r.e<<','<<r.cl
                     <<','<<r.runs<<','<<r.g8<<','<<r.covered<<','<<r.marked_phase<<"\n";
        }
    }
    return 0;
}
