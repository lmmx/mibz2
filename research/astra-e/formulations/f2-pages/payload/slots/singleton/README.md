# Width-one queries require no endpoint test

Every first-level directory query returns the unique run containing lo. For w=1, the interval [lo,lo+1) is wholly contained in that same run. Its next-byte test needs only the run byte and its stable LF displacement. A delimiter still causes retirement. At a published branch the same argument applies to a final selected piece with only one row remaining. For w>1 the original boundary test is required.

On the frozen eight-slot preload (6758831), the separate native observer measured 15,325,702 singleton intervals in 19,750,839 actual queries on 228 admitted Wikidata blocks. The 547,404 delimiter hits among them retire; the rest are unary. All 886,075 fully singleton waves had eight live slots. Summing the interval widths over every query gives the full 205,197,794 admitted rows. The census also checked the complete inverse against Segmented on every block. Its bins group widths [1,2), [2,4), ... through [256,infinity). The wave column at index i counts waves with exactly i singleton queries.

This permits an exact 32-bit hot word containing (delta mod 2^20) and the byte. Every legal translated row lies below 900,000, so modular addition with a final 20-bit mask recovers LF(x) for positive or negative displacements. A separate u32 array supplies exclusive endpoints when required. Splitting retains eight bytes per run overall, doubles the number of independently appended vectors, and can reduce the hot working set on singleton-heavy discovery. An ordinary interleaved control can also omit the logical endpoint test. These effects require direct stage pricing.

```sh
d=research/astra-e/formulations/f2-pages/payload/slots/singleton
bash "$d/census.sh" /tmp/astra-e-singleton-census
```
