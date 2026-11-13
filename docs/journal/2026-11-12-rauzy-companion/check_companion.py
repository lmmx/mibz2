"""Assert source preservation and the exact mathematics of the article's toy figures.

Run from any directory with Python 3.11+: python path/to/check_companion.py.
This is publication validation, not a decoder experiment or performance test.
"""

from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
ARTICLE = ROOT / "docs/site/blog/2026-11-12-rauzy-technical-companion.md"
SOURCES = {
    "rauzy-technical-report-source.md": (
        46590,
        "4a69f05174e03abae00f7f73c7ac4726fa9d8650e32031bcea0c9c71962d5d26",
    ),
    "reading-annotations-editorial-source.md": (
        7894,
        "feaf85b15ca73d1d05cd3cbeaa1c292a188d2f68b3a25564cfa780ba7a294064",
    ),
}


def normalise(text):
    """Only publishing syntax and immutable source links; never mathematical content."""
    return (
        text.replace(r"\[", "$$")
        .replace(r"\]", "$$")
        .replace(r"\(", "$")
        .replace(r"\)", "$")
        .replace(
            "https://github.com/lmmx/mibz2/blob/master/",
            "https://github.com/lmmx/mibz2/blob/2635f04e9155dd31388b24fd3f7248f87b3ddcda/",
        )
    )


def first_return(mapping, section, start):
    """Walk the finite permutation until its first positive return to the section."""
    visited = []
    point = start
    while True:
        visited.append(point)
        point = mapping[point]
        if point in section:
            return point, visited
        assert len(visited) <= len(mapping)


def main():
    """Verify preserved material, formal examples and actual SVG geometry."""
    for name, (size, digest) in SOURCES.items():
        raw = gzip.decompress((HERE / "source" / (name + ".gz")).read_bytes())
        assert (HERE / "source" / name).read_bytes() == raw + b"\n", name
        assert len(raw) == size, name
        assert hashlib.sha256(raw).hexdigest() == digest, name
    source = (HERE / "source/rauzy-technical-report-source.md").read_text()
    article = ARTICLE.read_text()
    edits = json.loads((HERE / "evidence/editorial-changes.json").read_text())
    expected = normalise(source)
    for edit in edits:
        assert edit["before"] in expected, edit["before"][:100]
        expected = expected.replace(edit["before"], edit["after"])
    paragraphs = [part.strip() for part in expected.split("\n\n") if part.strip()]
    missing = [part for part in paragraphs if part not in article]
    assert not missing, missing
    # Every source display formula survives literally, modulo math delimiters.
    equations = re.findall(r"\$\$(.*?)\$\$", normalise(source), re.S)
    # The final chain is typeset vertically after actual phone inspection.
    # Explicit recorded edit preserves all four terms and their order.
    for edit in edits:
        if edit["reason"].startswith("Reflow the identical"):
            equations = [eq.replace(edit["before"], edit["after"]) for eq in equations]
    actual_equations = Counter(re.findall(r"\$\$(.*?)\$\$", article, re.S))
    assert not Counter(equations) - actual_equations
    table_rows = [line for line in normalise(source).splitlines() if line.startswith("|")]
    assert all(row in article for row in table_rows)
    source_sections = re.findall(r"^## (.+)$", source, re.M)
    assert len(source_sections) == len(re.findall(r"^## (.+)$", article, re.M))
    # Both orders of F2.1 have exactly the advertised lengths and translations.
    translations = [1, 1, 1, 1, 1, -5]
    exchange = [x + translations[x] for x in range(6)]
    assert sorted(exchange) == list(range(6))
    assert exchange[:2] == [1, 2] and exchange[2:5] == [3, 4, 5] and exchange[5] == 0
    # A1: top B3 A5 / bottom A5 B3. Exact old returns on all retained rows.
    mapping = [x + 5 if x < 3 else x - 3 for x in range(8)]
    section = set(range(5))
    returns = [first_return(mapping, section, x) for x in range(5)]
    assert [dest for dest, _ in returns] == [2, 3, 4, 0, 1]
    assert [len(path) for _, path in returns] == [2, 2, 2, 1, 1]
    assert returns[1] == (3, [1, 6])
    assert all(
        ["B" if p < 3 else "A" for p in path] == (["B", "A"] if x < 3 else ["A"])
        for x, (_, path) in enumerate(returns)
    )
    # Reverse-itinerary convention: LF reads reverse(wB), then reverse(wA).
    word_a, word_b = "abc", "de"
    assert word_b[::-1] + word_a[::-1] == (word_a + word_b)[::-1]
    # Selected occurrence is an expansion offset, not an orig_ptr row number.
    words = ["a", "b"]
    words += [words[0] + words[1]]
    words += [words[2] + words[2]]
    words += [words[3] + words[2]]
    cycle = words[4][3:] + words[4][:3]
    assert words[4] == "ababab" and cycle == "bababa"
    assert (cycle * 2)[:10] == "bababababa"
    orbit = [1, 2, 3, 4, 5, 0]
    assert first_return(orbit, {0, 3}, 0) == (3, [0, 1, 2])
    assert first_return(orbit, {0, 3}, 3) == (0, [3, 4, 5])
    # Check drawn rectangle widths, not merely the numbers used by this oracle.
    ns = {"s": "http://www.w3.org/2000/svg"}
    figures = {}
    for ident, body in re.findall(
        r'<figure class="bf-fig" id="([^"]+)">(.*?)</figure>', article, re.S
    ):
        svg = ET.fromstring(re.search(r"<svg.*?</svg>", body, re.S).group())
        assert svg.attrib["role"] == "img"
        assert svg.find("s:title", ns) is not None and svg.find("s:desc", ns) is not None
        figures[ident] = svg

    def widths(ident, y):
        return [
            int(e.attrib["width"])
            for e in figures[ident].findall("s:rect", ns)
            if e.attrib.get("y") == str(y) and "stroke-dasharray" not in e.attrib.get("style", "")
        ]

    assert widths("rv-exchange", 86) == [136, 204, 68]
    assert widths("rv-exchange", 240) == [68, 136, 204]
    assert widths("rv-right-end", 85) == [165, 275]
    assert widths("rv-right-end", 185) == [275, 165]
    assert widths("rv-right-end", 484) == [165, 110]
    assert widths("rv-right-end", 585) == [110, 165]
    result = dict(
        source_hashes_verified=SOURCES,
        preserved_paragraphs=len(paragraphs),
        display_equations=len(equations),
        table_rows=len(table_rows),
        source_sections=source_sections,
        explicit_edits=len(edits),
        formal_static_figures=list(figures),
        toy_maps="all integer points checked",
        article_sha256=hashlib.sha256(ARTICLE.read_bytes()).hexdigest(),
    )
    print(json.dumps(result, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
