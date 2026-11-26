#define main census_main
#include "../census.cpp"
#undef main
struct Rank{
    V prefix,base;
    std::vector<U> mask;
    explicit Rank(V start,u m):base(std::move(start)),mask((m+63)/64){
        for(u p:base)mask[p/64]|=U(1)<<(p%64);
        prefix.resize(mask.size());u count=0;
        for(u i=0;i<mask.size();i++){prefix[i]=count;count+=__builtin_popcountll(mask[i]);}
        assert(count==base.size());base.push_back(m);
    }
    u operator()(u x)const{
        return prefix[x/64]+__builtin_popcountll(mask[x/64]&(UINT64_MAX>>(63-x%64)))-1;
    }
};
void choose(const Block& b){
    u B=b.band.size(),m=b.m;V start(B),order(B),q(m),boundary;
    std::vector<U> bits((m+63)/64);V pref(bits.size());
    u k=0;for(u j=0;j<B;j++){start[j]=k;k+=b.band[j].width;u d=b.band[j].start;bits[d/64]|=U(1)<<(d%64);}
    u p=0;for(u i=0;i<bits.size();i++){pref[i]=p;p+=__builtin_popcountll(bits[i]);}
    for(u j=0;j<B;j++){u d=b.band[j].start;order[pref[d/64]+__builtin_popcountll(bits[d/64]&((U(1)<<(d%64))-1))]=j;}
    u prev=NO;
    for(u j:order){const auto& v=b.band[j];u name=b.weight[v.code];
        if(name!=prev)boundary.push_back(v.start);
        prev=name;
    }
    auto insert=[&](u z){if(z==m)return;auto it=std::lower_bound(boundary.begin(),boundary.end(),z);
        if(it==boundary.end()||*it!=z)boundary.insert(it,z);};
    insert(b.mark);insert(b.mark+1);Rank rank(std::move(boundary),m);
    std::array<u,256> cursor{};for(u c:b.thin)cursor[c]++;
    p=0;for(auto& c:cursor){u n=c;c=p;p+=n;}
    for(u y=0;y<m;y++)q[cursor[b.thin[y]]++]=y; // Already in the original worker queues.
    u K=rank.base.size()-1,Q=1;while(Q<K)Q*=2;
    V hist(Q);u reads=0,fragments=0,distinct=0,source_ranges=0,point_ranks=0,fragment_single=0;
    auto readq=[&](u i){reads++;return q[i];};
    std::unordered_map<U,u> audit;
    u last_i=NO,last_j=NO;
    for(u bi:order){const auto& v=b.band[bi];u x=v.start,end=x+v.width;
        while(x<end){
            u i=rank(x),z=std::min(end,rank.base[i+1]);source_ranges++;
            u low=start[bi]+x-v.start,top=low+z-x;
            const u last=readq(top-1);
            while(low<top){
                u y=readq(low),j=rank(y),limit=rank.base[j+1],inside=top;
                point_ranks++;
                if(last>=limit){
                    u good=low,bad=top-1;
                    for(u step=1;step<bad-low;step*=2){
                        u pos=low+step;if(readq(pos)>=limit){bad=pos;break;}good=pos;
                    }
                    while(bad-good>1){u mid=good+(bad-good)/2;if(readq(mid)<limit)good=mid;else bad=mid;}
                    inside=bad;
                }
                assert(inside>low);u w=inside-low;hist[i^j]+=w;audit[(U(i)<<32)|j]+=w;fragments++;fragment_single+=w==1;
                if(i!=last_i||j!=last_j){assert(i!=last_i||j>last_j);distinct++;last_i=i;last_j=j;}
                low=inside;
            }x=z;
        }
    }
    // Independent checking only. Certification and fragment lengths above never read old T.
    V direct(Q);std::unordered_map<U,u> exact;
    for(u x=0;x<m;x++){u i=rank(x),j=rank(b.next[x]);direct[i^j]++;exact[(U(i)<<32)|j]++;}
    assert(direct==hist&&exact.size()==distinct&&audit==exact);
    u mask=0;int score=hist[0];
    for(u w=1;w<Q;w*=2){int delta=0;for(u z=w;z<2*w;z++)delta+=__builtin_parity(mask&z)?-int(hist[z]):int(hist[z]);
        if(delta>0)mask+=w;
        score-=std::abs(delta);
    }
    u mass=(int(m)-score)/4;
    std::cout<<b.index<<','<<m<<','<<B<<','<<K<<','<<source_ranges<<','<<fragments<<','<<fragment_single<<','<<distinct<<','<<reads<<','<<point_ranks<<','<<hist[0]<<','<<mask<<','<<mass<<'\n';
}
int main(int argc,char** argv){
    assert(argc==4);std::ifstream edge(argv[1],std::ios::binary),word(argv[2],std::ios::binary),thin(argv[3],std::ios::binary);
    std::cout<<"block,m,bands,seeded_classes,source_ranges,ordered_fragments,unit_fragments,distinct_pairs,queue_reads,destination_ranks,self_mass,mask,ingress\n";
    u n=0;while(true){Block b;if(!next_block(b,edge,word,thin))break;choose(b);n++;}assert(n==233);
}
