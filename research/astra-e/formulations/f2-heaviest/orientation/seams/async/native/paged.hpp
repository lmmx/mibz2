// Sparse, mutable ordered map on [0,n). Slot addresses survive all inserts.
// One page covers 64 row keys. All summaries are exact dynamic bitsets.
class PagedIndex {
    struct Pair { U first; Seg second; };
    struct Item { Pair kv; U prev=nil,next=nil; };
    struct Page { N bits=0; U head=nil,tail=nil; };
    Vec<Page> pages;
    // layer 0 indexes nonempty pages, layer 1 nonempty words of layer 0, etc.
    std::array<Vec<N>,6> layers;
    U levels=0,free=nil,count=0;
    std::deque<Item,Paid<Item>> items;
    Stats* z=nullptr;
    U first(U level,U x) const{
        U w=x/64;
        if(w>=layers[level].size())return nil;
        ++z->mask_reads;
        N b=layers[level][w] & (~N(0)<<(x%64));
        if(b)return 64*w+std::countr_zero(b);
        if(level+1==levels)return nil;
        w=first(level+1,w+1);
        if(w==nil)return nil;
        ++z->mask_reads;
        return 64*w+std::countr_zero(layers[level][w]);
    }
    U last(U level,U x) const{
        U w=x/64;
        ++z->mask_reads;
        N b=layers[level][w] & (~N(0)>>(63-x%64));
        if(b)return 64*w+63-std::countl_zero(b);
        if(w==0 || level+1==levels)return nil;
        w=last(level+1,w-1);
        if(w==nil)return nil;
        ++z->mask_reads;
        return 64*w+63-std::countl_zero(layers[level][w]);
    }
    void toggle(U page,bool on){
        for(U l=0;l<levels;++l){
            N& bits=layers[l][page/64];N old=bits;
            bits^=N(1)<<(page%64);
            ++z->mask_writes;
            if(on ? old!=0 : bits!=0)return;
            page/=64;
        }
    }
    U slot(U p,U offset) const{
        const Page& page=pages[p];
        U before=std::popcount(page.bits & ((N(1)<<offset)-1));
        U after=std::popcount(page.bits)-1-before;
        bool left=before<=after;
        U id=left?page.head:page.tail, steps=left?before:after;
        z->local_hops+=steps;
        for(U j=0;j<steps;++j)id=left?items[id].next:items[id].prev;
        return id;
    }
    U next(U id) const{
        U j=items[id].next;if(j!=nil)return j;
        U p=first(0,items[id].kv.first/64+1);return p==nil?nil:pages[p].head;
    }
    U prev(U id) const{
        if(id==nil){
            U p=last(0,pages.size()-1);return p==nil?nil:pages[p].tail;
        }
        U j=items[id].prev;if(j!=nil)return j;
        U p=items[id].kv.first/64;
        if(!p)return nil;
        p=last(0,p-1);return p==nil?nil:pages[p].tail;
    }
public:
    struct iterator{
        using iterator_category=std::bidirectional_iterator_tag;
        using value_type=Pair;
        using difference_type=ptrdiff_t;
        using pointer=Pair*;
        using reference=Pair&;
        PagedIndex* m;
        U id;
        bool operator==(const iterator&) const=default;
        Pair* operator->() const{return &m->items[id].kv;}
        Pair& operator*() const{return m->items[id].kv;}
        iterator& operator--(){id=m->prev(id);return *this;}
        iterator& operator++(){id=m->next(id);return *this;}
    };
    explicit PagedIndex(RowOrder){}
    void init(U n,Stats& stats){
        z=&stats;
        pages.resize((n+63)/64);
        U width=pages.size();
        do{
            width=(width+63)/64;
            assert(levels<layers.size());
            layers[levels++].resize(width);
        }while(width>1);
    }
    size_t size() const{return count;}
    bool empty() const{return !count;}
    iterator end(){return {this,nil};}
    iterator begin(){U p=first(0,0);return {this,p==nil?nil:pages[p].head};}
    iterator lower_bound(U row){
        U p=row/64;
        if(p>=pages.size())return end();
        ++z->mask_reads;
        N bits=pages[p].bits & (~N(0)<<(row%64));
        if(bits)return {this,slot(p,std::countr_zero(bits))};
        U q=first(0,p+1);
        return {this,q==nil?nil:pages[q].head};
    }
    iterator upper_bound(U row){return lower_bound(row+1);}
    iterator find(U row){return {this,slot(row/64,row%64)};}
    iterator emplace_hint(iterator right,U row,const Seg& seg){
        U id=free;
        if(id==nil){id=items.size();items.emplace_back();}
        else free=items[id].next;
        auto& x=items[id];x.kv={row,seg};
        U p=row/64;auto& pg=pages[p];
        x.next=right.id!=nil && right->first/64==p?right.id:nil;
        x.prev=x.next!=nil?items[x.next].prev:pg.tail;
        if(x.prev==nil)pg.head=id;else items[x.prev].next=id;
        if(x.next==nil)pg.tail=id;else items[x.next].prev=id;
        if(!pg.bits)toggle(p,true);
        pg.bits|=N(1)<<(row%64);++z->mask_writes;++count;
        return {this,id};
    }
    iterator erase(iterator pos){
        auto& x=items[pos.id];U p=x.kv.first/64;
        U after=next(pos.id);
        auto& pg=pages[p];
        if(x.prev==nil)pg.head=x.next;else items[x.prev].next=x.next;
        if(x.next==nil)pg.tail=x.prev;else items[x.next].prev=x.prev;
        pg.bits^=N(1)<<(x.kv.first%64);++z->mask_writes;
        if(!pg.bits)toggle(p,false);
        x.next=free;free=pos.id;--count;
        return {this,after};
    }
};
