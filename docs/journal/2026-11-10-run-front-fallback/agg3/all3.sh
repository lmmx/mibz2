W=/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad; mkdir -p $W/lay3
for t in sc-master sc-head sc-boxed sc-cand; do
  while [ "$(pgrep -c rustc)" != "0" ]; do sleep 10; done
  (cd $W/$t && cargo +1.99.1 build -j2 --release --example ab 2>&1 | tail -1)
  nm -S -C $W/$t/target/release/examples/ab | grep -E ' mibz2::(mtf::(mtf_avx2|STEPS)|block::read_item|bwt::build)$' > $W/lay3/$t-syms.txt
  $W/laytool3/run3.sh $t 0 $W/lay/rand1.bz2 > $W/lay3/$t-rand-0.txt
  $W/laytool3/run3.sh $t 336 $W/lay/rand1.bz2 > $W/lay3/$t-rand-336.txt
  $W/laytool3/run3.sh $t 0 $W/lay/rec1.bz2 > $W/lay3/$t-rec-0.txt
  rm -rf $W/$t/target
done
echo ALL3-DONE
