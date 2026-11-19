#include "xor/stream.hpp"

// All retaining limits and original connectivity are the old chooser's. A
// slice's edge is its *original* neighbor ordinal, computed before coloring.
template<bool Track> struct StreamFused{
    U m=0,self=0,ingress=0;bool done=false;
    std::vector<U> b,starts,degree,deleted;
    std::vector<Slice> parts;std::vector<int8_t> color;
    explicit StreamFused(const Root& in):b(bases(in)),starts(b.size()),degree(in.bytes.size()),deleted(in.bytes.size()),color(in.bytes.size(),-1){
        m=b.back();U pos=0;
        for(auto [k,n]:in.r){if(k==in.mark){assert(n==1);done=(pos==b[k]);break;}pos+=n;}
        if(done)return;
        U j=0;pos=0;parts.reserve(in.r.size()+color.size());
        {
            std::vector<U> last(color.size(),UINT32_MAX);
            for(auto [k,n]:in.r)while(n){
                U w=std::min(n,b[j+1]-pos);
                if(last[k]!=j){last[k]=j;++degree[k];}
                parts.push_back({k,j,w,degree[k]-1});
                if(k==j)self+=w;
                n-=w;pos+=w;if(pos==b[j+1])starts[++j]=parts.size();
            }
        }
        std::vector<int> pending(color.size());
        auto decide=[&](U k,bool forced){
            int vote=pending[k];
            for(U i=starts[k];i<starts[k+1];++i){auto s=parts[i];auto v=color[s.source];if(v>=0)vote+=v?-int(s.width):int(s.width);}
            color[k]=!forced&&vote>0;int sign=color[k]?-1:1;
            for(U i=starts[k];i<starts[k+1];++i){auto s=parts[i];if(color[s.source]<0)pending[s.source]+=sign*int(s.width);}
        };
        decide(in.mark,true);for(U k=0;k<color.size();++k)if(k!=in.mark)decide(k,false);
        for(auto s:parts)if(!color[s.source]&&color[s.target]){ingress+=s.width;deleted[s.target]+=s.width;}
    }
    Root apply(const Root& in,std::vector<Node>& nodes){
        const U K=color.size(),no=UINT32_MAX;
        std::vector<U> first(K+1),used(K);
        for(U k=0;k<K;++k)first[k+1]=first[k]+(color[k]?U(b[k+1]-b[k]>deleted[k]):degree[k]);
        Root out;out.bytes.assign(first.back(),no);out.mark=first[in.mark];out.r.reserve(in.r.size());
        Meter q;if constexpr(Track){
            q.rows=m;q.K=K;q.R=in.r.size();q.H=parts.size();q.live_ids=first.back();
            q.pairs=std::accumulate(degree.begin(),degree.end(),0u);q.target_count=std::count_if(deleted.begin(),deleted.end(),[](U v){return v>0;});
            q.cursors_bytes=(starts.capacity()+used.capacity())*sizeof(U);q.old_slices_bytes=parts.capacity()*sizeof(Slice);
        }
        for(auto s:parts){
            U k=s.source;
            if constexpr(Track)++q.outer;
            if(!color[k]){
                if(!color[s.target])append(out.r,first[k]+s.edge,s.width);
            }else if(!deleted[k]){
                append(out.r,first[k],s.width);
            }else{
                U left=s.width;
                while(left){
                    auto u=parts[starts[k]];assert(u.target==k);
                    U w=std::min(left,u.width-used[k]),v=first[k];
                    if(!color[u.source]){
                        v=first[u.source]+u.edge;out.bytes[v]=k;
                        if constexpr(Track)++q.selected_queries;
                    }
                    append(out.r,v,w);left-=w;used[k]+=w;
                    if constexpr(Track){++q.side;q.side_rows+=w;}
                    if(used[k]==u.width){++starts[k];used[k]=0;}
                }
            }
        }
        for(U k=0;k<K;++k)for(U v=first[k];v<first[k+1];++v){
            U a=in.bytes[k],j=out.bytes[v];
            if(j==no){out.bytes[v]=a;continue;}
            U r=in.bytes[j],n=nodes.size();nodes.push_back({nodes[a].n+nodes[r].n,a,r,Node::Join});out.bytes[v]=n;
            if constexpr(Track)++q.selected;
        }
        if constexpr(Track){q.published=out.r.size();audit.push_back(q);}
        return out;
    }
};
