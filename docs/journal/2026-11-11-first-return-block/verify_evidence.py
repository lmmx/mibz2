"""Verify the retained artifact and accounting identities without rerunning a probe."""

import gzip
import hashlib
import json
from pathlib import Path


def main():
    """Check frozen bytes, the sole attempt, exact output identities and disposition."""
    root = Path(__file__).parent
    evidence = root / "evidence"

    def read(name):
        return json.loads((evidence / name).read_text())

    def digest(data):
        return hashlib.sha256(data).hexdigest()

    freeze = json.loads((root / "FREEZE.json").read_text())
    for name, sha in freeze["source_sha256"].items():
        assert digest((root / name).read_bytes()) == sha, name
    assert digest((evidence / "identities.json").read_bytes()) == freeze["identities_sha256"]
    for name, sha in read("SHA256.json").items():
        assert digest((evidence / name).read_bytes()) == sha, name
    identities = read("identities.json")
    for name, record in identities["artifacts"].items():
        data = gzip.decompress((evidence / (name + ".gz")).read_bytes())
        assert len(data) == record["bytes"] and digest(data) == record["sha256"]
    attempt, supervisor = read("attempt-1/attempt.json"), read("attempt-1/supervisor.json")
    assert attempt["freeze_sha256"] == digest((root / "FREEZE.json").read_bytes())
    assert attempt["attempts"] == 1 and supervisor["retries"] == 0
    assert supervisor["returncode"] == 0 and not supervisor.get("timeout")
    result = read("attempt-1/result.json")
    c, n = result["counts"], identities["n"]
    assert result["complete"] and result["byte_origin_cycle_crc_match"] and result["token_lf"]
    assert c["weighted_rows"] == n == c["selected_cycle_bytes"]
    assert result["traversal_sum"] == (
        c["E"] + c.get("origin_search_steps", 0) + c["residual_token_visits"]
    )
    assert result["traversal_sum"] < n
    assert result["metadata_peak_lower_bound_bytes"] == max(
        result["metadata_live_samples"].values()
    )
    assert result["metadata_peak_lower_bound_bytes"] >= result["metadata_threshold_bytes"] == 4 * n
    assert not result["metadata_peak_exact"] and result["outcome"] == "revise_metadata"
    for key, filename in [
        ("rle1_sha256", "baseline-rle1.bin"),
        ("output_sha256", "baseline-output.bin"),
    ]:
        assert result[key] == identities["artifacts"][filename]["sha256"]
    lengths = result["phrase_length_histogram"]
    assert sum(lengths.values()) == c["dictionary_words"]
    assert sum(int(k) * v for k, v in lengths.items()) == c["dictionary_bytes"]
    copies = result["copy_length_histogram"]
    assert sum(copies.values()) == sum(v for k, v in c.items() if k.endswith("_copy_calls"))
    assert sum(int(k) * v for k, v in copies.items()) == sum(
        v for k, v in c.items() if k.endswith("_copy_bytes")
    )
    assert (
        c["logical_byte_writes"] - n - c["output_bytes"]
        == c["excess_writes_over_one_rle1_and_final_output"]
    )
    print("PASS: frozen sources, artifacts, single attempt and recorded REVISE arithmetic")


if __name__ == "__main__":
    main()
