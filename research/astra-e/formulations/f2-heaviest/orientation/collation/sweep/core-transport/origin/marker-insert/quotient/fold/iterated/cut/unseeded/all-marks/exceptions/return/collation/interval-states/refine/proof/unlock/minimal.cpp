#include <algorithm>
#include <array>
#include <bit>
#include <cassert>
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <limits>
#include <numeric>
#include <string>

using Mask = unsigned;
constexpr int capacity = 12;
struct Fraction { int num, den; };
bool less(Fraction left, Fraction right) { return left.num*right.den < right.num*left.den; }
struct Summary {
    uint64_t cycles=0, columns=0, potentially_locked=0, any_gap=0, unit_gap=0;
    std::string sample,unit_sample;
    Fraction sample_penalty{};
};
struct Census {
    int n;
    std::array<int,capacity> psi{};
    std::array<Mask,capacity> supports{};
    Mask interior() const {return (1u<<n)-2;}
    void examine(Mask original,Summary& result) const {
        ++result.columns;
        const Mask optional=interior() ^ original;
        Mask unrooted=0;
        for(int t=1;t<n;++t) {
            const Mask bit=1u<<t;
            if((optional&bit) && !(supports[t]&(original|bit))) unrooted |= bit;
        }
        // A first genuinely simultaneous addition requires a rootless nontrivial cycle.
        Mask remaining=unrooted;
        while(remaining) {
            Mask next=0;
            for(Mask scan=remaining;scan;scan&=scan-1) {
                int t=std::countr_zero(scan);
                if(supports[t]&remaining) next |= 1u<<t;
            }
            if(next==remaining) break;
            remaining=next;
        }
        if(!remaining) return;
        ++result.potentially_locked;
        std::array<int,capacity> last{};
        int letter=0;
        for(int t=0;t<n;++t) {
            letter += bool(original&(1u<<t));
            last[psi[t]]=letter;
        }
        std::array<std::array<int,capacity+1>,capacity> segment{};
        for(int lo=0;lo<n;++lo) {
            std::array<int,capacity> histogram{};
            for(int hi=lo+1;hi<=n;++hi)
                segment[lo][hi]=std::max(segment[lo][hi-1],++histogram[last[hi-1]]);
        }
        constexpr int absent=-100;
        std::array<int,capacity> all, singles;
        all.fill(absent);singles.fill(absent);
        for(Mask selected=optional;;selected=(selected-1)&optional) {
            const Mask physical=original|selected;
            bool neutral=true;
            for(Mask scan=selected;scan;scan&=scan-1) {
                if(!(supports[std::countr_zero(scan)]&physical)) {neutral=false;break;}
            }
            if(neutral) {
                int mass=0,lo=0;
                for(Mask scan=physical;scan;scan&=scan-1) {
                    int t=std::countr_zero(scan);mass+=segment[lo][t];lo=t;
                }
                mass+=segment[lo][n];
                const int size=std::popcount(selected);
                all[size]=std::max(all[size],mass);
                Mask known=original;
                for(;;) {
                    Mask old=known;
                    for(Mask scan=selected&~known;scan;scan&=scan-1) {
                        int t=std::countr_zero(scan);
                        if(supports[t]&(known|(1u<<t))) known |= 1u<<t;
                    }
                    if(old==known) break;
                }
                if(known==physical) singles[size]=std::max(singles[size],mass);
            }
            if(!selected) break;
        }
        bool any=false,unit=false;
        Fraction sample{};
        for(int q=1;q<n;++q) {
            if(all[q]<=singles[q]) continue;
            Fraction low{0,1},high{n+1,1};
            for(int t=0;t<n;++t) if(singles[t]!=absent && t!=q) {
                if(t<q) {
                    Fraction upper{all[q]-singles[t],q-t};
                    if(less(upper,high)) high=upper;
                } else {
                    Fraction lower{singles[t]-all[q],t-q};
                    if(less(low,lower)) low=lower;
                }
            }
            if(!less(low,high)) continue;
            any=true;sample={low.num+high.num,low.den+high.den};
            if(less(low,{1,1}) && less({1,1},high)) unit=true;
        }
        result.any_gap+=any;result.unit_gap+=unit;
        if((any && result.sample.empty()) || (unit && result.unit_sample.empty())) {
            std::string column;for(int x=0;x<n;++x) column += char('0'+last[x]);
            if(any && result.sample.empty()) {result.sample=column;result.sample_penalty=sample;}
            if(unit && result.unit_sample.empty()) result.unit_sample=column;
        }
    }
    Summary run() {
        Summary result;
        std::array<int,capacity> order{};std::iota(order.begin(),order.begin()+n,0);
        do {
            ++result.cycles;
            for(int x=0;x<n;++x) psi[order[x]]=order[(x+1)%n];
            Mask mandatory=0;
            for(int t=1;t<n;++t) {
                if(psi[t-1]>psi[t]) {mandatory|=1u<<t;supports[t]=0;}
                else supports[t]=(1u<<(psi[t]+1))-(1u<<(psi[t-1]+1));
            }
            Mask ascents=interior() ^ mandatory;
            for(Mask extras=ascents;;extras=(extras-1)&ascents) {
                examine(mandatory|extras,result);
                if(!extras) break;
            }
        } while(std::next_permutation(order.begin()+1,order.begin()+n));
        return result;
    }
};
int main(int argc,char**argv) {
    if(argc!=3)return 2;
    int lo=std::atoi(argv[1]),hi=std::atoi(argv[2]);
    if(lo<1 || hi>capacity || lo>hi)return 2;
    std::cout<<"n,cycles,primitive_ordered_columns,potentially_locked,strict_gap_at_any_positive_penalty,strict_gap_at_one,first_any_L,first_any_penalty,first_unit_L\n";
    for(int n=lo;n<=hi;++n) {
        auto s=Census{n}.run();
        std::cout<<n<<','<<s.cycles<<','<<s.columns<<','<<s.potentially_locked<<','
                 <<s.any_gap<<','<<s.unit_gap<<','<<s.sample<<',';
        if(!s.sample.empty())std::cout<<s.sample_penalty.num<<'/'<<s.sample_penalty.den;
        std::cout<<','<<s.unit_sample<<std::endl;
    }
}
