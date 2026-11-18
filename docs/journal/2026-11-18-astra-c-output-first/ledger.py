"""Derive per-block operations and paired cache-model totals from recorded counts."""
import argparse
import csv
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent


def rows(file):
    with (HERE / file).open() as f:
        return [{k: (int(v) if v.lstrip("-").isdigit() else v) for k,v in r.items()}
                for r in csv.DictReader(f)]


def work(linked, section):
    for l in linked:
        s = section[l["block"]]
        assert (s["n"],s["p"],s["runs"],s["marked_sha256"]) == \
               (l["n"],l["p"],l["runs"],l["output_sha256"])
        for mode, v in (("linked",l),("section",s)):
            anchored = mode == "section"
            n, r, m = l["n"], l["runs"], l["capped"]
            g = (n+63)//64
            d, q, hits = v["direct_steps"],v["queries"],v["hits"]
            clipped = v["walked"] if anchored else d
            published = v["certificates"] if anchored else v["posted"]
            extra = published if anchored else 0
            nodes = 0 if anchored else v["node_tests"]
            slot_stores = v["posted_rows"] if anchored else v["nodes"]
            assert d + v["copy_bytes"] == n
            yield dict(
                block=l["block"],model=mode,n=n,capped=m,runs=r,pages=g,
                capped_scan_iterations=2*m,histogram_run_updates=r,
                prefix_bucket_pairs=256,scatter_run_updates=r,base_entries_written=r,
                sentinel_writes=1,start_bits_inserted=r,page_records_written=g,
                cleared_index_bytes=8*s["section_size"] if anchored else 4*g,
                queries=q,misses=q-hits,hits=hits,linked_record_reads=nodes,
                direct_steps=d,rank_popcounts=d,base_record_reads=d,boundary_reads=clipped,
                bound_pair_intersections=clipped+extra,published_cylinders=published,
                index_entries_published=slot_stores,chain_head_updates=0 if anchored else slot_stores,
                relocated_nodes=0 if anchored else v["relocated_nodes"],
                # Counts pointer-dependent lookup stages in the serial algorithm.
                # The two adjacent run fields on a miss can be fetched concurrently.
                addressing_stages=2*d+q+nodes,
                hit_row_updates=hits-v["partial_hits"],copy_source_bytes=v["copy_bytes"],
                literal_writes=d,copy_spans=hits,marked_writes=n,raw_output=s["raw_bytes"],
                common_unrle_input=n,common_raw_store=s["raw_bytes"],common_crc_input=s["raw_bytes"],
            )


def pairs(cells, linked, section):
    for base in ("eager","lazy"):
        b = [r for r in cells if r["model"] == base]
        for alt, table in (("linked",linked),("section",section)):
            a = [table[r["block"]] for r in b]
            for label in ("32","256","2048","_share7"):
                def metric(rows, group):
                    return sum(r[group+label] for r in rows)
                cold, new = (metric(v,"fills")+metric(v,"dirty") for v in (b,a))
                old_r, new_r = (metric(v,"read_fills") for v in (b,a))
                ratios = [(v["fills"+label]+v["dirty"+label])/(r["fills"+label]+r["dirty"+label])
                          for v,r in zip(a,b,strict=True)]
                yield dict(base=base,alt=alt,blocks=len(b),cache=label.lstrip("_"),
                    base_transfers=cold,alt_transfers=new,transfer_ratio=round(new/cold,6),
                    min_block_ratio=round(min(ratios),6),max_block_ratio=round(max(ratios),6),
                    base_read_fills=old_r,alt_read_fills=new_r,read_ratio=round(new_r/old_r,6),
                    extra_read_premium=round((cold-new)/(new_r-old_r),6)
                    if new_r>old_r and cold>new else "")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("table",choices=["operations","populations"])
    args = parser.parse_args()
    linked, section, cells = rows("cursor.csv"),rows("section.csv"),rows("cells.csv")
    by_id = lambda items: {r["block"]:r for r in items}
    s = by_id(section)
    archived = rows("../2026-11-18-cells-live-reuse/blocks.csv")
    expected = {(r["block"],m) for r in archived if r["corpus"]=="wikidata.bz2" and r["block"] in s
                for m in ("eager","lazy") if r[m]}
    assert expected == {(r["block"],r["model"]) for r in cells}
    records = {r["block"]:r["records"] for r in archived if r["corpus"]=="wikidata.bz2"}
    assert all(records[r["block"]] == r["capped"] for r in linked)
    sequence = work(linked,s) if args.table=="operations" else pairs(cells,by_id(linked),s)
    output = None
    for row in sequence:
        if output is None:
            output=csv.DictWriter(sys.stdout, fieldnames=row, lineterminator="\n")
            output.writeheader()
        output.writerow(row)


if __name__ == "__main__":
    main()
