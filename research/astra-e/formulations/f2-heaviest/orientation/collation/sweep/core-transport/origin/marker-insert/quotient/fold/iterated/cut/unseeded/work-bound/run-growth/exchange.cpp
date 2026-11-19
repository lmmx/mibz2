// Stress the sparsest possible ordinary column: one L run for each type.
#include <random>
#include <vector>
#define main exhaustive_growth
#include "scan.cpp"
#undef main
int main(){
    std::mt19937 rng(880311);
    int zero=0,single=0,forced=0,forced_single=0,best=200;
    for(int trial=0;trial<60000;++trial){
        int k=3+rng()%6;
        A order{},a{};
        std::iota(order.begin(),order.begin()+k,0);
        std::shuffle(order.begin(),order.begin()+k,rng);
        int n=0;
        for(int i=0;i<k;++i){
            int width=1+rng()%10;
            for(int p=0;p<width;++p)a[n++]=order[i];
        }
        Table t(n,k,a);
        if(t.loop)continue;
        ++zero;single+=t.primitive;
        Results c;c.run(t);
        if(!c.any_loses)continue;
        ++forced;forced_single+=t.primitive;
        if(t.primitive&&n<best){
            best=n;
            std::cout<<"primitive draw="<<trial<<" n="<<n<<" k="<<k<<" runs="<<t.r
                     <<" optimum="<<t.r+c.largest<<" last="<<t.str()<<"\n";
        }
    }
    std::cout<<"draws=60000 zero_self="<<zero<<" primitive="<<single<<" forced="<<forced<<" primitive_forced="<<forced_single<<"\n";
}
