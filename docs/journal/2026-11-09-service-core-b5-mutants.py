# Hand-made mutants for D045's four invariants (plus the feeder guard): id -> (file, old, new, nth)
# Applied by: python3 mutants.py ID WORKTREE ; each old string must occur (nth occurrence, 1-based)
import sys
M = {
 # wake-up
 "W1": ("src/parallel.rs", "    if got.is_some() {\n        hub.cv.notify_all();\n    }", "    if got.is_some() {\n    }", 1),
 "W2": ("src/parallel.rs", "    if got.is_some() {\n        hub.cv.notify_all();\n    }", "    if got.is_some() {\n        hub.cv.notify_one();\n    }", 1),
 # retirement and buffer return
 "R1": ("src/parallel.rs", "        st.backs.extend(back);\n", "        let _ = back;\n", 1),
 "R2": ("src/parallel.rs", "            publish(shared, place, bit, Err(kind), symbols.syms);\n            hub.retire(Some(front), None);", "            publish(shared, place, bit, Err(kind), symbols.syms);\n            hub.lock().fronts.push(front);", 1),
 "R3": ("src/parallel.rs", "                hub.retire(Some(front), Some(Back::default()));", "                hub.retire(Some(front), None);", 1),
 "R4": ("src/parallel.rs", "            hub.retire(Some(Front::default()), None);", "            hub.retire(None, None);", 1),
 "R5": ("src/parallel.rs", "        return hub.retire(None, Some(back)); // (cancelled)", "        return hub.retire(None, None); // (cancelled)", 1),
 # ownership at helper completion
 "H1": ("src/coop.rs", "        if let Some(Erased(f)) = task {", "        s.done.fetch_add(1, Ordering::AcqRel);\n        if let Some(Erased(f)) = task {", 1),
 "H2": ("src/coop.rs", "        while self.s.done.load(Ordering::Acquire) != self.helpers {", "        while self.s.done.load(Ordering::Acquire) + 1 < self.helpers {", 1),
 "H3": ("src/parallel.rs", "                tt = Default::default();\n                let kind = ErrorKind::Internal(panic_message(p));\n                publish(shared, place, bit, Err(kind), symbols.syms);\n                hub.retire(Some(front), Some(Back::default()));", "                let kind = ErrorKind::Internal(panic_message(p));\n                publish(shared, place, bit, Err(kind), symbols.syms);\n                hub.retire(Some(front), Some(Back::default()));", 1),
 "H4": ("src/parallel.rs", "            back.seg.assemble(asm);\n", "", 1),
 # ordered publication
 "O1": ("src/parallel.rs", "        Ok(Err(kind)) => (Err(kind), back),", "        Ok(Err(_)) => (Ok(out), back),", 1),
 "O2": ("src/parallel.rs", "        Ok(Err(kind)) => {\n            publish(shared, place, bit, Err(kind), symbols.syms);\n            hub.retire(Some(front), None);", "        Ok(Err(kind)) => {\n            publish(shared, place + 1, bit, Err(kind), symbols.syms);\n            hub.retire(Some(front), None);", 1),
 "O3": ("src/parallel.rs", "                        .min_by_key(|&i| st.walked[i].place)", "                        .max_by_key(|&i| st.walked[i].place)", 1),
 "O4": ("src/parallel.rs", "                st.walked.push(Walked {\n                    place,\n                    bit,", "                st.walked.push(Walked {\n                    place,\n                    bit: 0,", 1),
 # the feeder's unwind guard
 "P1": ("src/read.rs", "                let _unwind = CancelOnUnwind(&d, &e);\n", "", 1),
}
if __name__ == "__main__":
    mid, wt = sys.argv[1], sys.argv[2]
    f, old, new, nth = M[mid]
    p = f"{wt}/{f}"
    s = open(p).read()
    assert s.count(old) >= nth, (mid, s.count(old))
    i = -1
    for _ in range(nth):
        i = s.index(old, i + 1)
    s = s[:i] + new + s[i + len(old):]
    open(p, "w").write(s)
    print(f"{mid}: applied to {f}")
