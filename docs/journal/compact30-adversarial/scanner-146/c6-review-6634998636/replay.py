"""Static listing and budget replay only; executes no constructor."""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
def trace(name, blob, end, skip):
    data = (ROOT / name).read_bytes()
    actual = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
    assert actual == blob, (name, actual)
    rows = [(int(m[1], 16), m[2]) for m in re.finditer(r"^\s+([0-9a-f]+):\s+(.+)$", data.decode(), re.M)]
    return [hex(a) for a, _ in rows if a <= end and a not in skip]

d = trace("descriptor-refill.asm", "60ad7562e7dd563f018a83daf84d0d5b1af3da97", 0x94442, {0x943f4, 0x943f9, 0x943fd})
b = trace("b2-refill.asm", "1f3b4b5a91e889c344a18ee914b3f2d156febbb6", 0x8a8bf, {0x8a884, 0x8a889, 0x8a88d})
assert (len(d), len(b)) == (69, 62)
# Exposure inputs are transcribed from pinned proposal results, not replayed
# from the primary binary archive in this review.
E, P, B, R, scalar = 1084913, 1795482, 434, 610521, 38294883
F = P - E
all_refills_upper = P + 256 * B + R
budget = (len(d) - 40) * E - 5 * F
out = {
    "descriptor_trace": d, "b2_trace": b,
    "descriptor_count": len(d), "b2_count": len(b),
    "eligible_lower_input": E, "failed_guard_upper": F,
    "all_refills_upper": all_refills_upper,
    "descriptor_budget_before_H_D": budget,
    "b2_budget_before_H": (len(b) - 40) * E - 5 * F,
    "baseline_translation_at_eligible_floor": (len(d) - len(b)) * E,
    "C_sensitivity_g5": {str(c): (len(d)-c)*E-5*F for c in [40,45,50,55,60,65,66]},
    "residual_after_one_added_instruction_per_scalar_row": budget-scalar,
    "hypothetical_three_added_per_refill_plus_one_per_build": budget-3*all_refills_upper-B,
    "hypothetical_cost_cap_NOT_an_observed_bound": 3*all_refills_upper+B,
    "corpus_multipliers": {"scalar_rows":scalar,"all_refills_upper":all_refills_upper,
                         "finished_portions":P,"runs":R,"block_characters":256*B,"blocks":B},
}
print(json.dumps(out, indent=2))
