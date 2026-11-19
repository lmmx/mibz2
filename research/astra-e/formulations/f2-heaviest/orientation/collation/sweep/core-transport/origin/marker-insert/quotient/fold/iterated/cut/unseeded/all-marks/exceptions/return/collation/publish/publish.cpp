#include <bits/stdc++.h>
namespace ctx {
#include "../../../contexts/contexts.hpp"
}
namespace physical {
#include "bands.hpp"
}
namespace literal {
#include "literal.hpp"
}
using U=uint32_t;using Q=uint64_t;using Byte=uint8_t;
constexpr U none=UINT32_MAX;
struct Band {U src,dst,len,dist;};
struct Domain {U start,width,prior,context,post;Byte b;};
struct Phase {U origin=none,skip=0,lap=0;};
struct Tally {U anchored=0,bands=0,domains=0,bytes=0,pieces=0,bound=0,classes=0,runs=0,events=0;
    Q lookup=0,key_comparisons=0,base_comparisons=0;};

physical::Physical anchored_geometry(const ctx::Input& raw,const ctx::Model& m) {
    physical::Physical p(raw.n);
    std::array<U,256> hist{},cursor{};
    U x=0;
    for(auto r:raw.runs){p.runs.push_back({x,r.width,0,r.symbol});hist[r.symbol]+=r.width;x+=r.width;}
    std::partial_sum(hist.begin(),hist.end()-1,cursor.begin()+1);
    for(auto& r:p.runs){r.y=cursor[r.c];cursor[r.c]+=r.w;}
    auto add=[&](U lo,U hi){
        if(!p.u.empty()&&lo<=p.u.back().hi) p.u.back().hi=std::max(hi,p.u.back().hi);
        else p.u.push_back({lo,hi,0});
    };
    auto& e=m.exceptions;
    bool inserted=false;p.true_mark=false;
    for(U t=0;t<e.begin.size();++t) {
        U lo=e.begin[t],hi=e.end[t];
        if(!inserted&&raw.primary<=lo){add(raw.primary,raw.primary+1);inserted=true;}
        if(lo<=raw.primary&&raw.primary<hi){p.true_mark=true;inserted=true;}
        add(lo,hi);
    }
    if(!inserted)add(raw.primary,raw.primary+1);
    x=0;
    for(auto& r:p.u){
        r.base=x;if(r.lo<=raw.primary&&raw.primary<r.hi)p.mark=x+raw.primary-r.lo;
        x+=r.hi-r.lo;
    }
    assert(p.mark!=none);return p;
}

std::vector<Band> remove_primary(const std::vector<physical::Return>& returns,U n,U mark,bool true_mark,Phase& phase) {
    U prev=none,next=none,dout=0;
    for(U b=0;b<returns.size();++b) {
        auto r=returns[b];U width=(b+1==returns.size()?n:returns[b+1].src)-r.src;
        if(r.src<=mark&&mark-r.src<width){next=r.y+mark-r.src;dout=r.dt;}
        if(r.y<=mark&&mark-r.y<width)prev=r.src+mark-r.y;
    }
    assert(prev!=none&&next!=none);
    auto drop=[&](U x){return x-U(!true_mark&&x>mark);};
    if(true_mark)phase={mark,1,0};
    else if(next==mark)phase={none,0,dout};
    else phase={drop(next),dout+1,0};
    std::vector<Band> out;
    for(U b=0;b<returns.size();++b) {
        auto r=returns[b];U width=(b+1==returns.size()?n:returns[b+1].src)-r.src;
        std::array<U,6> cut{0,width,width,width,width,width};
        if(!true_mark) {
            if(r.src<=mark&&mark-r.src<width){cut[2]=mark-r.src;cut[3]=cut[2]+1;}
            if(r.y<=mark&&mark-r.y<width){cut[4]=mark-r.y;cut[5]=cut[4]+1;}
            std::sort(cut.begin(),cut.end());
        }
        for(U h=0;h+1<cut.size();++h) {
            U w=cut[h+1]-cut[h];if(!w)continue;
            U x=r.src+cut[h],y=r.y+cut[h],dt=r.dt;
            if(!true_mark&&x==mark){assert(w==1);continue;}
            if(!true_mark&&y==mark){assert(x==prev&&next!=mark&&w==1);y=next;dt+=dout;}
            x=drop(x);y=drop(y);
            if(!out.empty()&&out.back().src+out.back().len==x&&out.back().dst+out.back().len==y&&out.back().dist==dt)
                out.back().len+=w;
            else out.push_back({x,y,w,dt});
        }
    }
    return out;
}

struct Descriptors {
    std::vector<Domain> full,byte;
    std::array<U,256> base{};
    explicit Descriptors(const ctx::Model& m) {
        const auto& e=m.exceptions;std::array<U,256> hist{};
        U left=0,state=0,ix=0,rank=0;
        for(const auto& bucket:m.states.end)for(U right:bucket) {
            while(ix<e.begin.size()&&e.begin[ix]<right) {
                U lo=std::max(left,e.begin[ix]),hi=std::min(right,e.end[ix]),w=hi-lo;
                Byte b=e.symbol[ix];assert(lo<hi);
                Domain d{rank,w,hist[b],state,e.next[ix],b};
                if(!full.empty()&&full.back().b==b&&full.back().context==state){
                    assert(full.back().post==d.post);full.back().width+=w;
                }else full.push_back(d);
                if(!byte.empty()&&byte.back().b==b)byte.back().width+=w;
                else byte.push_back(d);
                rank+=w;hist[b]+=w;
                if(e.end[ix]>right)break;
                ++ix;
            }
            left=right;++state;
        }
        assert(ix==e.begin.size()&&rank==m.exception_rows);
        std::partial_sum(hist.begin(),hist.end()-1,base.begin()+1);
    }
    U shifted(U x,const Domain& d)const {return base[d.b]+d.prior+x-d.start;}
};

struct Key {U dist,context;Byte b_src,b_dst;auto operator<=>(const Key&)const=default;};
struct Class {Key key;U post,base,width;};
struct Run {U id,width;};
struct Select {U prefix,pos;};
struct Column {
    std::vector<Class> classes;
    std::vector<Run> column;
    std::vector<U> order;
    std::vector<std::vector<Select>> select;
    Phase phase;
    U owner(U x)const {
        auto it=std::upper_bound(order.begin(),order.end(),x,[&](U v,U id){return v<classes[id].base;});
        assert(it!=order.begin());return *std::prev(it);
    }
    U forward(U x,U id)const {
        auto& ranges=select[id];U ix=x-classes[id].base;
        auto it=std::upper_bound(ranges.begin(),ranges.end(),ix,[](U v,Select s){return v<s.prefix;});
        assert(it!=ranges.begin());--it;return it->pos+ix-it->prefix;
    }
};
void append(std::vector<Run>& out,U id,U w) {
    if(!out.empty()&&out.back().id==id)out.back().width+=w;
    else out.push_back({id,w});
}
Column publish(const std::vector<Band>& bands,const Descriptors& ds,Phase phase,Tally& cost) {
    Column out;out.phase=phase;
    if(bands.empty()){assert(phase.origin==none);return out;}
    std::array<std::vector<Run>,256> bins;
    auto compare_keys=[&](const Key& a,const Key& b){++cost.key_comparisons;return a<b;};
    std::map<Key,U,decltype(compare_keys)> ids(compare_keys);
    U si=0;
    for(auto band:bands) {
        auto dit=std::upper_bound(ds.byte.begin(),ds.byte.end(),band.dst,[&](U v,const Domain& d){++cost.lookup;return v<d.start;});
        assert(dit!=ds.byte.begin());--dit;
        for(U h=0;h<band.len;) {
            U x=band.src+h,y=band.dst+h;
            if(x==ds.full[si].start+ds.full[si].width)++si;
            if(y==dit->start+dit->width)++dit;
            const auto& s=ds.full[si];const auto& d=*dit;
            U w=std::min({band.len-h,s.start+s.width-x,d.start+d.width-y});
            assert(w&&s.start<=x&&d.start<=y);
            U j=ds.shifted(y,d);
            Key key{band.dist,s.context,d.b,s.b};
            auto [it,fresh]=ids.try_emplace(key,out.classes.size());
            U id=it->second;
            if(fresh)out.classes.push_back({key,s.post,j,0});
            auto& c=out.classes[id];assert(c.post==s.post);
            c.base=std::min(c.base,j);c.width+=w;
            append(bins[s.b],id,w);
            if(phase.origin!=none&&y<=phase.origin&&phase.origin-y<w)
                out.phase.origin=j+phase.origin-y;
            ++cost.pieces;h+=w;
        }
    }
    cost.domains=ds.full.size();cost.bytes=ds.byte.size();cost.bands=bands.size();
    cost.bound=cost.bands+cost.domains+cost.bytes-2;
    assert(cost.pieces<=cost.bound);
    for(auto& bin:bins)for(auto r:bin)append(out.column,r.id,r.width);
    out.order.resize(out.classes.size());std::iota(out.order.begin(),out.order.end(),0);
    std::sort(out.order.begin(),out.order.end(),[&](U a,U b){++cost.base_comparisons;return out.classes[a].base<out.classes[b].base;});
    U x=0;for(U i:out.order){assert(out.classes[i].base==x);x+=out.classes[i].width;}
    cost.events=x;cost.classes=out.classes.size();cost.runs=out.column.size();
    out.select.resize(out.classes.size());std::vector<U> before(out.classes.size());
    x=0;
    for(auto [id,w]:out.column){out.select[id].push_back({before[id],x});before[id]+=w;x+=w;}
    for(U i=0;i<out.classes.size();++i)assert(before[i]==out.classes[i].width);
    return out;
}

void spell(const ctx::Model& m,const Class& c,U skip,U count,Byte* dest) {
    assert(skip+count<=c.key.dist);
    if(!count)return;
    if(!skip)dest[0]=c.key.b_src;
    U state=c.post;
    for(U p=c.key.dist-1;p>=std::max(1u,skip);--p) {
        if(p<skip+count)dest[p-skip]=m.modal[state];
        state=m.next[state];
    }
}
std::vector<Byte> emit(const ctx::Input& raw,const ctx::Model& m,const Column& c) {
    U n=raw.n;std::vector<Byte> out(n);auto p=c.phase;
    if(p.origin==none) {
        assert(p.lap&&p.lap<=n);U state=m.primary_state;
        for(U i=p.lap;i--;) {out[i]=m.modal[state];state=m.next[state];}
        for(U i=p.lap;i<n;++i)out[i]=out[i-p.lap];
    }else {
        U done=0,row=p.origin,skip=p.skip;
        while(done<n) {
            U id=c.owner(row);auto& cl=c.classes[id];assert(skip<=cl.key.dist);
            U take=std::min(cl.key.dist-skip,n-done);
            spell(m,cl,skip,take,out.data()+done);done+=take;skip=0;
            if(done<n)row=c.forward(row,id);
        }
    }
    return out;
}

struct Check {
    std::vector<Byte> last,first;
    std::vector<U> psi;
    explicit Check(const ctx::Input& raw) {
        for(auto r:raw.runs)last.insert(last.end(),r.width,r.symbol);
        first=last;std::sort(first.begin(),first.end());psi.resize(raw.n);
        std::array<U,256> count{},cur{};for(Byte b:last)++count[b];
        std::partial_sum(count.begin(),count.end()-1,cur.begin()+1);
        for(U x=0;x<raw.n;++x)psi[cur[last[x]]++]=x;
    }
    void column(const ctx::Model& model,const Column& c)const {
        U index=0;
        for(auto [id,w]:c.column)for(U h=0;h<w;++h) {
            assert(c.classes[id].base==literal::expected_base[literal::expected_l[index++]]);
        }
        assert(index==literal::expected_l.size()&&c.classes.size()==literal::expected_base.size());
        for(U old=0;old<literal::expected_base.size();++old) {
            U x=literal::expected_base[old],id=c.owner(x);
            auto& cl=c.classes[id];assert(cl.base==x);
            std::vector<Byte> word(cl.key.dist);spell(model,cl,0,word.size(),word.data());
            auto& ref=literal::expected_words[old];assert(ref.size()==word.size());
            assert(!std::memcmp(word.data(),ref.data(),word.size()));
        }
        for(U x=0;x<index;++x)assert(c.forward(x,c.owner(x))==literal::expected_psi[x]);
    }
    void marked(U p,const std::vector<Byte>& out)const {
        for(Byte b:out){assert(b==first[p]);p=psi[p];}
    }
};
struct Report { Tally t;Phase phase;U event_mark;U states,source_ranges;Q hops,bytes; };
void save_column(const Column& c,const char* path) {
    std::ofstream f(path,std::ios::binary);f.write("EXCCOL01",8);
    auto u32=[&](U value){for(U j=0;j<4;++j){f.put(char(value&255));value>>=8;}};
    u32(c.classes.size());u32(c.column.size());u32(c.phase.origin);u32(c.phase.skip);u32(c.phase.lap);
    for(U id:c.order) {
        const auto& v=c.classes[id];u32(v.base);u32(v.width);u32(v.key.dist);u32(v.key.context);
        f.put(v.key.b_src);f.put(v.key.b_dst);u32(v.post);
    }
    for(auto r:c.column){u32(c.classes[r.id].base);u32(r.width);}
    assert(f);
}
Report trial(const ctx::Input& raw,U k,const Check& check,const char* object=nullptr,const char* expected=nullptr) {
    ctx::Charge cc;auto model=ctx::construct(raw,k,cc);
    auto g=anchored_geometry(raw,model);
    physical::Cost cost;physical::Map map(g,true,cost);
    auto anchored=physical::induce(g,map,cost);
    Tally t;t.anchored=anchored.size();
    Phase phase;auto bands=remove_primary(anchored,cost.anchors,g.mark,g.true_mark,phase);
    Descriptors desc(model);
    auto result=publish(bands,desc,phase,t);
    assert(t.events==model.exception_rows);
    check.column(model,result);
    auto bytes=emit(raw,model,result);check.marked(raw.primary,bytes);
    if(object)save_column(result,object);
    if(expected) {
        std::ifstream f(expected,std::ios::binary);std::vector<Byte> want(raw.n);
        f.read((char*)want.data(),raw.n);assert(f&&f.peek()==EOF&&want==bytes);
    }
    return {t,result.phase,g.true_mark,model.states.size,U(model.exceptions.begin.size()),cost.hops,bytes.size()};
}
int main(int argc,char** argv) {
    if(argc==1) {
        Q models=0,marks=0,absent=0,normal=0,artificial=0,covered_artificial=0,inside=0,at_end=0,none_other=0;
        for(U n=1,ways=3;n<=7;++n,ways*=3)for(U seq=0;seq<ways;++seq) {
            U val=seq;ctx::Input raw;
            for(U i=0;i<n;++i){Byte b=val%3;val/=3;if(!raw.runs.empty()&&raw.runs.back().symbol==b)++raw.runs.back().width;else raw.runs.push_back({1,b});}
            raw.n=n;Check check(raw);
            for(U k:{1u,2u,3u}) {
                ++models;literal::evaluate(check.last,k,nullptr,0,false);
                for(U mark=0;mark<n;++mark) {
                    raw.primary=mark;auto r=trial(raw,k,check);++marks;
                    absent+=r.t.events==0;normal+=r.event_mark;artificial+=!r.event_mark;
                    none_other+=r.phase.origin==none&&r.t.events>0;
                    if(r.phase.origin!=none){
                        covered_artificial+=!r.event_mark;
                        U old=std::upper_bound(literal::expected_base.begin(),literal::expected_base.end(),r.phase.origin)-literal::expected_base.begin()-1;
                        U d=literal::expected_words[old].size();assert(r.phase.skip<=d);at_end+=r.phase.skip==d;inside+=r.phase.skip<d;
                    }
                }
            }
        }
        std::cout<<"models,marks,empty,primary_event,artificial,covered_artificial,phase_interior,phase_right_boundary,foreign_only\n"
                 <<models<<','<<marks<<','<<absent<<','<<normal<<','<<artificial<<','<<covered_artificial<<','<<inside<<','<<at_end<<','<<none_other<<'\n';
        return 0;
    }
    assert(argc==6);auto raw=ctx::read_input(argv[1]);U k=std::stoul(argv[2]);
    std::ifstream f(argv[3],std::ios::binary);char magic[8];f.read(magic,8);
    assert(std::string(magic,8)=="CLKMASK1"&&literal::word(f)==raw.n&&literal::word(f)==raw.primary);
    std::vector<Byte> last(raw.n),mask(raw.n);f.read((char*)last.data(),raw.n);f.read((char*)mask.data(),raw.n);
    assert(f&&f.peek()==EOF);Check check(raw);assert(check.last==last);
    auto oracle=literal::evaluate(last,k,&mask,raw.primary,false);
    auto r=trial(raw,k,check,argv[4],argv[5]);auto t=r.t;
    assert(t.events==oracle.events&&t.classes==oracle.classes&&t.runs==oracle.runs);
    std::cout<<raw.n<<','<<raw.primary<<','<<r.states<<','<<r.source_ranges<<','<<t.events<<','<<t.anchored<<','<<t.bands<<','<<t.domains<<','<<t.bytes
             <<','<<t.pieces<<','<<t.bound<<','<<t.classes<<','<<t.runs<<','<<t.lookup<<','<<r.hops<<','<<r.event_mark<<','<<r.phase.origin<<','<<r.phase.skip<<','<<r.bytes<<','<<t.key_comparisons<<','<<t.base_comparisons<<'\n';
}
