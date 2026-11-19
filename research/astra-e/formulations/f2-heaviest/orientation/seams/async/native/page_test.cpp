// Independent differential stress at every predecessor hierarchy boundary.
#define ASTRA_RANK_LIBRARY
#include "ranks.cpp"
#include <random>
#include <set>
int main(){
    std::mt19937 rng(430211);
    N commands=0,queries=0,stable=0;
    for(U size:{63u,64u,65u,4095u,4096u,4097u,262143u,262144u,262145u,900000u}){
        Stats stats;
        PagedIndex index(RowOrder{&stats.tree_cmp}); index.init(size,stats);
        std::set<U> gold;std::map<U,Seg*> pointers;
        auto check=[&](U row){
            for(bool strict:{false,true}){
                auto i=strict?index.upper_bound(row):index.lower_bound(row);
                auto j=strict?gold.upper_bound(row):gold.lower_bound(row);
                assert((i==index.end())==(j==gold.end()));
                if(j!=gold.end())assert(i->first==*j && &i->second==pointers.at(*j));
                assert((i==index.begin())==(j==gold.begin()));
                if(j!=gold.begin()){--i;--j;assert(i->first==*j);}
                ++queries;
            }
        };
        for(U i=0;i<6000;++i){
            U random=rng()%size;
            U row=i%3==0?random:(i%3==1?random%std::min(64u,size):size-1-random%std::min(129u,size));
            auto it=index.lower_bound(row);auto j=gold.find(row);
            if(j!=gold.end()){
                auto next=index.erase(it); gold.erase(j);pointers.erase(row);
                auto g=gold.lower_bound(row);
                assert((next==index.end())==(g==gold.end()));
                if(g!=gold.end())assert(next->first==*g);
            }else{
                auto q=index.emplace_hint(it,row,Seg{row,row+1,row,i,nullptr,nullptr});
                gold.insert(row);pointers[row]=&q->second;
            }
            ++commands;
            for(U x:{0u,row,random,size-1,U(rng()%size)})check(x);
            if(i%500==0)for(auto [key,ptr]:pointers){
                assert(ptr->lo==key && ptr==&index.find(key)->second);++stable;
            }
        }
    }
    assert(mem.live==0);
    std::cout<<"mutations="<<commands<<" exact_predecessor_successor_checks="<<queries
             <<" stable_pointer_checks="<<stable<<" rank_limits=63,64,65,4095,4096,4097,262143,262144,262145,900000\n";
}
