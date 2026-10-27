"""The home page's illustration, docs/site/assets/images/hero.svg.

Three glass tubes on the Borromean rectangles: a golden rectangle in each coordinate
plane, each threading the next. No two are linked, and the three cannot be pulled apart:
independent chains, one block. Each tube is drawn as translucent facets, painted far to
near, over a blurred shadow of its own colour.

Run from the repository root: python docs/scripts/hero.py
"""

import math
import re
from pathlib import Path

OUT = Path("docs/site/assets/images/hero.svg")
W, H = 1400, 1000
VIEWBOX = (355, 195, 690, 725)  # the painted area, shadows included, plus a margin
B = 112  # half the short side
A = B * (1 + 5**0.5) / 2  # half the long side
T = 64  # tube diameter: the tubes pass A - B = 69.2 apart, so 5 units clear
RC = 40  # corner radius of the centreline
K = 28  # facets round a tube
AZ, EL = math.radians(41), math.radians(31)
SC, OX, OY = 1.22, W / 2, H / 2 - 30
CORAL, BLUE, MINT = (255, 92, 74), (40, 92, 255), (40, 190, 135)


def add(*v):
    return tuple(map(sum, zip(*v)))


def mul(v, k):
    return tuple(c * k for c in v)


def sub(a, b):
    return add(a, mul(b, -1))


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def cross(a, b):
    return (
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    )


def norm(v):
    return mul(v, 1 / math.sqrt(dot(v, v)))


D = (math.cos(EL) * math.cos(AZ), math.cos(EL) * math.sin(AZ), math.sin(EL))  # to viewer
RT = norm(cross((0, 0, 1), D))
UP = cross(D, RT)
LIGHT = norm(add(mul(D, 0.7), mul(UP, 0.6), mul(RT, -0.35)))


def project(p):
    return OX + SC * dot(p, RT), OY - SC * dot(p, UP)


def centreline(corners):
    """Points and tangents round a closed polygon of right angles, corners rounded"""
    n = len(corners)
    out = []
    for i in range(n):
        p0, p1, p2 = corners[i - 1], corners[i], corners[(i + 1) % n]
        u, v = norm(sub(p0, p1)), norm(sub(p2, p1))
        c = add(p1, mul(u, RC), mul(v, RC))
        for k in range(10):
            th = math.pi / 2 * k / 9
            point = add(c, mul(u, -RC * math.sin(th)), mul(v, -RC * math.cos(th)))
            tangent = norm(add(mul(u, -math.cos(th)), mul(v, math.sin(th))))
            out.append((point, tangent))
    return out + [out[0]]


def tube(corners, normal, colour, faces):
    """The facets of a round tube along a closed rectangle in the plane with `normal`"""
    seq = centreline(corners)
    rings = []
    for q, tg in seq:
        s = norm(cross(tg, normal))
        h, w = mul(normal, T / 2), mul(s, T / 2)
        rings.append(
            [
                add(q, mul(h, math.cos(2 * math.pi * j / K)), mul(w, math.sin(2 * math.pi * j / K)))
                for j in range(K)
            ]
        )
    for i in range(len(rings) - 1):
        if dot(sub(seq[i + 1][0], seq[i][0]), sub(seq[i + 1][0], seq[i][0])) < 1e-6:
            continue
        r0, r1 = rings[i], rings[i + 1]
        mid = mul(add(seq[i][0], seq[i + 1][0]), 0.5)
        for j in range(K):
            quad = [r0[j], r1[j], r1[(j + 1) % K], r0[(j + 1) % K]]
            centre = mul(add(*quad), 0.25)
            faces.append((dot(centre, D), quad, norm(sub(centre, mid)), colour))


def rectangle(long_axis, short_axis):
    def point(lo, sh):
        v = [0.0, 0.0, 0.0]
        v[long_axis], v[short_axis] = lo, sh
        return tuple(v)

    normal = [0, 0, 0]
    normal[3 - long_axis - short_axis] = 1
    corners = [point(-A, -B), point(A, -B), point(A, B), point(-A, B)]
    return corners, tuple(normal)


def rgb(c):
    return "rgb(%d,%d,%d)" % tuple(max(0, min(255, int(x))) for x in c)


def mix(c, w, k):
    return tuple(a * (1 - k) + b * k for a, b in zip(c, w))


def svg():
    faces = []
    # coral in the xy-plane, blue in yz, mint in zx: each threads the next
    for axes, colour in (((0, 1), CORAL), ((1, 2), BLUE), ((2, 0), MINT)):
        corners, normal = rectangle(*axes)
        tube(corners, normal, colour, faces)
    faces.sort(key=lambda f: f[0])
    body = []
    for _, quad, n, colour in faces:
        lit = max(0, dot(n, LIGHT))
        fill = mix(colour, (255, 255, 255), 0.06 + 0.34 * lit**2)
        alpha = 0.50 if dot(n, D) > 0 else 0.20
        if dot(n, D) > 0 and lit > 0.75:
            alpha += 0.08
        points = " ".join("%.1f,%.1f" % project(v) for v in quad)
        body.append(f'<polygon points="{points}" fill="{rgb(fill)}" fill-opacity="{alpha:.2f}"/>')
    # soft coloured shadows: each tube's footprint on a floor below, blurred
    floor = -(A + 30)
    footprints = (
        (CORAL, [(-A, -B), (A, -B), (A, B), (-A, B)]),
        (BLUE, [(0, -A), (0, A)]),
        (MINT, [(-B, 0), (B, 0)]),
    )
    shadows = []
    for colour, pts in footprints:
        poly = " ".join("%.1f,%.1f" % project((x, y, floor)) for x, y in pts)
        shadows.append(
            f'<polygon points="{poly}" fill="none" stroke="{rgb(colour)}" stroke-opacity="0.30"'
            f' stroke-width="{T * 1.5 * SC}" stroke-linejoin="round"/>'
        )
    x, y, w, h = VIEWBOX
    out = f"""<svg xmlns="http://www.w3.org/2000/svg" role="img" aria-label="Three interlocking glass loops" viewBox="{x} {y} {w} {h}" width="{w}" height="{h}">
<defs><filter id="blur" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="26"/></filter></defs>
<g filter="url(#blur)">{"".join(shadows)}</g>
{chr(10).join(body)}
</svg>"""
    # whole units are finer than a pixel at the sizes the page shows it
    return re.sub(
        r'points="([^"]*)"',
        lambda m: 'points="%s"'
        % re.sub(r"-?\d+\.\d+", lambda d: str(round(float(d.group(0)))), m.group(1)),
        out,
    )


if __name__ == "__main__":
    OUT.write_text(svg())
