// Iterative destination-ordered pure-letter backward search. Source ordinals,
// raw runs, grammar weights and output record definitions are shared by all arms.
struct Alias {
    U k,d; int side;
    bool operator==(const Alias&) const = default;
    bool operator<(const Alias& other) const {
        if(k!=other.k) return k<other.k;
        if(side!=other.side) return side<other.side;
        return side<0? d<other.d : d>other.d;
    }
};
struct MappedRun { U lo,hi,k; int delta; };
struct Pending { int x; U n,d; };
struct TokenRun { Alias key; U width; };
Result interval(const Root& original) {
    auto in=seed(original);auto b=bases(in);auto counts=b;
    std::vector<MappedRun> raw;raw.reserve(in.r.size());
    U pos=0;
    for(auto [k,n]:in.r) {
        raw.push_back({pos,pos+n,k,int(counts[k])-int(pos)});
        counts[k]+=n;pos+=n;
    }
    Result out;
    std::vector<TokenRun> fragments;
    std::vector<Pending> todo;
    U owner=0;
    for(const auto& origin:raw) {
        U y=origin.lo;
        while(y<origin.hi) {
            U n=std::min(b[owner+1],origin.hi)-y;
            U k=origin.k;
            if(k!=owner) {
                int side=owner<k?-1:1;
                todo.push_back({int(y)+origin.delta,n,1});
                while(!todo.empty()) {
                    auto q=todo.back();todo.pop_back();
                    if(q.x<0) {
                        Alias a{k,q.d,side};
                        if(!fragments.empty()&&fragments.back().key==a) fragments.back().width+=q.n;
                        else fragments.push_back({a,q.n});
                        continue;
                    }
                    ++out.queries;
                    U x=U(q.x),end=x+q.n;
                    auto it=std::upper_bound(raw.begin(),raw.end(),x,
                        [](U value,const auto& piece){return value<piece.lo;})-1;
                    auto before=todo.size();
                    while(x<end) {
                        U split=std::min(end,it->hi),w=split-x;
                        if(it->k!=k) todo.push_back({-1,w,q.d});
                        else {
                            int v=it->delta;
                            assert(v!=0);
                            U d=v>0?(it->hi-split)/U(v)+1:(x-it->lo)/U(-v)+1;
                            todo.push_back({int(x)+int(d)*v,w,q.d+d});
                        }
                        x=split;++it;
                    }
                    std::reverse(todo.begin()+before,todo.end());
                }
            }
            y+=n;if(y==b[owner+1])++owner;
        }
    }
    std::vector<Alias> symbols;symbols.reserve(fragments.size());
    for(const auto& part:fragments) symbols.push_back(part.key);
    std::sort(symbols.begin(),symbols.end());
    symbols.erase(std::unique(symbols.begin(),symbols.end()),symbols.end());
    for(U a=0;a<symbols.size();++a) {
        auto s=symbols[a];out.words.push_back({in.bytes[s.k],s.d,s.side});
        if(s.k==in.mark){assert(s.d==1);out.mark=a;}
    }
    for(auto& f:fragments) {
        U id=std::lower_bound(symbols.begin(),symbols.end(),f.key)-symbols.begin();
        append(out.r,id,f.width);
    }
    return out;
}
