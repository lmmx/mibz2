#define POWER_SOLVER_LIBRARY
#include "../direct_cut.cpp"

template<bool Gaps,class Colors> Root publish_compact(const Root& in,std::vector<Node>& grammar,
                                                     std::vector<Slice>& parts,const Colors& color) {
        const U no=UINT32_MAX,K=color.size();
        std::vector<U> first(K+1),last(K,no);
        for(U k=0;k<K;++k)first[k+1]=color[k];
        for(auto s:parts)if(!color[s.source]) {
            U group=Gaps&&!color[s.target]?K:s.target;
            if(last[s.source]!=group){++first[s.source+1];last[s.source]=group;}
        }
        std::partial_sum(first.begin(),first.end(),first.begin());
        U A=first.back();
        std::vector<U> rem(A),into(A,no),id(A,no),deleted(K),cursor(first.begin(),first.end()-1);
        std::fill(last.begin(),last.end(),no);
        for(auto& s:parts) {
            if(color[s.source])s.edge=first[s.source];
            else {
                U group=Gaps&&!color[s.target]?K:s.target;
                if(last[s.source]!=group){last[s.source]=group;++cursor[s.source];}
                s.edge=cursor[s.source]-1;
                if(color[s.target]){into[s.edge]=s.target;deleted[s.target]+=s.width;}
            }
            rem[s.edge]+=s.width;
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
        for(auto s:parts) if(deleted[s.target]) {
            U v=into[s.edge]!=no?s.edge:first[s.target];
            if(s.target!=prior){prior=s.target;at[s.target]=map.size();map.push_back({v,s.width});}
            else append(map,v,s.width);
        }
        out.r.reserve(in.r.size());
        for(auto s:parts) {
            if(into[s.edge]!=no)continue;
            if(!deleted[s.source]){append(out.r,id[s.edge],s.width);continue;}
            U n=s.width;
            while(n) {
                auto p=map[at[s.source]];U take=std::min(n,p.n-used[s.source]);
                assert(id[p.k]!=no);append(out.r,id[p.k],take);
                used[s.source]+=take;n-=take;
                if(used[s.source]==p.n){++at[s.source];used[s.source]=0;}
            }
        }
        return out;
    }
template<bool Exact,bool Gaps=false> struct ParityCut {
    U m=0,self=0,ingress=0,mask=0;
    bool done=false;
    std::vector<U> base;
    std::vector<Slice> parts;
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
        return publish_compact<Gaps>(in,grammar,parts,color);
    }
};

#include "stream.hpp"

struct GraphCompact: Cut {
    using Cut::Cut;
    Root apply(const Root& in,std::vector<Node>& grammar) {
        return publish_compact<false>(in,grammar,parts,color);
    }
};

std::vector<uint8_t> execute(const Root& in,const std::string& arm,Stats& s) {
    if(arm=="direct")return direct(in);
    if(arm=="graph")return decode_with<Cut>(in,s);
    if(arm=="stream")return decode_with<StreamCut<false>>(in,s);
    if(arm=="streamgaps")return decode_with<StreamCut<true>>(in,s);
    if(arm=="graphcompact")return decode_with<GraphCompact>(in,s);
    if(arm=="gaps")return decode_with<ParityCut<false,true>>(in,s);
    if(arm=="greedy")return decode_with<ParityCut<false>>(in,s);
    assert(arm=="exact");return decode_with<ParityCut<true>>(in,s);
}
#ifndef XOR_ROUTE_LIBRARY
int main(int argc,char** argv) {
    assert(argc>=4); auto in=load(argv[1]);std::string mode=argv[2];
    std::vector<std::string> arms={"graph","greedy","exact","direct"};
    if(mode=="panel5"||mode=="check5")arms={"graph","graphcompact","greedy","gaps","direct"};
    if(mode=="panelstream"||mode=="checkstream")arms={"graph","graphcompact","stream","streamgaps","direct"};
    if(mode=="check"||mode=="check5"||mode=="checkstream") {
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
    int k_arms=int(arms.size());
    bool panel=mode=="panel"||mode=="panel5"||mode=="panelstream";
    for(int i=0;i<reps;++i)for(int j=0;j<(panel?k_arms:1);++j){
        int k=i%(2*k_arms)<k_arms?(i+j)%k_arms:(i+k_arms-1-j)%k_arms;
        auto arm=panel?arms[k]:mode;Stats s;U n;auto t=cpu();
        { auto v=execute(in,arm,s);n=v.size();asm volatile("" : : "g"(v.data()) : "memory"); }
        auto dt=cpu()-t;
        std::cout<<i<<','<<arm<<','<<dt<<','<<s.clocks.graph<<','<<s.clocks.apply<<','<<s.clocks.powers<<','<<s.clocks.emit<<','<<n<<','<<s.nodes<<','<<resident_peak()<<'\n';
    }
}
#endif
