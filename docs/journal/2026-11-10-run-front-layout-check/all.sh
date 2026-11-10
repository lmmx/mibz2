cd /tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad
for pad in 0 336 1360 2400; do for t in sc-master sc-head sc-boxed; do laytool/run.sh $t $pad lay/rand1.bz2 > lay/$t-rand-$pad.txt; done; done
for t in sc-master sc-head sc-boxed; do laytool/run.sh $t 0 lay/rec1.bz2 > lay/$t-rec-0.txt; done
echo ALL-DONE
