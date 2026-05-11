#!/usr/bin/env python3
"""
generate_entity_icons.py

Generates a 512×320 PNG icon tileset (8 cols × 5 rows × 64px) for LDtk entity
definitions, then patches levels/jasperworld.ldtk to reference it.

Non-prop entities (row 0) are rendered as colored rounded-rectangle labels.
Prop entities (rows 1–4) are rendered from GLB models using trimesh + matplotlib.
PNG-model props are thumbnailed from the source PNG.

Output tileset: assets/entity_icons.png
"""

import json
import io
import os
import sys
import math
import traceback
import warnings

import numpy as np
from PIL import Image, ImageDraw, ImageFont

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------
REPO = "/Users/ericwitte/Documents/claude_projects/jasperworld_alpha"
ASSETS_DIR = os.path.join(REPO, "assets")
MODELS_DIR = os.path.join(ASSETS_DIR, "models")
LDTK_PATH  = os.path.join(REPO, "levels", "jasperworld.ldtk")
OUT_PNG    = os.path.join(ASSETS_DIR, "entity_icons.png")

# ---------------------------------------------------------------------------
# Icon grid parameters
# ---------------------------------------------------------------------------
ICON_SIZE = 64
COLS      = 8

# ---------------------------------------------------------------------------
# Entity lists
# ---------------------------------------------------------------------------
NON_PROP = [
    ("Spawn",      "#FFEB3B", "⊙"),
    ("Enemy",      "#F44336", "⚠"),
    ("Star",       "#FFD700", "★"),
    ("HealthFood", "#FF8A65", "♥"),
    ("Door",       "#9C27B0", "▣"),
    ("Gate",       "#795548", "⊞"),
    ("Exit",       "#607D8B", "→"),
    ("Light",      "#D77643", "✦"),
]

PROP_MODELS = [
    # (entity_identifier, model_relative_to_assets, entity_color)
    ("Prop_TreeOak",       "models/tree_oak.glb",                               "#8BC34A"),
    ("Prop_TreePine",      "models/tree_pine.glb",                              "#8BC34A"),
    ("Prop_TreeFat",       "models/tree_fat.glb",                               "#8BC34A"),
    ("Prop_TreeDefault",   "models/tree_default.glb",                           "#8BC34A"),
    ("Prop_TreeSubLg",     "models/suburban/tree-suburban-large.glb",           "#8BC34A"),
    ("Prop_TreeSubSm",     "models/suburban/tree-suburban-small.glb",           "#8BC34A"),
    ("Prop_CherryBlossom", "models/sanctuary/tree_cherryblossom.glb",           "#F48FB1"),
    ("Prop_PlantBush",     "models/plant_bush.glb",                             "#8BC34A"),
    ("Prop_BushLarge",     "models/plant_bushLarge.glb",                        "#8BC34A"),
    ("Prop_GrassLarge",    "models/grass_large.glb",                            "#8BC34A"),
    ("Prop_FlowerRed",     "models/flower_redA.glb",                            "#E91E63"),
    ("Prop_YellowFlower",  "models/yellow_flower.glb",                          "#FFD600"),
    ("Prop_MushroomRed",   "models/mushroom_red.glb",                           "#F44336"),
    ("Prop_MushroomTan",   "models/mushroom_tan.glb",                           "#A1887F"),
    ("Prop_Mushrooms",     "models/mushrooms.glb",                              "#9C27B0"),
    ("Prop_SmallRock",     "models/small_rock.glb",                             "#9E9E9E"),
    ("Prop_LargeRock",     "models/large_rock.glb",                             "#757575"),
    ("Prop_Taxi",          "models/city/taxi.glb",                              "#FDD835"),
    ("Prop_LightCurved",   "models/city/light-curved.glb",                      "#FFF176"),
    ("Prop_ConstrCone",    "models/city/construction-cone.glb",                 "#FF6F00"),
    ("Prop_SewerBrick",    "models/sewer/brick-wall.glb",                       "#795548"),
    ("Prop_SewerColumn",   "models/sewer/column-large.glb",                     "#795548"),
    ("Prop_SewerFence",    "models/sewer/iron-fence.glb",                       "#607D8B"),
    ("Prop_SewerStoneCol", "models/sewer/stone-wall-column.glb",                "#78909C"),
    ("Prop_CaveRock",      "models/cave/cliff_cave_rock.glb",                   "#616161"),
    ("Prop_CaveStone",     "models/cave/cliff_cave_stone.glb",                  "#757575"),
    ("Prop_SanctuaryArch", "models/sanctuary/ornate+chinese+arch+3d+model.glb", "#E91E63"),
    ("Prop_SanctuaryLion", "models/sanctuary/sanctuary_lionstatue.glb",         "#E91E63"),
    ("Prop_Temple",        "models/sanctuary/asian+temple+island+3d+model.glb", "#E91E63"),
    ("Prop_RaccoonFamily", "models/sanctuary/raccoon_family.png",               "#A1887F"),  # PNG
    ("Prop_Water",         "models/sanctuary/water_at_end_oflevel.png",         "#2196F3"),  # PNG
]

# Flat ordered list used for index → tileset position mapping
ALL_ENTITIES = (
    [(ident, "#000000", "") for ident, color, sym in NON_PROP]  # placeholder; we rebuild below
)
# Build the real flat list: (identifier, color, symbol_or_path, is_prop, model_rel_path)
FLAT = []
for ident, color, sym in NON_PROP:
    FLAT.append({"id": ident, "color": color, "sym": sym, "is_prop": False, "path": None})
for ident, path, color in PROP_MODELS:
    FLAT.append({"id": ident, "color": color, "sym": None, "is_prop": True, "path": path})

# Total = 8 + 31 = 39 icons → 5 rows of 8 (last row has 7 real, 1 blank)
TOTAL_ROWS = math.ceil(len(FLAT) / COLS)
GRID_W = COLS * ICON_SIZE        # 512
GRID_H = TOTAL_ROWS * ICON_SIZE  # 320


# ---------------------------------------------------------------------------
# Color helpers
# ---------------------------------------------------------------------------
def hex_to_rgb(h: str):
    h = h.lstrip("#")
    return tuple(int(h[i:i+2], 16) for i in (0, 2, 4))


def hex_to_rgba(h: str, alpha: int = 255):
    r, g, b = hex_to_rgb(h)
    return (r, g, b, alpha)


def hex_to_rgb_float(h: str):
    r, g, b = hex_to_rgb(h)
    return r / 255.0, g / 255.0, b / 255.0


# ---------------------------------------------------------------------------
# Fallback: colored rectangle icon
# ---------------------------------------------------------------------------
def make_fallback_icon(color_hex: str, label: str = "") -> Image.Image:
    """Solid rounded-rect with optional label, RGBA 64×64."""
    img = Image.new("RGBA", (ICON_SIZE, ICON_SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    r, g, b = hex_to_rgb(color_hex)
    # Rounded rectangle
    margin = 4
    draw.rounded_rectangle(
        [margin, margin, ICON_SIZE - margin - 1, ICON_SIZE - margin - 1],
        radius=8,
        fill=(r, g, b, 220),
        outline=(255, 255, 255, 120),
        width=1,
    )
    if label:
        try:
            font = ImageFont.truetype("/System/Library/Fonts/Apple Color Emoji.ttc", 28)
        except Exception:
            try:
                font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 20)
            except Exception:
                font = ImageFont.load_default()
        # Center the text
        bbox = draw.textbbox((0, 0), label, font=font)
        tw = bbox[2] - bbox[0]
        th = bbox[3] - bbox[1]
        tx = (ICON_SIZE - tw) // 2 - bbox[0]
        ty = (ICON_SIZE - th) // 2 - bbox[1]
        draw.text((tx, ty), label, font=font, fill=(255, 255, 255, 240))
    return img


# ---------------------------------------------------------------------------
# Non-prop icon renderer
# ---------------------------------------------------------------------------
def render_nonprop_icon(color_hex: str, symbol: str) -> Image.Image:
    return make_fallback_icon(color_hex, symbol)


# ---------------------------------------------------------------------------
# GLB renderer (trimesh + matplotlib Agg)
# ---------------------------------------------------------------------------
def render_glb_icon(model_path: str, color_hex: str) -> Image.Image:
    """
    Render a GLB model to a 64×64 RGBA icon using trimesh + matplotlib (Agg).
    Returns a fallback colored rect on any failure.
    """
    import trimesh
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    from mpl_toolkits.mplot3d.art3d import Poly3DCollection

    try:
        if not os.path.isfile(model_path):
            raise FileNotFoundError(f"Model not found: {model_path}")

        # --- Load ---
        scene = trimesh.load(model_path, force=None)

        # --- Extract meshes (apply scene transforms for correctness) ---
        if isinstance(scene, trimesh.Scene):
            # to_geometry() applies node transforms before concatenating
            try:
                mesh = scene.to_geometry()
                if not isinstance(mesh, trimesh.Trimesh):
                    raise ValueError("to_geometry did not return a Trimesh")
            except Exception:
                geoms = list(scene.geometry.values())
                mesh = trimesh.util.concatenate(geoms) if len(geoms) > 1 else geoms[0]
        elif isinstance(scene, trimesh.Trimesh):
            mesh = scene
        else:
            geoms = list(scene.geometry.values()) if hasattr(scene, "geometry") else [scene]
            mesh = trimesh.util.concatenate(geoms) if len(geoms) > 1 else geoms[0]

        if not isinstance(mesh, trimesh.Trimesh) or len(mesh.vertices) == 0:
            raise ValueError("No usable mesh found in GLB")

        # --- Normalize to unit cube centered at origin ---
        verts = mesh.vertices.copy().astype(float)
        bounds_min = verts.min(axis=0)
        bounds_max = verts.max(axis=0)
        center = (bounds_min + bounds_max) / 2.0
        verts -= center
        extents = bounds_max - bounds_min
        max_extent = extents.max()
        if max_extent > 1e-9:
            verts /= max_extent

        # Build a normalized mesh for face/normal access
        norm_mesh = trimesh.Trimesh(
            vertices=verts,
            faces=mesh.faces,
            process=False,
        )

        # For very high-poly meshes (>100k faces), use convex hull to avoid hang
        MAX_FACES = 100_000
        if len(norm_mesh.faces) > MAX_FACES:
            try:
                norm_mesh = norm_mesh.convex_hull
                verts = norm_mesh.vertices.copy()
                print(f"(convex hull, {len(norm_mesh.faces)} faces) ", end="", flush=True)
            except Exception:
                pass  # proceed with full mesh, accept slowness

        # --- Face geometry ---
        faces = norm_mesh.faces                               # (F, 3)
        face_normals = norm_mesh.face_normals.astype(np.float32)  # float32 avoids BLAS warnings

        # Vertices for each face: shape (F, 3, 3)
        verts_3d = verts[faces]

        # --- Simple diffuse shading ---
        light_dir = np.array([0.5, 0.3, 1.0], dtype=np.float32)
        light_dir /= np.linalg.norm(light_dir)
        with warnings.catch_warnings():
            warnings.filterwarnings("ignore", category=RuntimeWarning)
            diffuse = np.clip(face_normals @ light_dir, 0.15, 1.0)  # (F,)

        # Base color from hex
        cr, cg, cb = hex_to_rgb_float(color_hex)

        # Per-face RGBA
        face_colors = np.stack(
            [diffuse * cr, diffuse * cg, diffuse * cb, np.ones(len(diffuse))],
            axis=1,
        )  # (F, 4)

        # --- Render ---
        fig = plt.figure(figsize=(1, 1), dpi=ICON_SIZE, facecolor="none")
        ax = fig.add_axes([0, 0, 1, 1], projection="3d", facecolor=(0, 0, 0, 0))

        poly = Poly3DCollection(verts_3d, facecolors=face_colors, edgecolors="none")
        ax.add_collection3d(poly)

        lim = 0.9
        ax.set_xlim(-lim, lim)
        ax.set_ylim(-lim, lim)
        ax.set_zlim(-lim, lim)
        ax.set_axis_off()
        ax.view_init(elev=25, azim=225)

        buf = io.BytesIO()
        fig.savefig(buf, format="png", transparent=True, bbox_inches=None, dpi=ICON_SIZE)
        plt.close(fig)
        buf.seek(0)

        img = Image.open(buf).convert("RGBA")
        img = img.resize((ICON_SIZE, ICON_SIZE), Image.LANCZOS)
        return img

    except Exception as exc:
        print(f"    WARNING: GLB render failed ({exc}), using fallback color rect")
        return make_fallback_icon(color_hex)


# ---------------------------------------------------------------------------
# PNG source renderer (thumbnail-crop from center)
# ---------------------------------------------------------------------------
def render_png_icon(png_path: str, color_hex: str) -> Image.Image:
    """Open a PNG, center-crop to square, resize to 64×64 RGBA."""
    try:
        if not os.path.isfile(png_path):
            raise FileNotFoundError(f"PNG not found: {png_path}")
        src = Image.open(png_path).convert("RGBA")
        w, h = src.size
        side = min(w, h)
        left  = (w - side) // 2
        top   = (h - side) // 2
        src = src.crop((left, top, left + side, top + side))
        src = src.resize((ICON_SIZE, ICON_SIZE), Image.LANCZOS)
        return src
    except Exception as exc:
        print(f"    WARNING: PNG load failed ({exc}), using fallback color rect")
        return make_fallback_icon(color_hex)


# ---------------------------------------------------------------------------
# Main generation
# ---------------------------------------------------------------------------
def generate_icons() -> Image.Image:
    grid = Image.new("RGBA", (GRID_W, GRID_H), (0, 0, 0, 0))

    total = len(FLAT)
    for i, entry in enumerate(FLAT):
        col = i % COLS
        row = i // COLS
        ident = entry["id"]
        color = entry["color"]
        label_idx = i + 1  # 1-based for display

        print(f"[{label_idx}/{total}] {ident} ...", end=" ", flush=True)

        if not entry["is_prop"]:
            # Non-prop: colored rounded rect with symbol
            icon = render_nonprop_icon(color, entry["sym"])
            print("label icon")
        else:
            path_rel = entry["path"]
            full_path = os.path.join(ASSETS_DIR, path_rel)

            if path_rel.lower().endswith(".png"):
                print(f"PNG ({os.path.basename(full_path)})")
                icon = render_png_icon(full_path, color)
            else:
                print(f"GLB ({os.path.basename(full_path)})")
                icon = render_glb_icon(full_path, color)

        x = col * ICON_SIZE
        y = row * ICON_SIZE
        grid.paste(icon, (x, y))

    return grid


# ---------------------------------------------------------------------------
# LDtk patch
# ---------------------------------------------------------------------------
TILESET_UID = 600

TILESET_DEF = {
    "uid": TILESET_UID,
    "identifier": "EntityIcons",
    "relPath": "../assets/entity_icons.png",
    "pxWid": GRID_W,
    "pxHei": GRID_H,
    "tileGridSize": ICON_SIZE,
    "spacing": 0,
    "padding": 0,
    "tags": [],
    "tagsSourceEnumUid": None,
    "enumTags": [],
    "customData": [],
    "savedSelections": [],
    "cachedPixelData": None,
    "__cHei": TOTAL_ROWS,
    "__cWid": COLS,
}


def patch_ldtk():
    with open(LDTK_PATH, "r", encoding="utf-8") as f:
        ldtk = json.load(f)

    # --- Upsert tileset ---
    tilesets = ldtk["defs"]["tilesets"]
    existing_idx = next(
        (i for i, t in enumerate(tilesets) if t.get("uid") == TILESET_UID), None
    )
    if existing_idx is not None:
        tilesets[existing_idx] = TILESET_DEF
        print(f"  Updated existing tileset uid={TILESET_UID} in defs.tilesets")
    else:
        tilesets.append(TILESET_DEF)
        print(f"  Appended new tileset uid={TILESET_UID} to defs.tilesets")

    # Advance nextUid so LDtk doesn't reuse our UID
    if ldtk.get("nextUid", 0) <= TILESET_UID:
        ldtk["nextUid"] = TILESET_UID + 1

    # Build lookup: identifier → flat index
    id_to_index = {entry["id"]: idx for idx, entry in enumerate(FLAT)}

    # --- Patch entity definitions ---
    patched = 0
    for entity in ldtk["defs"]["entities"]:
        ident = entity.get("identifier", "")
        if ident not in id_to_index:
            continue
        idx = id_to_index[ident]
        col = idx % COLS
        row = idx // COLS
        tile_rect = {
            "tilesetUid": TILESET_UID,
            "x": col * ICON_SIZE,
            "y": row * ICON_SIZE,
            "w": ICON_SIZE,
            "h": ICON_SIZE,
        }
        entity["tilesetId"]      = TILESET_UID
        entity["tileRect"]       = tile_rect
        entity["uiTileRect"]     = tile_rect
        entity["renderMode"]     = "Tile"
        entity["tileRenderMode"] = "FitInside"
        patched += 1

    print(f"  Patched {patched} entity definitions")

    with open(LDTK_PATH, "w", encoding="utf-8") as f:
        json.dump(ldtk, f, indent=2, ensure_ascii=False)
        f.write("\n")


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------
def main():
    print(f"Generating {len(FLAT)} entity icons ({GRID_W}×{GRID_H} tileset) ...\n")

    grid = generate_icons()

    os.makedirs(os.path.dirname(OUT_PNG), exist_ok=True)
    grid.save(OUT_PNG, "PNG")
    print(f"\n✓ Saved tileset: {os.path.relpath(OUT_PNG, REPO)}")

    print(f"\nPatching {os.path.relpath(LDTK_PATH, REPO)} ...")
    patch_ldtk()
    print("✓ Updated jasperworld.ldtk")


if __name__ == "__main__":
    main()
