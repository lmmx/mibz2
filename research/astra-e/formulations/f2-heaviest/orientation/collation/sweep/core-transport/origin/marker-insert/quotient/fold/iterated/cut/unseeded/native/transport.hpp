// Column ordinals and grammar lengths fit the bzip2 block bound in this native test.
enum class Carry { Singleton, Ordinal };
constexpr U absent=UINT32_MAX;
struct Mark {
    U value,skip=0;
    uint64_t moved=0,queries=0,hops=0;
};
template<Carry Mode> struct Transport {
    Mark& mark;
    U source=absent,incoming=absent,predecessor=absent,mark_run=absent,mark_slice=absent;
    U power_entry=absent,power_rank=0,power_new=absent;
    std::vector<U> base;
    bool done=false;

    Transport(const Root& in,Mark& m):mark(m),base(in.bytes.size()+1) {
        U at=0,ordinal=0;
        if constexpr(Mode==Carry::Ordinal) {
            // Split the compulsory histogram exactly once, at the containing run.
            U i=0;
            while(at+in.r[i].n<=mark.value) {
                const auto r=in.r[i++]; base[r.k+1]+=r.n;at+=r.n;
            }
            mark_run=i; const auto r=in.r[i];
            incoming=r.k;
            ordinal=base[incoming+1]+mark.value-at;
            for(;i<in.r.size();++i)base[in.r[i].k+1]+=in.r[i].n;
        } else {
            source=mark.value;
            for(auto [k,n]:in.r) {
                if(k==source){assert(n==1);ordinal=at;}
                base[k+1]+=n; at+=n;
            }
        }
        std::partial_sum(base.begin(),base.end(),base.begin());
        if constexpr(Mode==Carry::Ordinal) {
            source=U(std::upper_bound(base.begin(),base.end(),mark.value)-base.begin()-1);
            predecessor=base[incoming]+ordinal;
            done=predecessor==mark.value;
        } else {done=ordinal==base[source];}
    }
    template<class Classify> U classify(std::vector<Slice>& parts,const std::vector<U>& deleted,Classify& apply) {
        if constexpr(Mode==Carry::Ordinal) {
            auto hit=parts.begin()+mark_slice;
            std::for_each(parts.begin(),hit,apply);
            U local=deleted[source];
            std::for_each(hit,parts.end(),apply);
            return local;
        } else {
            for(auto& s:parts)apply(s);
            return 0;
        }
    }
    U discount(const std::vector<Slice>& parts,const std::vector<U>& into,const std::vector<U>& deleted,
               const std::vector<U>& first,std::vector<U>& rem) {
        U i=0,gone=0;
        if constexpr(Mode==Carry::Ordinal) {
            U owner=into[parts[mark_slice].edge]!=absent?incoming:source;
            for(;i<owner;++i)if(deleted[i]){rem[first[i]]-=deleted[i];gone+=deleted[i];}
        }
        for(;i<deleted.size();++i)if(deleted[i])rem[first[i]]-=deleted[i];
        return gone;
    }
    template<class Colors> U finish(const Root& in,const std::vector<Node>& grammar,const std::vector<Slice>& parts,
             const Colors& color,const std::vector<U>& first,const std::vector<U>& id,const std::vector<U>& into,
             U local,U gone) {
        auto s=parts[mark_slice];
        assert(s.target==source);
        bool moved=into[s.edge]!=absent;
        if(moved){mark.skip+=grammar[in.bytes[s.source]].n;++mark.moved;}
        if constexpr(Mode==Carry::Ordinal) {
            assert(s.source==incoming && moved==(!color[incoming]&&color[source]));
            U z=moved?predecessor:mark.value;
            // Only an unmoved mark in a destination-side bucket can have earlier deletions inside it.
            mark.value=z-gone-(moved?0:local);
        } else {
            assert(s.width==1);
            mark.value=id[moved?s.edge:first[source]];
            assert(mark.value!=absent);
        }
        return mark.value;
    }
    void power_start(const Root& in,const std::vector<U>& b,const std::vector<MappedRun>& raw) {
        if constexpr(Mode==Carry::Ordinal) {
            U y=mark.value,hops=0,queries=0,i=mark_run,advances=0;
            int direction=0;
            while(true) {
                const auto& r=raw[i];++queries;
                assert(r.lo<=y&&y<r.hi);
                if(r.k!=source)break;
                int d=r.delta;assert(d!=0);
                U n=d>0?(r.hi-1-y)/U(d)+1:(y-r.lo)/U(-d)+1;
                int sign=d>0?1:-1;
                assert(direction==0||sign==direction);direction=sign;
                y=U(int(y)+int(n)*d);hops+=n;
                if(sign>0)while(y>=raw[i].hi){++advances;++i;}
                else while(y<raw[i].lo){++advances;--i;}
            }
            assert(queries<=raw.size()&&advances<raw.size()&&b[source]<=y&&y<b[source+1]);
            power_entry=y;mark.queries+=queries;mark.hops+=hops;
        }
        (void)in;
    }
    void power_range(U start,U n) {
        if constexpr(Mode==Carry::Ordinal) {
            if(power_new==absent) {
                if(power_entry>=start&&power_entry<start+n)power_new=power_rank+power_entry-start;
                power_rank+=n;
            }
        }
    }
};
