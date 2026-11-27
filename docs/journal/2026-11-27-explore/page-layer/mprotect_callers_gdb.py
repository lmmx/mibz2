import gdb, collections
cnt = collections.Counter(); n=[0]; sizes=collections.Counter()
class BP(gdb.Breakpoint):
    def stop(self):
        n[0]+=1
        if n[0] % 5: return False
        f = gdb.newest_frame(); names=[]
        while f is not None and len(names) < 30:
            names.append((f.name() or '?')[:100]); f = f.older()
        k=[i for i,x in enumerate(names) if 'decode_owned' in x or 'worker' in x]
        if k: cnt[tuple(names[max(0,k[0]-4):k[0]+1])] += 1
        else: cnt[('other',)+tuple(names[4:7])]+=1
        return False
BP("__mprotect")
gdb.events.exited.connect(lambda e: (print("TOTAL", n[0]), [print(v, '\n    '+'\n    '.join(k)) for k, v in cnt.most_common(8)]))
gdb.execute("run 4 < /tmp/claude-0/wd4.bz2 > /dev/null")
