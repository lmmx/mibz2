#include "arc.hpp"
#include <iostream>
using namespace arc_cut;
int main(){
 for(U n=2;n<=14;++n)for(U bits=1;bits+1<(1u<<n);++bits){
  std::vector<Run> rs;std::vector<U> f(n),l(n),lf(n);
  U zeros=n-__builtin_popcount(bits),z=0,o=zeros;
  for(U j=0;j<n;++j){l[j]=(bits>>j)&1;lf[j]=l[j]?o++:z++;
   if(!j||l[j]!=l[j-1])rs.push_back({j,j+1,lf[j],l[j]});else++rs.back().end;}
  U u=lf[0],lap=1;while(u){++lap;u=lf[u];}if(lap!=n)continue;
  for(U p=0;p<n;++p){
   auto e=isolate(initial(rs),p);
   Grammar g;arc_cut::Layer s;U mark=p,terminal=none;auto f=powers(e,mark,g,s,terminal);
   if(terminal!=none)continue;
   auto k=cut(f,mark,g,s);
   if(k.arcs.size()>f.arcs.size()){
    for(U v:l)std::cout<<v;
    std::cout<<" "<<p<<" "<<n<<" "<<e.arcs.size()<<" "<<f.arcs.size()<<" "<<s.pairs<<" "<<s.triples<<" "<<s.cut<<" "<<k.arcs.size()<<"\n";return 0;
   }
  }
 }
 std::cout<<"none\n";
}
