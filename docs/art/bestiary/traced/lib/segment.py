"""Stage 3: Non-destructive ink-guided watershed anatomical segmentation and smooth joint tucks."""
from __future__ import annotations

from typing import Any, Dict, List, Optional, Tuple

import numpy as np
from PIL import Image, ImageDraw
from scipy import ndimage
from skimage.filters import sobel
from skimage.morphology import skeletonize
from skimage.segmentation import watershed

from .presets import FamilyPreset


def _disk(r: int) -> np.ndarray:
    if r <= 0:
        return np.ones((1, 1), dtype=bool)
    y, x = np.ogrid[-r : r + 1, -r : r + 1]
    return (x * x + y * y) <= (r * r + 0.25)


def _draw_polyline_mask(pts_u: List[List[float]], w: int, h: int, px_per_u: float, width_px: int = 3) -> np.ndarray:
    m = Image.new("L", (w, h), 0)
    draw = ImageDraw.Draw(m)
    pts_px = [(p[0] * px_per_u, p[1] * px_per_u) for p in pts_u]
    if len(pts_px) == 1:
        r = max(1, width_px // 2)
        x, y = pts_px[0]
        draw.ellipse((x - r, y - r, x + r, y + r), fill=255)
    elif len(pts_px) >= 2:
        draw.line(pts_px, fill=255, width=max(1, width_px))
    return np.asarray(m, dtype=np.uint8) > 127


def _draw_polygon_mask(pts_u: List[List[float]], w: int, h: int, px_per_u: float) -> np.ndarray:
    m = Image.new("L", (w, h), 0)
    draw = ImageDraw.Draw(m)
    draw.polygon([(p[0] * px_per_u, p[1] * px_per_u) for p in pts_u], fill=255)
    return np.asarray(m, dtype=np.uint8) > 127


def _build_elevation_map(
    lab: np.ndarray,
    ink_mask: np.ndarray,
    fg_mask: np.ndarray,
    cut_guides: Optional[List[Dict[str, Any]]],
    px_per_u: float,
) -> np.ndarray:
    """Build watershed elevation surface that snaps partition boundaries to painter's iron-gall ink lines."""
    H, W = fg_mask.shape
    # Color gradient in CIELAB
    gL = sobel(lab[..., 0].astype(np.float32) / 100.0)
    ga = sobel(lab[..., 1].astype(np.float32) / 80.0)
    gb = sobel(lab[..., 2].astype(np.float32) / 80.0)
    grad = np.sqrt(gL * gL + ga * ga + gb * gb)
    if np.any(fg_mask):
        p95 = float(np.percentile(grad[fg_mask], 95))
        grad = np.clip(grad / max(p95, 1e-4), 0.0, 1.5)

    # Strong elevation ridge along actual iron-gall ink lines (smoothed slightly so ridges are clean)
    ink_ridge = ndimage.gaussian_filter(ink_mask.astype(np.float32), sigma=1.1)
    elev = 0.45 * grad + 1.40 * ink_ridge

    # Optional declarative joint cut guides: gentle barrier that guides cuts across wide wash zones
    # while letting nearby real ink strokes dominate exact placement
    if cut_guides:
        for guide in cut_guides:
            pts = guide.get("points", [])
            weight = float(guide.get("weight", 1.8))
            width_u = float(guide.get("width_u", 1.6))
            if len(pts) >= 2:
                gm = _draw_polyline_mask(pts, W, H, px_per_u, width_px=max(2, int(round(width_u * px_per_u))))
                elev += weight * ndimage.gaussian_filter(gm.astype(np.float32), sigma=1.2)

    return elev.astype(np.float32)


def _partition_by_seeds(
    domain_mask: np.ndarray,
    lab: np.ndarray,
    ink_mask: np.ndarray,
    pigment_masks: Dict[str, np.ndarray],
    part_specs: List[Dict[str, Any]],
    cut_guides: Optional[List[Dict[str, Any]]],
    px_per_u: float,
) -> Dict[str, np.ndarray]:
    """Run non-destructive ink-guided watershed strictly inside domain_mask. Union == 100.0% of domain_mask."""
    H, W = domain_mask.shape
    elev = _build_elevation_map(lab, ink_mask, domain_mask, cut_guides, px_per_u)

    markers = np.zeros((H, W), dtype=np.int32)
    id_to_name: Dict[int, str] = {}

    for idx, ps in enumerate(part_specs, start=1):
        name = ps["id"]
        id_to_name[idx] = name
        seed_m = np.zeros((H, W), dtype=bool)

        for pt in ps.get("seeds", []):
            x = int(np.clip(round(pt[0] * px_per_u), 0, W - 1))
            y = int(np.clip(round(pt[1] * px_per_u), 0, H - 1))
            r_px = max(2, int(round(1.2 * px_per_u)))
            y0, y1 = max(0, y - r_px), min(H, y + r_px + 1)
            x0, x1 = max(0, x - r_px), min(W, x + r_px + 1)
            yy, xx = np.ogrid[y0:y1, x0:x1]
            disk = ((xx - x) ** 2 + (yy - y) ** 2) <= r_px * r_px
            seed_m[y0:y1, x0:x1] |= disk

        for stroke in ps.get("seed_strokes", []):
            seed_m |= _draw_polyline_mask(stroke, W, H, px_per_u, width_px=max(2, int(round(1.5 * px_per_u))))

        if ps.get("seed_polygon"):
            seed_m |= _draw_polygon_mask(ps["seed_polygon"], W, H, px_per_u)

        if ps.get("prefer_layer") and ps["prefer_layer"] in pigment_masks:
            # Erode preferred pigment layer slightly to form pure interior seeds
            pref = ndimage.binary_erosion(pigment_masks[ps["prefer_layer"]] & domain_mask, structure=_disk(2))
            if ps.get("region_box"):
                bx0, by0, bx1, by1 = [int(round(v * px_per_u)) for v in ps["region_box"]]
                box_m = np.zeros((H, W), dtype=bool)
                box_m[max(0, by0) : min(H, by1), max(0, bx0) : min(W, bx1)] = True
                pref &= box_m
            seed_m |= pref

        # Snap seeds inside domain_mask; if a seed point fell slightly on an edge, snap to nearest domain pixel
        seed_in = seed_m & domain_mask
        if not np.any(seed_in) and np.any(seed_m) and np.any(domain_mask):
            sy, sx = ndimage.center_of_mass(seed_m)
            dy, dx = np.where(domain_mask)
            nearest = int(np.argmin((dy - sy) ** 2 + (dx - sx) ** 2))
            seed_in[dy[nearest], dx[nearest]] = True

        # Avoid overwriting earlier markers at direct overlap
        markers[seed_in & (markers == 0)] = idx

    if np.max(markers) == 0:
        return {part_specs[0]["id"]: domain_mask.copy()}

    # Watershed partition
    labels = watershed(elev, markers=markers, mask=domain_mask, compactness=0.0005)

    # Guarantee 100.0% non-destructive coverage of domain_mask (fill any unreached island via nearest label)
    unassigned = domain_mask & (labels == 0)
    if np.any(unassigned) and np.any(labels > 0):
        _, nearest_idx = ndimage.distance_transform_edt(labels == 0, return_indices=True)
        labels[unassigned] = labels[nearest_idx[0][unassigned], nearest_idx[1][unassigned]]

    # Clean thin contour-crawling whiskers (1-4px filaments sticking far out along ink ridges)
    # and tiny disconnected speckles by reassigning them to their touching neighbor component.
    # Preserves Union(out) == 100.0% of domain_mask (zero outer foreground pixels ever dropped!).
    core_struct = _disk(4)
    for _pass in range(2):
        for idx in id_to_name:
            m = (labels == idx) & domain_mask
            if not np.any(m):
                continue
            core = ndimage.binary_opening(m, structure=core_struct)
            if not np.any(core):
                continue
            dt_core = ndimage.distance_transform_edt(~core)
            far_thin = m & (dt_core > 7.0)
            if np.any(far_thin):
                lbl_ft, n_ft = ndimage.label(far_thin)
                for cid in range(1, n_ft + 1):
                    blob = lbl_ft == cid
                    ring = ndimage.binary_dilation(blob, iterations=2) & (~blob) & domain_mask
                    nbrs = labels[ring]
                    nbrs = nbrs[(nbrs > 0) & (nbrs != idx)]
                    if len(nbrs) > 0:
                        # Only reassign if this thin whisker has substantial contact with a neighbor
                        vals, counts = np.unique(nbrs, return_counts=True)
                        best_i = int(np.argmax(counts))
                        if counts[best_i] >= 4:
                            labels[blob] = int(vals[best_i])

            # Reassign any detached minor islands touching a neighbor component so every part is single-bodied
            m2 = (labels == idx) & domain_mask
            lbl_cc, n_cc = ndimage.label(m2)
            if n_cc <= 1:
                continue
            areas = ndimage.sum(m2, lbl_cc, index=np.arange(1, n_cc + 1))
            max_area = float(np.max(areas))
            for cid in range(1, n_cc + 1):
                if areas[cid - 1] < min(400, 0.20 * max_area):
                    speck = lbl_cc == cid
                    ring = ndimage.binary_dilation(speck, iterations=2) & (~speck) & domain_mask
                    nbrs = labels[ring]
                    nbrs = nbrs[(nbrs > 0) & (nbrs != idx)]
                    if len(nbrs) > 0:
                        vals, counts = np.unique(nbrs, return_counts=True)
                        labels[speck] = int(vals[int(np.argmax(counts))])

    # Rebuild final exact partition from cleaned labels
    out: Dict[str, np.ndarray] = {}
    for idx, name in id_to_name.items():
        out[name] = (labels == idx) & domain_mask

    return out


def _auto_skeleton_partition(fg_mask: np.ndarray, max_parts: int = 4) -> Dict[str, np.ndarray]:
    """Automatic skeleton + EDT pinch discovery for specimens without declarative component seeds."""
    if not np.any(fg_mask):
        return {"body": fg_mask.copy()}
    if max_parts <= 1:
        return {"body": fg_mask.copy()}

    # If foreground naturally has multiple large connected components (e.g., multi-figure woodcut plate),
    # split cleanly by connected components first!
    lbl, n_cc = ndimage.label(fg_mask)
    if n_cc > 1:
        areas = ndimage.sum(fg_mask, lbl, index=np.arange(1, n_cc + 1))
        order = np.argsort(areas)[::-1]
        major = [int(i) + 1 for i in order if areas[i] >= 0.05 * np.sum(fg_mask)]
        if len(major) >= 2:
            # Assign minor CCs to nearest major CC
            base = np.zeros_like(lbl)
            for new_id, old_id in enumerate(major, start=1):
                base[lbl == old_id] = new_id
            unass = fg_mask & (base == 0)
            if np.any(unass):
                _, idx = ndimage.distance_transform_edt(base == 0, return_indices=True)
                base[unass] = base[idx[0][unass], idx[1][unass]]
            return {f"figure-{i}": (base == i) for i in range(1, len(major) + 1)}

    # Single connected creature: discover skeleton branch points & local EDT pinch necks
    edt = ndimage.distance_transform_edt(fg_mask)
    skel = skeletonize(fg_mask)
    if np.sum(skel) < 30:
        return {"body": fg_mask.copy()}

    # Count 8-neighbors along skeleton to locate endpoints and junctions
    k3 = np.ones((3, 3), dtype=np.int32)
    k3[1, 1] = 0
    deg = ndimage.convolve(skel.astype(np.int32), k3, mode="constant", cval=0) * skel
    junctions = deg >= 3
    if not np.any(junctions):
        return {"body": fg_mask.copy()}

    # Dilate junctions slightly to cut skeleton into branch segments
    j_dil = ndimage.binary_dilation(junctions, iterations=3)
    branches = skel & (~j_dil)
    blbl, nb = ndimage.label(branches)
    if nb < 2:
        return {"body": fg_mask.copy()}

    b_areas = ndimage.sum(branches, blbl, index=np.arange(1, nb + 1))
    top_ids = [int(i) + 1 for i in np.argsort(b_areas)[::-1][:max_parts] if b_areas[i] >= 18]
    if len(top_ids) < 2:
        return {"body": fg_mask.copy()}

    markers = np.zeros(fg_mask.shape, dtype=np.int32)
    for new_i, bid in enumerate(top_ids, start=1):
        markers[blbl == bid] = new_i
    labels = watershed(-edt, markers=markers, mask=fg_mask)
    unass = fg_mask & (labels == 0)
    if np.any(unass) and np.any(labels > 0):
        _, idx = ndimage.distance_transform_edt(labels == 0, return_indices=True)
        labels[unass] = labels[idx[0][unass], idx[1][unass]]
    return {f"part-{i}": (labels == i) for i in range(1, len(top_ids) + 1)}


def _apply_joint_tucks(
    parts: Dict[str, np.ndarray],
    fg_mask: np.ndarray,
    tuck_specs: List[Dict[str, Any]],
    px_per_u: float,
) -> Dict[str, np.ndarray]:
    """Add smooth rounded under-tuck tabs (clipped strictly to fg_mask) at articulated joints."""
    out = {k: v.copy() for k, v in parts.items()}
    for ts in tuck_specs:
        parent = ts.get("parent")
        child = ts.get("child")
        if parent not in out or child not in out:
            continue
        radius_u = float(ts.get("radius_u", 3.5))
        radius_px = max(2.0, radius_u * px_per_u)
        p_mask = out[parent]
        c_mask = out[child]
        # Seam pixels: child pixels directly touching parent (optionally restricted to an attachment window)
        seam = ndimage.binary_dilation(p_mask, iterations=2) & c_mask
        if ts.get("joint_point"):
            jx, jy = ts["joint_point"]
            j_rad_px = float(ts.get("joint_window_u", 16.0)) * px_per_u
            yy, xx = np.mgrid[0 : fg_mask.shape[0], 0 : fg_mask.shape[1]]
            near_joint = ((xx - jx * px_per_u) ** 2 + (yy - jy * px_per_u) ** 2) <= j_rad_px ** 2
            seam &= near_joint
        if not np.any(seam):
            continue
        dist_from_seam = ndimage.distance_transform_edt(~seam)
        tuck = (dist_from_seam <= radius_px) & c_mask & fg_mask
        # Smooth the tuck tab slightly so it is nicely rounded inside child without altering exterior contour
        tuck = ndimage.binary_closing(tuck, structure=_disk(2)) & c_mask & fg_mask
        out[parent] = p_mask | tuck
    return out


def segment_components(
    rgb: np.ndarray,
    fg_mask: np.ndarray,
    lab: np.ndarray,
    pigments: Dict[str, Any],
    preset: FamilyPreset,
    spec: Optional[Dict[str, Any]] = None,
    px_per_u: float = 3.0,
) -> Dict[str, Any]:
    """Stage 3: Partition foreground into non-destructive anatomical components (+ optional subparts & tucks)."""
    spec = spec or {}
    if preset.segment_hook is not None:
        return preset.segment_hook(rgb, fg_mask, pigments, spec, preset)

    ink_mask = pigments["ink"]
    p_masks = pigments["masks"]
    comp_specs = spec.get("components", [])

    if not comp_specs:
        auto_parts = _auto_skeleton_partition(fg_mask, max_parts=int(spec.get("auto_max_parts", 1)))
        return {
            "parts": auto_parts,
            "raw_partition": {k: v.copy() for k, v in auto_parts.items()},
            "subparts": {},
            "order": list(auto_parts.keys()),
            "part_meta": {k: {"id": k, "title": k} for k in auto_parts},
            "union_coverage_pct": 100.0,
        }

    # Primary non-destructive ink-guided watershed partition
    raw_parts = _partition_by_seeds(
        domain_mask=fg_mask,
        lab=lab,
        ink_mask=ink_mask,
        pigment_masks=p_masks,
        part_specs=comp_specs,
        cut_guides=spec.get("cut_guides"),
        px_per_u=px_per_u,
    )

    # Verify 100.0% non-destructive union coverage before tucks
    union_mask = np.zeros_like(fg_mask)
    for m in raw_parts.values():
        union_mask |= m
    fg_n = max(1, int(np.sum(fg_mask)))
    cov_pct = 100.0 * float(np.sum(union_mask & fg_mask)) / float(fg_n)

    # Apply smooth rounded joint under-tucks (strictly clipped to fg_mask)
    tuck_specs = spec.get("tucks", [])
    parts_tucked = _apply_joint_tucks(raw_parts, fg_mask, tuck_specs, px_per_u)

    # Partition any semantic subparts (e.g., head -> head-jaw, head-skull, head-ears)
    subparts: Dict[str, Dict[str, np.ndarray]] = {}
    part_meta: Dict[str, Dict[str, Any]] = {}
    for cs in comp_specs:
        cid = cs["id"]
        part_meta[cid] = cs
        if cs.get("subparts"):
            dom = parts_tucked[cid]
            raw_subs = _partition_by_seeds(
                domain_mask=dom,
                lab=lab,
                ink_mask=ink_mask,
                pigment_masks=p_masks,
                part_specs=cs["subparts"],
                cut_guides=cs.get("subpart_cut_guides"),
                px_per_u=px_per_u,
            )
            subs_tucked = _apply_joint_tucks(raw_subs, dom, cs.get("subpart_tucks", []), px_per_u)
            subparts[cid] = subs_tucked

    return {
        "parts": parts_tucked,
        "raw_partition": raw_parts,
        "subparts": subparts,
        "order": [cs["id"] for cs in comp_specs],
        "part_meta": part_meta,
        "union_coverage_pct": round(cov_pct, 4),
    }
