"""Check the spacing bound against direct enumeration and rerouted short schedules."""
from itertools import combinations
from random import Random
from schedule import replay, weighted_bound


def brute_spacing(delta, k):
    n = len(delta)
    return max(sum(delta[i] for i in selected)
               for size in range(n+1)
               for selected in combinations(range(n),size)
               if all(y-x>=k for x,y in zip(selected,selected[1:])))


def main():
    rng = Random(651304)
    rosters_changed = 0
    for _ in range(4096):
        n = rng.randrange(1, 10)
        w = rng.randrange(1, 5)
        k = rng.randrange(w, w+5)
        scan = [rng.randrange(1,9) for _ in range(n)]
        work = [rng.randrange(1,21) for _ in range(n)]
        front = [rng.randrange(d+1) for d in work]
        copy = [rng.randrange(9) for _ in range(n)]
        delta = list(map(min,scan,front))
        bound = weighted_bound(delta,k)
        assert bound == brute_spacing(delta,k)
        atomic, roster, *_ = replay(w,k,scan,front,work,copy,False)
        fixed = replay(w,k,scan,front,work,copy,True,roster)[0]
        early, rerouted, *_ = replay(w,k,scan,front,work,copy,True)
        assert 0 <= atomic-fixed <= bound and 0 <= atomic-early <= bound
        rosters_changed += roster != rerouted
    # A slot returned only after each job makes all three discounts collectible.
    s=f=[4,3,7]
    atomic = replay(1,1,s,f,f,[0]*3,False)[0]
    early = replay(1,1,s,f,f,[0]*3,True)[0]
    assert atomic - early == weighted_bound(s,1) == 14
    print(f"4096 schedules and exhaustive spacing sets passed; {rosters_changed} reassigned workers")
    print("one-slot tight witness: 28 -> 14, maximum single discount 7")


if __name__ == "__main__":
    main()
