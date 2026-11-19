// Deterministic run-clustered zero-diagonal Eulerian columns at larger alphabets.
#include <random>
#include <vector>
#define main exhaustive_growth
#include "scan.cpp"
#undef main
int main(){
    std::mt19937 rng(20261119);
    int seen=0,single=0,forced=0,first=-1;
    for(int draw=0;draw<16000;++draw){
        int k=3+rng()%5;
        // A union of short directed cycles is balanced and has no diagonal.
        std::array<std::array<int,8>,8> e{};
        int cycles=2+rng()%6;
        std::vector<int> permutation(k);std::iota(permutation.begin(),permutation.end(),0);
        for(int t=0;t<cycles;++t){
            std::shuffle(permutation.begin(),permutation.end(),rng);
            int len=t?2+rng()%(k-1):k, mass=t?1+rng()%3:1;
            for(int i=0;i<len;++i)e[permutation[i]][permutation[(i+1)%len]]+=mass;
        }
        A a{};int n=0;
        for(int j=0;j<k;++j){
            std::shuffle(permutation.begin(),permutation.end(),rng);
            for(int i:permutation)for(int s=0;s<e[i][j];++s){
                if(n>=int(a.size()))return 3;
                a[n++]=i;
            }
        }
        Table t(n,k,a);assert(!t.loop);
        ++seen;single+=t.primitive;
        if(!t.primitive)continue;
        Results c;c.run(t);
        if(!c.any_loses)continue;
        ++forced;
        if(first<0){
            first=draw;
            std::cout<<"first_draw="<<draw<<" n="<<n<<" k="<<k<<" runs="<<t.r
                     <<" best="<<t.r+c.largest<<" last="<<t.str()<<"\n";
        }
    }
    std::cout<<"draws="<<seen<<" primitive="<<single<<" forced_growth="<<forced<<"\n";
}
