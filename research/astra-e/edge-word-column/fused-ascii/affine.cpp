#define main old_census_main
#include "../census.cpp"
#undef main
struct G{u start,end,code,k;};
void census(const Block& b){
    const u m=b.m,B=b.band.size();
    const bool fits=U(256+b.nwords)+16383<=(UINT32_MAX>>b.bits);
    if(!fits){std::cout<<b.index<<",0";for(u i=0;i<15;i++)std::cout<<",0";std::cout<<'\n';return;}
    V order(B),kbase(B),q(m),band_of(m);
    std::iota(order.begin(),order.end(),0);
    std::sort(order.begin(),order.end(),[&](u a,u z){return b.band[a].start<b.band[z].start;});
    std::array<u,256> freq{};for(u c:b.thin)freq[c]++;
    u k=0;for(auto& f:freq){u w=f;f=k;k+=w;}
    for(u y=0;y<m;y++)q[freq[b.thin[y]]++]=y;
    k=0;for(u j=0;j<B;j++){kbase[j]=k;k+=b.band[j].width;}
    V simple(B),simple_g;
    std::vector<G> range,affine;u ascii_b=0,ascii_groups=0,arithmetic_b=0,arithmetic_g=0;
    for(u j:order){auto v=b.band[j];u c=std::min(v.code,128u);u lo=v.start,hi=lo+v.width,ordinal=kbase[j];
        std::fill(band_of.begin()+lo,band_of.begin()+hi,j);
        if(c<128){
            ascii_b++;simple[j]=q[ordinal+v.width-1]-q[ordinal]==v.width-1;arithmetic_b+=simple[j];
        }
        if(!range.empty()&&range.back().code==c&&(c==128||range.back().k+lo-range.back().start==ordinal))range.back().end=hi;
        else range.push_back({lo,hi,c,ordinal});
        for(u x=lo;x<hi;x++){
            u d=c==128?0:q[ordinal+x-lo]-x;
            if(!affine.empty()&&affine.back().code==c&&affine.back().k==d)affine.back().end++;
            else affine.push_back({x,x+1,c,d});
        }
    }
    V gid(m),aid(m);
    for(u i=0;i<range.size();i++){auto g=range[i];std::fill(gid.begin()+g.start,gid.begin()+g.end,i);
        bool ok=false;if(g.code<128){ascii_groups++;u n=g.end-g.start;ok=q[g.k+n-1]-q[g.k]==n-1;arithmetic_g+=ok;}
        simple_g.push_back(ok);
    }
    for(u i=0;i<affine.size();i++)std::fill(aid.begin()+affine[i].start,aid.begin()+affine[i].end,i);
    u gathers=0,whole=0,merged=0,claims=0,active_claims=0,affine_claims=0,safe_claims=0,reduced=0;
    for(u j=0;j<B;j++)if(b.band[j].code<128){
        u lo=kbase[j],end=lo+b.band[j].width,last=NO;
        while(lo<end){u id=gid[q[lo]],hi=lo+1;while(hi<end&&gid[q[hi]]==id)hi++;
            claims++;auto g=range[id];u a=lo,z=hi;
            if(g.code<128){
                active_claims++;
                while(a<z&&q[a]==b.mark)a++;
                while(a<z&&q[z-1]==b.mark)z--;
            }
            u used=0;
            for(u i=lo;i<hi;i++){u y=q[i];affine_claims+=aid[y]!=last;last=aid[y];
                if(g.code>=128||y==b.mark)continue;
                assert(y+affine[aid[y]].k==b.next[y]);used++;gathers++;
                whole+=simple[band_of[y]];merged+=simple_g[id];
            }
            if(used){u l=q[a],r=q[z-1];
                if(q[g.k+r-g.start]-q[g.k+l-g.start]==r-l){safe_claims++;reduced+=used-std::min(used,2u);}
            }lo=hi;
        }
    }
    std::cout<<b.index<<",1,"<<ascii_b<<','<<arithmetic_b<<','<<range.size()<<','<<ascii_groups<<','<<arithmetic_g<<','<<gathers<<','<<whole<<','<<merged<<','<<claims<<','<<active_claims<<','<<safe_claims<<','<<reduced<<','<<affine.size()<<','<<affine_claims<<','<<B<<'\n';
}
int main(int argc,char** argv){
    assert(argc==4);std::ifstream edges(argv[1],std::ios::binary),words(argv[2],std::ios::binary),thin(argv[3],std::ios::binary);
    std::cout<<"block,fits,ascii_bands,certified_bands,merged_groups,ascii_groups,certified_groups,pair_gathers,certified_band_mass,certified_group_mass,source_claims,pair_claims,affine_pair_claims,per_claim_loads_avoided,full_affine_groups,full_affine_claims,all_bands\n";
    u n=0;while(true){Block b;if(!next_block(b,edges,words,thin))break;census(b);n++;}assert(n==233);
}
