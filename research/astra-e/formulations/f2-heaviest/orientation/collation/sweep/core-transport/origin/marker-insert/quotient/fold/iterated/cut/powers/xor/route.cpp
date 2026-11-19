#define POWER_SOLVER_LIBRARY
#include "../direct_cut.cpp"

struct Part { U k,j,n,alias; };
template<bool Exact> struct ParityCut {
    U m=0,self=0,ingress=0,mask=0;
    bool done=false;
    std::vector<U> base;
    std::vector<Part> parts;
    std::vector<uint8_t> color;
    explicit ParityCut(const Root& in):base(bases(in)),color(in.bytes.size()) {
        m=base.back(); U pos=0;
        for(auto [k,n]:in.r) {
            if(k==in.mark){assert(n==1);done=(pos==base[k]);break;}
            pos+=n;
        }
        if(done)return;
        U q=1;while(q<color.size())q*=2;
        std::vector<int> d(q);
        U j=0;pos=0;parts.reserve(in.r.size()+in.bytes.size());
        for(auto [k,n]:in.r)while(n) {
            U w=std::min(n,base[j+1]-pos);
            d[k^j]+=int(w);parts.push_back({k,j,w,0});
            n-=w;pos+=w;if(pos==base[j+1])++j;
        }
        self=d[0];int score=d[0];
        if constexpr(Exact) {
            for(U w=1;w<q;w*=2)for(U i=0;i<q;i+=2*w)for(U j=i;j<i+w;++j) {
                int a=d[j],b=d[j+w];d[j]=a+b;d[j+w]=a-b;
            }
            auto best=std::min_element(d.begin(),d.end());mask=best-d.begin();score=*best;
        } else {
            for(U w=1;w<q;w*=2) {
                int sum=0;
                for(U t=w;t<2*w;++t)sum+=__builtin_parity(mask&t)?-d[t]:d[t];
                if(sum>0)mask+=w;
                score-=std::abs(sum);
            }
        }
        assert(score<=int(self)&&(int(m)-score)%4==0);ingress=U(int(m)-score)/4;
        for(U k=0;k<color.size();++k)color[k]=__builtin_parity(mask&(k^in.mark));
    }
    Root apply(const Root& in,std::vector<Node>& grammar) {
        const U no=UINT32_MAX,K=color.size();
        std::vector<U> first(K+1),last(K,no);
        for(U k=0;k<K;++k)first[k+1]=color[k];
        for(auto s:parts)if(!color[s.k]&&last[s.k]!=s.j){++first[s.k+1];last[s.k]=s.j;}
        std::partial_sum(first.begin(),first.end(),first.begin());
        U A=first.back();
        std::vector<U> rem(A),into(A,no),id(A,no),deleted(K),cursor(first.begin(),first.end()-1);
        std::fill(last.begin(),last.end(),no);
        for(auto& s:parts) {
            if(color[s.k])s.alias=first[s.k];
            else {
                if(last[s.k]!=s.j){last[s.k]=s.j;++cursor[s.k];}
                s.alias=cursor[s.k]-1;
                if(color[s.j]){into[s.alias]=s.j;deleted[s.j]+=s.n;}
            }
            rem[s.alias]+=s.n;
        }
        for(U j=0;j<K;++j)if(deleted[j])rem[first[j]]-=deleted[j];
        Root out;out.bytes.reserve(A);
        for(U k=0;k<K;++k)for(U a=first[k];a<first[k+1];++a)if(rem[a]) {
            U node=in.bytes[k],j=into[a];id[a]=out.bytes.size();
            if(j!=no){U r=in.bytes[j],f=grammar.size();grammar.push_back({grammar[node].n+grammar[r].n,node,r,Node::Join});node=f;}
            out.bytes.push_back(node);
        }
        out.mark=id[first[in.mark]];
        std::vector<Run> map;
        std::vector<U> at(K),used(K);
        U prior=no;
        for(auto s:parts) if(deleted[s.j]) {
            U v=into[s.alias]!=no?s.alias:first[s.j];
            if(s.j!=prior){prior=s.j;at[s.j]=map.size();map.push_back({v,s.n});}
            else append(map,v,s.n);
        }
        out.r.reserve(in.r.size());
        for(auto s:parts) {
            if(into[s.alias]!=no)continue;
            if(!deleted[s.k]){append(out.r,id[s.alias],s.n);continue;}
            U n=s.n;
            while(n) {
                auto p=map[at[s.k]];U take=std::min(n,p.n-used[s.k]);
                assert(id[p.k]!=no);append(out.r,id[p.k],take);
                used[s.k]+=take;n-=take;
                if(used[s.k]==p.n){++at[s.k];used[s.k]=0;}
            }
        }
        return out;
    }
};

std::vector<uint8_t> execute(const Root& in,const std::string& arm,Stats& s) {
    if(arm=="direct")return direct(in);
    if(arm=="graph")return decode_with<Cut>(in,s);
    if(arm=="greedy")return decode_with<ParityCut<false>>(in,s);
    assert(arm=="exact");return decode_with<ParityCut<true>>(in,s);
}
int main(int argc,char** argv) {
    assert(argc>=4); auto in=load(argv[1]);std::string mode=argv[2];
    std::vector<std::string> arms={"graph","greedy","exact","direct"};
    if(mode=="check") {
        auto expected=direct(in);
        for(const auto& arm:arms) if(arm!="direct") {
            Stats s;auto out=execute(in,arm,s);assert(out==expected);
            std::filesystem::path p=argv[3];
            std::ofstream f(p/(arm+"-trajectory.csv"));
            f<<"depth,rows,input_runs,source_types,self_mass,original_ingress,after,output_runs,after_types,power\n";
            for(U i=0;i<s.passes.size();++i) {
                auto x=s.passes[i];f<<i<<','<<x.old_m<<','<<x.old_runs<<','<<x.old_types<<','<<x.self<<','<<x.ingress<<','<<x.after<<','<<x.new_runs<<','<<x.new_types<<','<<x.normalized<<'\n';
            }
            std::ofstream data(p/(arm+"-output.bin"),std::ios::binary);
            data.write(reinterpret_cast<const char*>(out.data()),out.size());
        }
        return 0;
    }
    std::cout<<"round,arm,thread_ns,choice_ns,publish_ns,normalize_ns,emit_ns,bytes,nodes,peak_kib\n";
    int reps=std::stoi(argv[3]);
    for(int i=0;i<reps;++i)for(int j=0;j<(mode=="panel"?4:1);++j){
        int k=i%8<4?(i+j)%4:(i+3-j)%4;
        auto arm=mode=="panel"?arms[k]:mode;Stats s;U n;auto t=cpu();
        { auto v=execute(in,arm,s);n=v.size();asm volatile("" : : "g"(v.data()) : "memory"); }
        auto dt=cpu()-t;
        std::cout<<i<<','<<arm<<','<<dt<<','<<s.clocks.graph<<','<<s.clocks.apply<<','<<s.clocks.powers<<','<<s.clocks.emit<<','<<n<<','<<s.nodes<<','<<resident_peak()<<'\n';
    }
}
