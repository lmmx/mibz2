// Offline rank of known band starts. No mutable predecessor search is needed.
struct StartRanks{
    Vec<uint64_t> masks;
    Vec<U> prefix;
    explicit StartRanks(U width):masks((width+63)/64),prefix(masks.size()){}
    void add(U x){masks[x>>6]|=N(1)<<(x&63);}
    U finish(){
        U n=0;
        for(U i=0;i<masks.size();++i){prefix[i]=n;n+=std::popcount(masks[i]);}
        return n;
    }
    U rank(U x) const{return prefix[x>>6]+std::popcount(masks[x>>6]&((N(1)<<(x&63))-1));}
};
template<class ConstructorType,class Bands>
bool rank_form(const Edges& e,const ConstructorType& c,const Bands& hits,const SectionRanks& s,
               U n,U p,bool rotate,bool source_sorted,TokenObject& out){
    if(s.s.size()!=1)return false;
    U lo=s.s[0].first,m=s.width;
    out.tokens=m;out.n=n;out.mark=transfer(e,s,p,rotate);
    if(out.mark.rank==nil)return false;
    StartRanks src(source_sorted?0:m),dst(m);
    for(auto b:hits){
        if(b.lo<lo || b.hi>lo+m || b.dest-lo>=m || b.hi-b.lo>m-(b.dest-lo))return false;
        if(!source_sorted)src.add(b.lo-lo);
        dst.add(b.dest-lo);
    }
    if((!source_sorted && src.finish()!=hits.size()) || dst.finish()!=hits.size())return false;
    Vec<U> by_src(source_sorted?0:hits.size()),by_dst(hits.size()),ids(hits.size());
    for(U i=0;i<hits.size();++i){
        auto b=hits[i];if(!source_sorted)by_src[src.rank(b.lo-lo)]=i;
        by_dst[dst.rank(b.dest-lo)]=i;
    }
    int first_byte=-1;
    U consumed=0;
    Triple last{nil,nil,0};
    for(U j=0;j<hits.size();++j){
        U i=source_sorted?j:by_src[j];auto b=hits[i];
        if(b.lo!=lo+consumed)return false;
        if(b.left!=last.left || b.right!=last.right || b.c!=last.c){
            last={b.left,b.right,b.c};
            U base=out.dict.size(),prefix=c.groups[b.left].depth;
            out.dict.resize(base+prefix);
            for(U g=b.left;c.groups[g].parent!=nil;g=c.groups[g].parent)
                out.dict[base+c.groups[g].depth-1]=c.groups[g].c;
            out.dict.push_back(b.c);
            for(U g=b.right;c.groups[g].parent!=nil;g=c.groups[g].parent)out.dict.push_back(c.groups[g].c);
            if(rotate){
                Byte b0=out.dict[base];if(first_byte<0)first_byte=b0;
                if(b0!=first_byte)return false;
                std::rotate(out.dict.begin()+base,out.dict.begin()+base+1,out.dict.end());
            }
            out.words.push_back({base,U(out.dict.size()-base)});out.bases.push_back(b.lo-lo);
        }
        ids[i]=out.words.size()-1;consumed+=b.hi-b.lo;
    }
    if(consumed!=m)return false;
    Vec<U> next=out.bases;U dest=0;
    for(U i:by_dst){
        auto b=hits[i];U id=ids[i],width=b.hi-b.lo;
        if(b.dest!=lo+dest || b.lo!=lo+next[id])return false;
        next[id]+=width;dest+=width;
        if(out.root.empty() || out.root.back().id!=id)out.root.push_back({id,width});
        else out.root.back().width+=width;
    }
    return dest==m;
}
