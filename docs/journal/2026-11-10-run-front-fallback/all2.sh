cd /tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad
for t in sc-master sc-cand; do
  laytool/run.sh $t 0 lay/rand1.bz2 > lay2/$t-rand-0.txt
  laytool/run.sh $t 336 lay/rand1.bz2 > lay2/$t-rand-336.txt
  laytool/run.sh $t 0 lay/rec1.bz2 > lay2/$t-rec-0.txt
  laytool/run2.sh $t lay/rand1.bz2 > lay2/$t-rand-pairs.txt
  laytool/run2.sh $t lay/rec1.bz2 > lay2/$t-rec-pairs.txt
done
laytool/run2.sh sc-head lay/rec1.bz2 > lay2/sc-head-rec-pairs.txt
echo ALL2-DONE
