// Same greedy visitation and coloring as Cut, using only the ordered bucket fragments.
template<bool Gaps> struct StreamCut {
    U m=0,self=0,ingress=0;
    bool done=false;
    std::vector<Slice> parts;
    std::vector<int8_t> color;
    explicit StreamCut(const Root& in):color(in.bytes.size(),-1) {
        auto b=bases(in);m=b.back();U pos=0;
        for(auto [k,n]:in.r) {
            if(k==in.mark){assert(n==1);done=(pos==b[k]);break;}
            pos+=n;
        }
        if(done)return;
        std::vector<U> offset(b.size());parts.reserve(in.r.size()+color.size());
        U j=0;pos=0;
        for(auto [k,n]:in.r)while(n) {
            U w=std::min(n,b[j+1]-pos);parts.push_back({k,j,w,0});
            if(k==j)self+=w;
            n-=w;pos+=w;if(pos==b[j+1])offset[++j]=parts.size();
        }
        std::vector<int> pending(color.size(),0);
        auto decide=[&](U k,bool forced) {
            int vote=pending[k];
            for(U i=offset[k];i<offset[k+1];++i) {
                auto s=parts[i];auto v=color[s.source];
                if(v>=0)vote+=v?-int(s.width):int(s.width);
            }
            color[k]=!forced&&vote>0;
            int sign=color[k]?-1:1;
            for(U i=offset[k];i<offset[k+1];++i) {
                auto s=parts[i];
                if(color[s.source]<0)pending[s.source]+=sign*int(s.width);
            }
        };
        decide(in.mark,true);
        for(U k=0;k<color.size();++k)if(k!=in.mark)decide(k,false);
        for(auto s:parts)if(!color[s.source]&&color[s.target])ingress+=s.width;
    }
    Root apply(const Root& in,std::vector<Node>& grammar) {
        return publish_compact<Gaps>(in,grammar,parts,color);
    }
};
