#include "capped-base.inc"
#include "matrix.hpp"
#include "tail.inc"
#include "residual.inc"
#include "journal.inc"

extern "C" const uint8_t* grammar_run(Output* p,const uint16_t* caps,U count,U n,U mark,U arm){
    journal.current=PrefixStamp{arm,n,mark,count,cpu(),0,0};
    if(arm==0){
        auto in=read_caps(caps,count,n,mark);Stats s;
        p->data=decode_with<Timed<StreamFused<false>>>(in,s);
        if(!journal.current.publication)journal.current.publication=cpu();
    }else{
        assert(arm<=2);
        Root next;std::vector<Node> grammar;ByteMatrix::Route route;U rows=0,byte=0;
        {
            ByteMatrix b(caps,count,n,mark);
            journal.current.construct=cpu();route=b.choose();
            if(route==ByteMatrix::Route::cut){next=b.publish();rows=n-b.ingress;grammar=std::move(b.grammar);}
            if(route==ByteMatrix::Route::fixed)byte=b.marked_byte;
            journal.current.publication=cpu();
        }
        if(route==ByteMatrix::Route::fixed)p->data.assign(n,uint8_t(byte));
        else if(route==ByteMatrix::Route::normalize){
            auto in=read_caps(caps,count,n,mark);Stats s;p->data=decode_with<StreamFused<false>>(in,s);
        }else if(arm==1){
            Stats s;p->data=finish_seeded<StreamFused<false>>(std::move(next),std::move(grammar),n,s);
        }else p->data=residual(Prefix{std::move(next),std::move(grammar),n,rows});
    }
    assert(p->data.size()==n&&journal.current.publication>=journal.current.construct&&journal.current.construct>=journal.current.start);
    journal.all.push_back(journal.current);return p->data.data();
}
