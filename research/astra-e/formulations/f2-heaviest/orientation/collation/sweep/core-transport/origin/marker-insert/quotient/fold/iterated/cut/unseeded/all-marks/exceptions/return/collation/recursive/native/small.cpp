#include "terminal.cpp"
Q marked=0,foreign=0,nondivisor=0,right_boundary=0;
void check(const std::vector<U>& labels,bool all) {
    Input a;
    std::vector<U> hist(1+*std::max_element(labels.begin(),labels.end()));for(U id:labels)++hist[id];
    std::vector<U> base(hist.size()),cursor(hist.size());
    std::vector<std::vector<Byte>> literal;
    for(U i=0;i<7;++i)a.defaults.push_back((((i+1)%7)<<8)|(17+i));
    U total=0;
    for(U i=0;i<hist.size();++i)if(hist[i]){
        U w=i%3==0?1:(i%3==1?9:18);base[i]=cursor[i]=a.pop;
        a.rules.push_back({a.pop,w,i%7,Byte(193+i%20)});a.pop+=hist[i];total+=hist[i]*w;
        std::vector<Byte> word;U s=i%7;
        for(U j=1;j<w;++j){U e=a.defaults[s];word.push_back(Byte(e));s=e>>8;}
        std::reverse(word.begin(),word.end());word.insert(word.begin(),Byte(193+i%20));literal.push_back(word);
    }
    a.n=total;
    std::vector<U> psi(a.pop),which(a.pop);
    for(U i=0;i<a.rules.size();++i){
        U end=i+1<a.rules.size()?a.rules[i+1].base:a.pop;
        std::fill(which.begin()+a.rules[i].base,which.begin()+end,i);
    }
    for(U j=0;j<labels.size();++j){
        U id=labels[j];psi[cursor[id]++]=j;
        if(!a.incoming.empty()&&a.incoming.back().base==base[id])++a.incoming.back().width;
        else a.incoming.push_back({base[id],1});
    }
    build_bank(a,a.bank);jumps(a);Workspace w;
    std::vector<U> marks;
    if(all){marks.resize(a.pop);std::iota(marks.begin(),marks.end(),0);}
    else marks={0,a.pop-1};
    for(U origin:marks){
        U x=origin;std::vector<Byte> lap;U visits=0;
        do{auto& word=literal[which[x]];lap.insert(lap.end(),word.begin(),word.end());x=psi[x];++visits;}while(x!=origin);
        a.origin=origin;
        for(U skip=0;skip<=literal[which[origin]].size();++skip){
            a.skip=skip;std::vector<Byte> want(a.n);
            for(U j=0;j<a.n;++j)want[j]=lap[(skip+j)%lap.size()];
            for(U arm=0;arm<3;++arm){Stamp stamp{};compute(a,w,arm,stamp);assert(w.out==want&&stamp.visits==visits&&stamp.literal==lap.size());}
            ++marked;foreign+=visits<a.pop;nondivisor+=a.n%lap.size()!=0;right_boundary+=skip==literal[which[origin]].size();
        }
    }
}
int main(){
    for(U n=1,ways=3;n<=6;++n,ways*=3)for(U code=0;code<ways;++code){
        std::vector<U> l(n);U x=code;for(auto& v:l){v=x%3;x/=3;}check(l,true);
    }
    for(U n:{65535u,65536u,65537u}){std::vector<U> l(n);std::iota(l.begin(),l.end(),0);check(l,false);}
    for(U n:{1u,8u,16u,19u}){
        Input a;a.defaults={1u<<8|42u,2u<<8|43u,44u};a.empty_lap=n>=3?3:1;a.n=n;
        if(n==1)a.defaults[0]=42;
        a.start=0;build_bank(a,a.bank);jumps(a);Workspace w;
        for(U arm=0;arm<3;++arm){Stamp t{};compute(a,w,arm,t);assert(!t.visits);
            for(U i=0;i<a.n;++i)assert(w.out[i]==(n==1?42:44-i%3));}
    }
    std::cout<<"marked,foreign,nondivisor,right_boundary,default_only,wide_boundaries\n"
             <<marked<<','<<foreign<<','<<nondivisor<<','<<right_boundary<<",4,3\n";
}
