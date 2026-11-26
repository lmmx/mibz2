#include <algorithm>
#include <array>
#include <bit>
#include <cassert>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <limits>
#include <set>
#include <string>
#include <vector>

using U = uint32_t;
using B = uint8_t;
using Blob = std::vector<B>;
constexpr int HEADER = 40;
int bits(U n) { return std::bit_width(n); }
int packed(int n,int w) { return (n*w+7)/8; }
void push32(Blob& b,U v) {
    for(int i=0;i<4;++i) b.push_back((v>>(8*i))&255);
}
U get32(const Blob& b,int at) {
    U v=0;
    for(int i=0;i<4;++i) v |= U(b.at(at+i))<<(8*i);
    return v;
}
void emit(Blob& b,const std::vector<U>& values,int w) {
    uint64_t acc=0; int remain=0;
    for(U v:values) {
        assert(!w ? !v : v<(1u<<w));
        acc |= uint64_t(v)<<remain; remain+=w;
        while(remain>=8){b.push_back(acc&255);acc>>=8;remain-=8;}
    }
    if(remain)b.push_back(acc&255);
}
U item(const Blob& b,int at,int i,int w) {
    if(!w)return 0;
    int bit=i*w,lo=bit%8,first=at+bit/8,nb=(lo+w+7)/8;
    uint64_t value=0;
    for(int k=0;k<nb;++k)value|=uint64_t(b.at(first+k))<<(8*k);
    return (value>>lo)&((1u<<w)-1);
}
int read_mask_size(int q) {return (q+7)/8;}
uint64_t getmask(const Blob& b,int at,int q) {
    uint64_t mask=0;
    for(int i=0;i<read_mask_size(q);++i)mask|=uint64_t(b.at(at+i))<<(8*i);
    return mask;
}
struct Counts {
    long long pages=0, increasing=0, descending=0, adjacent=0, unit=0;
    long long raw=0, constant=0, unit_slope=0, affine=0, exceptional=0, exc=0;
    long long rawpoints=0, predictivepoints=0, zerobit=0;
};
struct Store {
    Blob data,dir;
    int mode=0, page=0, w=0;
    size_t bytes() const {return data.size()+dir.size();}
};
struct Model {
    int kind=0, slope=0, base=0, width=0, k=0, bytes=0;
};
int affine_size(int q,int w,int k,int rb,bool general) {
    return 6 + (general?4:0) + (k?read_mask_size(q):0) + packed(q-k,w)+packed(k,rb);
}
int min_range(const std::vector<int>& v,int drop) {
    int n=v.size();
    if(n<=drop+1)return 0;
    int best=std::numeric_limits<int>::max();
    for(int a=0;a<=drop;++a)best=std::min(best,v[a+n-drop-1]-v[a]);
    return best;
}
int optimistic(const std::vector<U>& y,int rb,const std::vector<int>& adjacent) {
    int q=y.size(), K=std::min(8,q/4);
    std::array<int,9> minspread{};
    for(int lag:{1,2,4,8}) {
        if(lag>=q)continue;
        std::vector<int> v;
        if(lag==1)v=adjacent;
        else {
            for(int i=lag;i<q;++i)v.push_back(int(y[i])-int(y[i-lag]));
            std::sort(v.begin(),v.end());
        }
        for(int k=0;k<=K;++k)minspread[k]=std::max(minspread[k],(min_range(v,2*k)+1)/2);
    }
    int best=1+packed(q,rb);
    for(int k=0;k<=K;++k)
        best=std::min(best,affine_size(q,bits(minspread[k]),k,rb,false));
    return best;
}
Model select(const std::vector<U>& y,int rb,const std::vector<int>& adjacent) {
    int q=y.size(),K=std::min(8,q/4);
    Model best;best.bytes=1+packed(q,rb);
    std::vector<int> slopes={0,1};
    if(q>1){
        slopes.push_back(std::lround(double(int(y.back())-int(y.front()))/(q-1)));
        slopes.push_back(adjacent[adjacent.size()/2]);
    }
    std::sort(slopes.begin(),slopes.end());
    slopes.erase(std::unique(slopes.begin(),slopes.end()),slopes.end());
    for(int s:slopes){
        std::vector<int> v;
        for(int i=0;i<q;++i)v.push_back(int(y[i])-s*i);
        std::sort(v.begin(),v.end());
        for(int k=0;k<=K;++k) {
            // The retained residuals occupy one interval. Try all trim splits for k outliers.
            for(int a=0;a<=k;++a){
                int base=v[a],last=v[a+q-k-1],w=bits(last-base);
                int size=affine_size(q,w,k,rb,s!=0 && s!=1);
                if(size<best.bytes)best={s==0?1:(s==1?2:3),s,base,w,k,size};
            }
        }
    }
    return best;
}
void page(Blob& data,const std::vector<U>& y,Model b,int rb,Counts& c) {
    int q=y.size();
    if(!b.kind) {
        data.push_back(0); emit(data,y,rb); ++c.raw;c.rawpoints+=q;return;
    }
    std::vector<U> residual,exception;
    uint64_t mask=0;
    for(int i=0;i<q;++i) {
        int d=int(y[i])-b.base-b.slope*i;
        if(d<0 || U(d)>=(1u<<b.width)) {mask|=(uint64_t(1)<<i);exception.push_back(y[i]);}
        else residual.push_back(d);
    }
    // Ties and the rounded-to-power-of-two span can bring some counted outliers back in.
    // Retain the selected exact k by marking redundant positions if this happens.
    if(int(exception.size())<b.k) {
        for(int i=0;i<q && std::popcount(mask)<b.k;++i)mask|=uint64_t(1)<<i;
        residual.clear();exception.clear();
        for(int i=0;i<q;++i) {
            if(mask&(uint64_t(1)<<i))exception.push_back(y[i]);
            else residual.push_back(int(y[i])-b.base-b.slope*i);
        }
    }
    assert(std::popcount(mask)==b.k);
    int start=data.size();
    data.push_back(b.kind+(b.k?4:0));data.push_back(b.width);push32(data,U(b.base));
    if(b.kind==3)push32(data,U(b.slope));
    if(b.k)for(int i=0;i<read_mask_size(q);++i)data.push_back((mask>>(8*i))&255);
    emit(data,residual,b.width);emit(data,exception,rb);
    assert(int(data.size())-start==b.bytes);
    c.constant+=b.kind==1;c.unit_slope+=b.kind==2;c.affine+=b.kind==3;
    c.exceptional+=b.k!=0;c.exc+=b.k;c.predictivepoints+=q;c.zerobit+=b.width==0;
}
U successor(const Store& s,int index,int n){
    if(!s.mode)return item(s.data,0,index,s.w);
    int p=index/s.page,i=index%s.page,q=std::min(s.page,n-p*s.page);
    int at=get32(s.dir,4*p),tag=s.data.at(at++);
    if(!tag)return item(s.data,at,i,s.w);
    int kind=tag&3,w=s.data.at(at++),base=int32_t(get32(s.data,at));at+=4;
    int slope=kind==2;
    if(kind==3){slope=int32_t(get32(s.data,at));at+=4;}
    if(tag&4){
        uint64_t mask=getmask(s.data,at,q);at+=read_mask_size(q);
        int earlier=std::popcount(mask&((uint64_t(1)<<i)-1));
        if(mask&(uint64_t(1)<<i))return item(s.data,at+packed(q-std::popcount(mask),w),earlier,s.w);
        return base+slope*i+item(s.data,at,i-earlier,w);
    }
    return base+slope*i+item(s.data,at,i,w);
}
Store labels(const std::vector<U>& code,int cb) {
    Store s;s.mode=1;s.page=16;s.w=cb;
    for(int p=0;p<int(code.size());p+=16) {
        int q=std::min(16,int(code.size())-p);
        std::vector<U> v(code.begin()+p,code.begin()+p+q),u,idx;
        for(U x:v) {
            auto i=std::find(u.begin(),u.end(),x);
            if(i==u.end()) {idx.push_back(u.size());u.push_back(x);}
            else idx.push_back(i-u.begin());
        }
        if(u.size()==1){push32(s.dir,u[0]);continue;}
        push32(s.dir,(1u<<31)|U(s.data.size()));
        int iw=bits(u.size()-1);
        if(packed(u.size(),cb)+packed(q,iw)<packed(q,cb)){
            s.data.push_back(u.size());emit(s.data,u,cb);emit(s.data,idx,iw);
        } else {s.data.push_back(0);emit(s.data,v,cb);}
    }
    if(s.bytes()>=size_t(packed(code.size(),cb))){
        s.mode=0;s.dir.clear();s.data.clear();emit(s.data,code,cb);
    }
    return s;
}
U label(const Store& s,int i){
    if(!s.mode)return item(s.data,0,i,s.w);
    U h=get32(s.dir,4*(i/s.page));
    if(!(h>>31))return h;
    int at=h&0x7fffffff,nu=s.data.at(at++),j=i%s.page;
    if(!nu)return item(s.data,at,j,s.w);
    int rank=item(s.data,at+packed(nu,s.w),j,bits(nu-1));
    return item(s.data,at,rank,s.w);
}
void block(std::array<U,8> h,const std::vector<U>& edge){
    int n=h[2],m=h[3],rb=h[5];
    std::vector<U> y(m),code(m);
    U mask=(1u<<rb)-1;
    for(int i=0;i<m;++i){y[i]=edge[i]&mask;code[i]=edge[i]>>rb;assert(y[i]<U(m));}
    int cb=bits(*std::max_element(code.begin(),code.end()));
    auto cs=labels(code,cb);
    for(int i=0;i<m;++i)assert(label(cs,i)==code[i]);
    int mixed=0;
    for(int p=0;p<m;p+=16){
        bool yes=false; U first=code[p]>>(24-rb);
        for(int i=p;i<std::min(p+16,m);++i){
            assert((code[i]>>(24-rb))<=255);
            yes|=first!=(code[i]>>(24-rb));
        }mixed+=yes;
    }
    int residual24=3*(m+1)+(mixed<32768?2:4)*((m+15)/16)+16*mixed;
    for(int P:{16,32,64}) {
        Store ss;ss.mode=1;ss.page=P;ss.w=rb;
        Counts c;int lower=0;
        for(int p=0;p<m;p+=P) {
            int q=std::min(P,m-p);
            std::vector<U> ys(y.begin()+p,y.begin()+p+q);
            std::vector<int> dif;
            for(int j=1;j<q;++j)dif.push_back(int(ys[j])-int(ys[j-1]));
            bool increasing=std::all_of(dif.begin(),dif.end(),[](int x){return x>0;});
            c.pages++;c.increasing+=increasing;
            c.descending+=std::count_if(dif.begin(),dif.end(),[](int x){return x<0;});
            c.unit+=std::count(dif.begin(),dif.end(),1);c.adjacent+=dif.size();
            std::sort(dif.begin(),dif.end());
            int floor=optimistic(ys,rb,dif);
            Model b=select(ys,rb,dif);
            assert(floor<=b.bytes);
            lower+=4+floor;
            push32(ss.dir,ss.data.size());
            page(ss.data,ys,b,rb,c);
        }
        int raw=packed(m,rb),page_size=ss.bytes();
        if(page_size>=raw) {
            ss.mode=0;ss.dir.clear();ss.data.clear();emit(ss.data,y,rb);
        }
        lower=std::min(lower,raw);
        // Materialize the exact ten-word block header. Section lengths delimit concatenated
        // directories and byte pools; directory lengths follow from the mode, rows and P.
        Blob header;
        for(U v:std::array<U,10>{U(m),h[4],U(n),U(rb),U(cb),U(P),U(ss.mode),U(cs.mode),
                                 U(ss.bytes()),U(cs.bytes())})push32(header,v);
        assert(header.size()==HEADER);
        for(int i=0;i<m;++i)assert(successor(ss,i,get32(header,0))==y[i]);
        auto total=HEADER+ss.bytes()+cs.bytes();
        printf("%u,%u,%d,%d,%d,%d,%d,%d,%d,%zu,%d,%d,%d,%d,%u,%zu,%zu,%zu,%d,%lld,%lld,%lld,%lld,%lld,%lld,%lld,%lld,%lld,%lld,%lld,%lld,%lld\n",
               h[0],h[1],n,m,rb,cb,P,4*m,residual24,cs.bytes(),cs.mode,raw,page_size,lower,
               U(ss.mode),ss.bytes(),total,HEADER+lower+cs.bytes(),HEADER+packed(m,rb)+packed(m,cb),
               c.pages,c.increasing,c.adjacent,c.descending,c.unit,c.raw,c.constant,c.unit_slope,c.affine,
               c.exceptional,c.exc,c.predictivepoints,c.zerobit);
    }
}
void check_floor(){
    int permutations=0;long long trials=0;
    for(int q=1;q<=7;++q){
        std::vector<U> ys;for(int i=0;i<q;++i)ys.push_back(i);
        do {
            std::vector<int> dif;
            for(int i=1;i<q;++i)dif.push_back(int(ys[i])-int(ys[i-1]));
            std::sort(dif.begin(),dif.end());
            int rb=std::max(1,bits(q-1)),floor=optimistic(ys,rb,dif);
            for(int s=-2*q;s<=2*q;++s)for(int mask=0;mask<(1<<q);++mask){
                int k=std::popcount(U(mask));if(k>q/4)continue;
                int lo=INT32_MAX,hi=INT32_MIN;
                for(int i=0;i<q;++i)if(!(mask&(1<<i))){
                    int z=int(ys[i])-s*i;lo=std::min(lo,z);hi=std::max(hi,z);
                }
                assert(floor<=affine_size(q,bits(hi-lo),k,rb,s!=0&&s!=1));++trials;
            }
            ++permutations;
        }while(std::next_permutation(ys.begin(),ys.end()));
    }
    printf("%d permutations checked, %lld page/slope/exception comparisons, all lower bounds valid\n",permutations,trials);
}
int main(int argc,char** argv){
    assert(argc==2 && std::endian::native==std::endian::little);
    if(std::string(argv[1])=="--check"){check_floor();return 0;}
    FILE* f=fopen(argv[1],"rb");assert(f);
    puts("block,stream,n,m,row_bits,code_bits,page,original,residual24,code_bytes,code_paged,global_row_bytes,page_row_bytes,row_floor,row_paged,row_bytes,total_bytes,total_floor,global_both,pages,increasing_pages,adjacent,negative_steps,unit_steps,raw_pages,constant_pages,unit_slope_pages,custom_slope_pages,exception_pages,exceptions,predictive_rows,zero_bit_pages");
    std::array<U,8> h;
    while(size_t got=fread(h.data(),4,8,f)) {
        assert(got==8 && h[3]>0 && h[3]<=900000 && h[5]<=21);
        std::vector<U> edge(h[3]);assert(fread(edge.data(),4,h[3],f)==h[3]);
        assert(fseek(f,long(h[7])*16,SEEK_CUR)==0);
        block(h,edge);
    }
    assert(feof(f));fclose(f);
}
