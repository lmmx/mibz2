// Tests only. This class builds dense stable partners independently from the raw bytes.
struct Dense {
    std::vector<Byte> last,first;
    std::vector<U> lf,psi,state,buffer;
    U depth=0,classes=1;
    explicit Dense(const Input& raw):first(raw.n),lf(raw.n),psi(raw.n),state(raw.n),buffer(raw.n) {
        last.reserve(raw.n);
        for(auto r:raw.runs)last.insert(last.end(),r.width,r.symbol);
        std::array<U,256> c{},base{};
        for(Byte a:last)++c[a];
        std::partial_sum(c.begin(),c.end()-1,base.begin()+1);
        for(U x=0;x<raw.n;++x){Byte a=last[x];U y=base[a]++;lf[x]=y;psi[y]=x;first[y]=a;}
    }
    void advance(U k) {
        assert(k>=depth);
        while(depth<k){
            U partition=0;std::pair<Byte,U> prior{};
            for(U x=0;x<first.size();++x) {
                std::pair<Byte,U> word={first[x],state[psi[x]]};
                if(x){assert(prior<=word);partition+=prior!=word;}
                buffer[x]=partition;prior=word;
            }
            ++depth;state.swap(buffer);classes=partition+1;
        }
    }
    void check(const Input& raw,const Model& m) const {
        assert(m.states.size==classes&&m.primary_state==state[raw.primary]);
        U s=0,x=0,des=0,edges=0,exception_rows=0;
        const auto& e=m.exceptions;
        for(const auto& bucket:m.states.end)for(U end:bucket) {
            assert(end>x&&state[x]==s&&state[end-1]==s);
            if(end<raw.n)assert(state[end]==s+1);
            U best=0;Byte modal=0;
            std::array<U,256> freq{};
            for(U j=x;j<end;++j) {
                Byte a=last[j];U n=++freq[a];edges+=n==1;
                if(n>best||(n==best&&a<modal)){best=n;modal=a;}
            }
            assert(m.modal[s]==modal);
            for(;x<end;++x) {
                const bool exception=last[x]!=modal;
                if(des<e.begin.size()&&e.end[des]<=x)++des;
                const bool covered=des<e.begin.size()&&e.begin[des]<=x&&x<e.end[des];
                assert(exception==covered);
                U to=state[lf[x]];
                if(covered){assert(e.symbol[des]==last[x]&&e.next[des]==to);++exception_rows;}
                else assert(m.next[s]==to);
            }
            ++s;
        }
        assert(x==raw.n&&edges==m.edges&&exception_rows==m.exception_rows);
        for(U i=0;i<e.begin.size();++i) {
            assert(e.begin[i]<e.end[i]);
            if(i){assert(e.end[i-1]<=e.begin[i]);assert(e.end[i-1]!=e.begin[i]||
                    e.symbol[i-1]!=e.symbol[i]||e.next[i-1]!=e.next[i]);}
        }
    }
    // The dense row clock below is an oracle for the conditional writer only.
    bool marked(const Input& in,const Model& m,U horizon) const {
        std::vector<Byte> want(horizon),out(horizon);
        U y=in.primary;for(auto& v:want){v=first[y];y=psi[y];}
        U s=state[y];
        const auto& e=m.exceptions;
        for(U j=horizon;j>0;--j) {
            auto after=std::upper_bound(e.begin.begin(),e.begin.end(),y);
            U at=U(after-e.begin.begin());
            if(at&&y<e.end[at-1]){out[j-1]=e.symbol[at-1];s=e.next[at-1];}
            else{out[j-1]=m.modal[s];s=m.next[s];}
            y=lf[y];assert(s==state[y]);
        }
        return y==in.primary&&out==want;
    }
};
