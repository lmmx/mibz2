// Build after the archived reference definitions with build.py.
struct Pruning {
    std::vector<U> allowed;
    uint64_t states = 0, skipped = 0, allowed_initial = 0, interior_extremes = 0;
    uint64_t curve_fragments = 0, active_fragments = 0, modal = 0;
    Pruning(const Scans& s, U charge) {
        std::vector<U> hist(s.labels), touched;
        size_t atom = 0, cut = 0;
        U source_start = 0;
        for (auto stop : s.states) {
            U run_start = source_start, mode = 0;
            std::vector<U> local;
            uint64_t original = 0, fragments = 0;
            while (run_start < stop) {
                auto label = s.atoms[atom].label;
                U end = s.atoms[atom++].end;
                while (end < stop && s.atoms[atom].label == label) {
                    end = s.atoms[atom++].end;
                }
                if (!hist[label]) touched.push_back(label);
                mode = std::max(mode, hist[label] += end-run_start);
                while (cut < s.safe.size() && s.safe[cut] <= run_start) ++cut;
                auto first = cut;
                while (cut < s.safe.size() && s.safe[cut] < end) ++cut;
                original += cut-first;
                fragments++;
                if (cut > first) { local.push_back(s.safe[first]); fragments++; }
                if (cut > first+1) { local.push_back(s.safe[cut-1]); fragments++; }
                if (end < stop && cut < s.safe.size() && s.safe[cut] == end) {
                    local.push_back(end); original++;
                }
                run_start = end;
            }
            bool can_improve = !local.empty() && stop-source_start-mode > charge;
            states++;
            modal += mode;
            allowed_initial += original;
            interior_extremes += local.size();
            curve_fragments += fragments;
            if (can_improve) {
                allowed.insert(allowed.end(), local.begin(), local.end());
                active_fragments += fragments;
            } else {
                skipped++;
            }
            for(auto t:touched) hist[t]=0;
            touched.clear();
            source_start = stop;
        }
        allowed.push_back(s.states.back());
        assert(modal == s.modal);
    }
};

int main(int argc,char** argv) {
    assert(argc==3);
    std::ifstream f(argv[1],std::ios::binary);
    char magic[8];f.read(magic,8);assert(std::string(magic,8)=="CLKMASK1");
    U n=word(f);word(f);
    std::vector<U> l(n);
    for(auto& a:l) a=static_cast<unsigned char>(f.get());
    Source p(std::move(l));
    std::array<U,256> freq{};
    for (auto r:p.runs) freq[r.byte] += r.end-r.begin;
    std::vector<U> initial;
    U pos=0;
    for (auto w:freq) if (w) initial.push_back(pos+=w);
    for(U charge : {1u,4u,16u,64u}) {
        auto parts=initial;
        for (U pass=0;pass<16;pass++) {
            Scans scan(p,parts);
            Cost cost;
            auto base=optimize(scan,charge,cost);
            Pruning stats(scan,charge);
            scan.safe=stats.allowed;
            Cost check_cost;
            auto alternative=optimize(scan,charge,check_cost);
            assert(base.objective==alternative.objective);
            std::cout<<argv[2]<<','<<charge<<','<<pass<<','<<parts.size()<<','<<scan.atoms.size()
                <<','<<stats.allowed_initial<<','<<stats.interior_extremes<<','<<stats.allowed.size()-1
                <<','<<stats.skipped<<','<<cost.fragments<<','<<stats.curve_fragments<<','
                <<stats.active_fragments<<','<<base.objective<<','<<base.ends.size()-parts.size()
                <<','<<alternative.ends.size()-parts.size()<<'\n';
            if (base.ends.size()==parts.size()) break;
            parts=std::move(base.ends);
        }
    }
}
