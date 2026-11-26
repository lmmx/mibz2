"""Per-cell Chaingrind diagnostics for the two switches. No cross-block pooling."""
import argparse
import csv
from datetime import datetime
import hashlib
import importlib.util
import json
import lzma
from pathlib import Path
import struct


HERE = Path(__file__).resolve().parent
ARMS = ("edge", "scalar", "counter", "both")
WORKERS = (7, 10)
REGISTERED_AT = datetime.fromisoformat("2026-11-19T05:02:55.728084+00:00")
# Canonical output lengths and SHA-256 of warm block 1 plus one measured record.
SAMPLES = {
    1: (900000, "4494c243198cd4d82fc5843e77af4fc0e3be9c86992b3f58ba5ad43c176b565f"),
    85: (899988, "3b6ba01c4917779d3124095ca38f51fc84261a05cda497f703ebf1fdd61728c1"),
    165: (900000, "77088c19c767b5db767d12f79159eb9f67df34b3e0aacbc437c35df37a73412e"),
}
CONTRASTS = (
    ("counter", "edge"), ("both", "scalar"), ("scalar", "edge"),
    ("both", "counter"), ("both", "edge"),
)
# check::write_symbols writes u32 CRC, u32 original row, 256-byte alphabet,
# u16 EOB, u64 max block length and u64 number of u16 Huffman symbols.
HEADER = struct.Struct("<II256sHQQ")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def scorer_and_model(source, manifest):
    for name in ("bench/moonshot/moonshot.py", "bench/moonshot/calibration.json",
                 "bench/moonshot/chaingrind/chaingrind.c", "bench/moonshot/src/check.rs"):
        require(digest((source / name).read_bytes()) == manifest["files"][name],
                f"source fingerprint mismatch for {name}")
    spec = importlib.util.spec_from_file_location("moonshot", source / "bench/moonshot/moonshot.py")
    scorer = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(scorer)
    return scorer, json.loads(scorer.CALIBRATION.read_text())["model"]


def geometry(data, manifest):
    result = {}
    prefix = None
    for block, (_, tape_sha) in SAMPLES.items():
        path = Path("symbols") / f"warm1-measure{block}.sym.xz"
        archive = data / path
        if not archive.exists():
            archive = HERE.parent / "scalar-route-pricing" / path
        raw = lzma.decompress(archive.read_bytes())
        info = manifest["samples"][str(block)]
        require(digest(raw) == tape_sha == info["two_record_sha256"], f"wrong tape {block}")
        first_size = HEADER.size + 2 * HEADER.unpack_from(raw)[-1]
        if block == 1:
            prefix = raw[:first_size]
            require(raw[first_size:] == prefix, "block 1 must appear twice")
        require(raw[:first_size] == prefix, f"wrong warm record for {block}")
        measured = raw[first_size:]
        crc, mark, _, _, capacity, symbols = HEADER.unpack_from(measured)
        require(len(measured) == HEADER.size + 2 * symbols and 0 <= mark < capacity,
                f"invalid measured record geometry for {block}")
        require(digest(measured) == info["measured_single_sha256"], f"wrong measured record {block}")
        result[block] = dict(block_crc32=f"{crc:08x}", original_row=mark,
                             max_block_length=capacity, symbol_count=symbols)
    return result


def expected_config(model, latencies):
    # The wrapper passes latency/issue parameters; the pinned tool supplies the
    # remaining four geometry defaults (ways, store buffer, prefetch, no-RFO).
    return dict(**latencies, l1_lat=model["l1_latency"], l2_lat=model["l2_latency"],
                rob=model["rob"], width=model["width"], mshr=model["mshr"],
                llc_ways=16, sb=56, prefetch=True, no_rfo=True)


def contrast(values):
    """Signed differences in original metric units; undefined ratios remain blank."""
    edge, scalar, counter, both = (values[arm] for arm in ARMS)
    result = dict(values)
    for numerator, denominator in CONTRASTS:
        a, b = values[numerator], values[denominator]
        defined = a is not None and b is not None
        result[f"{numerator}_minus_{denominator}"] = a - b if defined else None
        result[f"{numerator}_over_{denominator}"] = a / b if defined and b > 0 else None
    complete = all(value is not None for value in values.values())
    result["interaction_diff"] = (both - scalar) - (counter - edge) if complete else None
    result["interaction_ratio"] = ((both / scalar) / (counter / edge)
                                   if complete and min(values.values()) > 0 else None)
    return result


def report(source, data):
    manifest = json.loads((data / "manifest.json").read_text())
    require(set(manifest["arms"]) == set(ARMS), "expected all four arm identities")
    require(set(manifest["samples"]) == {str(b) for b in SAMPLES}, "unexpected sample set")
    scorer, model = scorer_and_model(source, manifest)
    inputs = geometry(data, manifest)
    records = [json.loads(line) for line in (data / "runs.jsonl").read_text().splitlines()]
    cells = {(r["workers"], r["block"], r["arm"]): r for r in records}
    expected = {(w, b, a) for w in WORKERS for b in SAMPLES for a in ARMS}
    require(set(cells) == expected and len(records) == len(expected), "missing, repeated or extra cells")
    windows = sorted((datetime.fromisoformat(r["start"]), datetime.fromisoformat(r["finish"]))
                     for r in records)
    require(all(REGISTERED_AT <= start < finish for start, finish in windows),
            "measurement predates registration or has invalid duration")
    require(all(prev[1] <= nxt[0] for prev, nxt in zip(windows, windows[1:])),
            "overlapping measurement windows")
    raw_keys = set(records[0]["counters"]) - {"config"}
    required = {"instructions", "cycles", "l1_miss", "l2_miss", "llc_miss",
                "l2_transfers", "l3_transfers", "store_buffer_stall_cycles",
                "dep_overlap_n", "dep_overlap_sum", "loads", "stores"}
    require(required <= raw_keys, "missing raw counters")

    metrics, comparisons = [], []
    for workers in WORKERS:
        latencies = scorer.nominal_latencies(model, workers)
        config = expected_config(model, latencies)
        for block, (out_bytes, _) in SAMPLES.items():
            per_arm = {}
            for arm in ARMS:
                rec = cells[workers, block, arm]
                counters = rec["counters"]
                context = f"W{workers}, block {block}, {arm}"
                require(rec["latencies"] == latencies and counters["config"] == config,
                        f"wrong modeled geometry for {context}")
                raw = {k: v for k, v in counters.items() if k != "config"}
                require(set(raw) == raw_keys and all(type(v) is int and v >= 0 for v in raw.values()),
                        f"inconsistent raw counter schema for {context}")
                require(raw["out_bytes"] == out_bytes and raw["instructions"] > 0 and raw["cycles"] > 0,
                        f"wrong measured work for {context}")
                require(raw["dep_overlap_n"] <= raw["dep_dram_loads"] <= raw["dram_loads"],
                        f"invalid useful-overlap coverage for {context}")
                measurements = scorer.proxy_metrics(counters)
                # Keep raw event names and denominators alongside the scorer's normalized values.
                require(set(measurements).isdisjoint(raw), "raw/normalized metric name collision")
                measurements.update((k, raw[k]) for k in sorted(raw) if k != "out_bytes")
                per_arm[arm] = measurements
                metrics.append(dict(workers=workers, block=block, arm=arm, out_bytes=out_bytes,
                                    **inputs[block], **latencies, **measurements))
            for metric in per_arm["edge"]:
                values = {arm: per_arm[arm][metric] for arm in ARMS}
                comparisons.append(dict(workers=workers, block=block, metric=metric, **contrast(values)))
    for name, rows in (("metrics.csv", metrics), ("contrasts.csv", comparisons)):
        with (data / name).open("w", newline="") as stream:
            writer = csv.DictWriter(stream, fieldnames=rows[0], lineterminator="\n")
            writer.writeheader()
            writer.writerows(rows)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, help="checkout containing the recorded scorer projection")
    parser.add_argument("data", type=Path, nargs="?", default=HERE)
    args = parser.parse_args()
    report(args.source.resolve(), args.data.resolve())
