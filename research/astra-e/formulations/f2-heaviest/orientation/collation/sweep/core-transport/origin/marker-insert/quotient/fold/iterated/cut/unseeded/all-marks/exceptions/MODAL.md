# Exact scope of a supplied exception clock

For a requested N-byte marked output, begin backwards in the context of Ψ^N(p). At each exceptional time restart at the supplied state; otherwise take its modal predecessor and fixed default transition. A restart state identifies the emitted byte whenever k≥1. For k=0 an exception byte must be supplied separately. When the marked lap length is known, an alternative starts directly at p, reverses one full backward lap and cyclically copies to N. The independent tests cover 343,410 finite marked horizons on arbitrary physical columns, including 175,356 mid-cycle horizons. The construction needs the event order and the endpoint convention as well as the physical context table. [Theorem](PROOF.md) · [Verifier](check.py) · [Exhaustive receipt](clock.json)

The temporal information can itself contain another inverse BWT. Encode a source symbol j by b followed by G+s−1−j copies of a. The expanded binary L column has a direct formula in the old compressed column, and the induced return on its b ranks is exactly the old LF permutation. With G≥k+2, every state predicts a; every exception emits b and enters the same poststate. The times alone give the old symbols in marked order. For fixed positive source counts at prime length m, at least m!/(m product_j m_j!) different clocks share the same model, exception payload sequence, mark and complete cyclic (k+1)-spectrum. On the enumerated 13-symbol balanced family, all 132 possible necklaces produced distinct clocks. The verifier also independently sorted the rotations of 374 encoded marked sources and checked 5,892 induced b-rank returns. [Reduction and cost qualification](CLOCK-REDUCTION.md) · [Source](reduction.py) · [Raw receipt](reduction.json)

Given physical context geometry, a separate construction now acquires these clocks through stable range transport. This includes sparse, long default cycles and arbitrary foreign cycles. [Constructive acquisition](acquire/README.md)

```bash
e=$(dirname "$(rg --files | rg '/all-marks/exceptions/PROOF.md$')")
python3 -B "$e/check.py" | diff - "$e/clock.json"
python3 -B "$e/reduction.py" | diff - "$e/reduction.json"
```
