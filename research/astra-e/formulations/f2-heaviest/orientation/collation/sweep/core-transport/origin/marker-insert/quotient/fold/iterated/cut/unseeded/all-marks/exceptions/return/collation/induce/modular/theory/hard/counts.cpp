#include "../../modular.hpp"
#include <iostream>
#include <set>
void examine(unsigned q,unsigned m,unsigned n,unsigned b,unsigned mark,const char* family){
    using namespace modular;
    std::vector<Run> runs{{0,n-b,b,1},{n-b,n,0,0}};
    auto t=induce(runs,mark,q);std::vector<unsigned char> out(n);
    auto e=spell(t,out.data(),n);
    unsigned x=mark;for(auto c:out){assert(c==unsigned(x>=b));x=x>=b?x-b:n-b+x;}
    std::vector<bool> live(t.grammar.size());
    std::vector<U> stack{t.terminal};
    while(!stack.empty()){auto a=stack.back();stack.pop_back();if(a<256||live[a-256])continue;
        live[a-256]=true;auto [l,r]=t.grammar[a-256];stack.push_back(l);stack.push_back(r);}
    auto first=t.layers[0];U contracts=0,strict=0,pieces=0;
    for(auto l:t.layers){contracts+=l.strict+l.equal+l.partial+l.self;strict+=l.strict;pieces+=l.pieces;}
    std::cout<<family<<','<<q<<','<<m<<','<<n<<','<<b<<','<<mark<<','<<e.lap<<','<<first.pieces<<','
        <<first.strict<<','<<first.equal<<','<<first.self<<','<<first.partial<<','<<first.final_bands<<','
        <<contracts<<','<<strict<<','<<pieces<<','<<t.grammar.size()<<','<<std::count(live.begin(),live.end(),true)<<'\n';
}
int main(){std::cout<<"family,q,m,n,offset,mark,lap,first_pieces,first_strict,first_equal,first_self,first_partial,first_final_bands,total_contractions,total_strict,total_pieces,all_joins,live_joins\n";
    for(unsigned q:{2,4})for(unsigned m:{2,3,4,5,7,8,16,64,256,1024,4096,16384}){
        examine(q,m,q*m+1,q,0,"primitive_q");
        examine(q,m,q*m,q,0,"periodic_q");
        examine(q,m,q*m,1,0,"primitive_one");
    }
    for(unsigned q:{2,4}){
        unsigned t=1;
        for(unsigned s=1;s<=5;++s){t*=q;examine(q,t,t*t,t,0,"square_periodic");}
    }
}
