// Inside Transport. The previous point is consumed before current publication.
static constexpr bool published = Mode==Carry::Prefix || Mode==Carry::Delayed;
bool lazy=false;
U remaining=absent;
void start_publication(U aliases) {
    if constexpr(published) {
        auto& memo=mark.memo;
        assert(!memo.valid);
        memo.base.clear();memo.base.reserve(aliases+1);memo.base.push_back(0);
    }
}
void publish_frequency(U& amount) {
    if constexpr(published) {
        auto& b=mark.memo.base;U start=b.back();
        b.push_back(start+amount);
        if constexpr(Mode==Carry::Prefix)amount=start;
    }
}
void emit(std::vector<Run>& output,U id,U alias,U n,std::vector<U>& frequency) {
    if constexpr(published) {
        if(remaining!=absent) {
            auto& memo=mark.memo;
#ifdef AUDIT_MARK
            ++mark.emitted_prefix;
#endif
            if(n>remaining) {
                bool merge=!output.empty()&&output.back().k==id;
                memo.valid=true;
                memo.incoming=id;
                memo.piece=U(output.size())-U(merge);
                memo.within=remaining+(merge?output.back().n:0);
                if constexpr(Mode==Carry::Prefix)memo.predecessor=frequency[alias]+remaining;
                remaining=absent;
            } else {
                remaining-=n;
                if constexpr(Mode==Carry::Prefix)frequency[alias]+=n;
            }
        }
    }
    append(output,id,n);
}
