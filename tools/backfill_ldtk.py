#!/usr/bin/env python3
"""Backfill jasperworld.ldtk from compiled_levels.json.

Reads compiled_levels.json (game-coordinate tile and entity data) and writes
all IntGrid tiles and entity instances into jasperworld.ldtk, replacing the
existing levels array with 9 fully-populated levels:
  Forest / Forest_Cave
  Subdivision / Subdivision_Sewer / Subdivision_Rooftop
  City / City_Subway / City_Rooftop

Coordinate conventions:
  compiled_levels.json uses game-space (Y up, origin at bottom-left).
  LDtk uses screen-space (Y down, origin at top-left, pixel units).

All reverse-mapping logic mirrors the Rust converter in ldtk_compiler/.
"""

import json
import math
import uuid
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

TILE_SIZE = 18.0
REPO_ROOT = Path(__file__).resolve().parent.parent

COMPILED_JSON = REPO_ROOT / "assets" / "levels" / "compiled_levels.json"
LDTK_FILE    = REPO_ROOT / "levels" / "jasperworld.ldtk"

# ---------------------------------------------------------------------------
# Helper: unique identifiers
# ---------------------------------------------------------------------------

def make_iid() -> str:
    return str(uuid.uuid4())


# ---------------------------------------------------------------------------
# Coordinate reverse-mapping
# ---------------------------------------------------------------------------

def reverse_surface(wx: float, wy: float, origin_x: float, origin_y: float, rows: int):
    """Reverse of px_to_world_surface (pivot [0.5, 1] — bottom-centre of tile cell).

    Rust original:
        col      = px[0] / TILE_SIZE
        ldtk_row = px[1] / TILE_SIZE
        game_row = (c_hei - 1) - ldtk_row
        wx       = origin_x + col * TILE_SIZE + TILE_SIZE * 0.5
        wy       = origin_y + (game_row + 1) * TILE_SIZE

    Reverse:
        col      = (wx - origin_x - TILE_SIZE/2) / TILE_SIZE
        game_row = (wy - origin_y) / TILE_SIZE - 1
        ldtk_row = (rows - 1) - game_row
        px_x     = round(col * TILE_SIZE)
        px_y     = round(ldtk_row * TILE_SIZE)
    """
    col      = (wx - origin_x - TILE_SIZE / 2.0) / TILE_SIZE
    game_row = (wy - origin_y) / TILE_SIZE - 1.0
    ldtk_row = (rows - 1) - game_row
    return [round(col * TILE_SIZE), round(ldtk_row * TILE_SIZE)]


def reverse_center(wx: float, wy: float, origin_x: float, origin_y: float, rows: int):
    """Reverse of px_to_world (pivot [0.5, 0.5] — tile centre).

    Rust original:
        col      = px[0] / TILE_SIZE
        ldtk_row = px[1] / TILE_SIZE
        game_row = (c_hei - 1) - ldtk_row
        wx       = origin_x + col * TILE_SIZE + TILE_SIZE * 0.5
        wy       = origin_y + game_row * TILE_SIZE + TILE_SIZE * 0.5

    Reverse:
        col      = (wx - origin_x - TILE_SIZE/2) / TILE_SIZE
        game_row = (wy - origin_y - TILE_SIZE/2) / TILE_SIZE
        ldtk_row = (rows - 1) - game_row
        px_x     = round(col * TILE_SIZE)
        px_y     = round(ldtk_row * TILE_SIZE)
    """
    col      = (wx - origin_x - TILE_SIZE / 2.0) / TILE_SIZE
    game_row = (wy - origin_y - TILE_SIZE / 2.0) / TILE_SIZE
    ldtk_row = (rows - 1) - game_row
    return [round(col * TILE_SIZE), round(ldtk_row * TILE_SIZE)]


# ---------------------------------------------------------------------------
# IntGrid conversion
# ---------------------------------------------------------------------------

def tiles_to_intgrid(tiles: list, cols: int, rows: int) -> list:
    """Convert game-space tiles[game_row][col] to LDtk flat intGridCsv.

    game_row 0 = bottom of level.
    LDtk intGridCsv is row-major, top-down (ldtk_row 0 = top of level).

    Mapping:
        ldtk_row = (rows - 1) - game_row
        csv_index = ldtk_row * cols + col
    """
    csv = [0] * (rows * cols)
    for game_row, row_data in enumerate(tiles):
        ldtk_row = (rows - 1) - game_row
        for col, val in enumerate(row_data):
            csv_index = ldtk_row * cols + col
            csv[csv_index] = val
    return csv


# ---------------------------------------------------------------------------
# Field instance builders
# ---------------------------------------------------------------------------

def field_string(identifier: str, def_uid: int, value) -> dict:
    """Create a String field instance."""
    return {
        "__identifier": identifier,
        "__type": "String",
        "__value": value,
        "__tile": None,
        "defUid": def_uid,
        "realEditorValues": [{"id": "V_String", "params": [value]}],
    }


def field_int(identifier: str, def_uid: int, value: int) -> dict:
    """Create an Int field instance."""
    return {
        "__identifier": identifier,
        "__type": "Int",
        "__value": value,
        "__tile": None,
        "defUid": def_uid,
        "realEditorValues": [{"id": "V_Int", "params": [value]}],
    }


def field_float(identifier: str, def_uid: int, value: float) -> dict:
    """Create a Float field instance."""
    return {
        "__identifier": identifier,
        "__type": "Float",
        "__value": value,
        "__tile": None,
        "defUid": def_uid,
        "realEditorValues": [{"id": "V_Float", "params": [value]}],
    }


def field_float_nullable(identifier: str, def_uid: int, value) -> dict:
    """Create a nullable Float field instance (value may be None)."""
    if value is None:
        return {
            "__identifier": identifier,
            "__type": "Float",
            "__value": None,
            "__tile": None,
            "defUid": def_uid,
            "realEditorValues": [],
        }
    return field_float(identifier, def_uid, value)


def field_string_nullable(identifier: str, def_uid: int, value) -> dict:
    """Create a nullable String field instance (value may be None)."""
    if value is None:
        return {
            "__identifier": identifier,
            "__type": "String",
            "__value": None,
            "__tile": None,
            "defUid": def_uid,
            "realEditorValues": [],
        }
    return field_string(identifier, def_uid, value)


def field_bool(identifier: str, def_uid: int, value: bool) -> dict:
    """Create a Bool field instance."""
    return {
        "__identifier": identifier,
        "__type": "Bool",
        "__value": value,
        "__tile": None,
        "defUid": def_uid,
        "realEditorValues": [{"id": "V_Bool", "params": [value]}],
    }


# ---------------------------------------------------------------------------
# Entity instance builder
# ---------------------------------------------------------------------------

def make_entity(
    identifier: str,
    def_uid: int,
    width: int,
    height: int,
    px: list,
    pivot_x: float,
    pivot_y: float,
    cols: int,
    rows: int,
    fields: list,
    level_id: int,
) -> dict:
    """Build a single entity instance dict matching LDtk's JSON schema."""
    # __grid is the tile-column / tile-row of the entity pivot (integer division)
    grid_x = px[0] // int(TILE_SIZE)
    grid_y = px[1] // int(TILE_SIZE)

    return {
        "__identifier": identifier,
        "__grid": [grid_x, grid_y],
        "__pivot": [pivot_x, pivot_y],
        "__tags": [],
        "__tile": None,
        "__smartColor": "",  # will be overwritten by LDtk on open; acceptable to leave blank
        "iid": make_iid(),
        "width": width,
        "height": height,
        "defUid": def_uid,
        "levelId": level_id,
        "px": px,
        "fieldInstances": fields,
    }


# ---------------------------------------------------------------------------
# Color helper
# ---------------------------------------------------------------------------

def rgb_floats_to_hex(rgb: list) -> str:
    """Convert [r, g, b] float list (0..1 each) to '#RRGGBB' hex string."""
    r = max(0, min(255, round(rgb[0] * 255)))
    g = max(0, min(255, round(rgb[1] * 255)))
    b = max(0, min(255, round(rgb[2] * 255)))
    return f"#{r:02X}{g:02X}{b:02X}"


# ---------------------------------------------------------------------------
# Per-entity-type builders
# ---------------------------------------------------------------------------

def build_spawn(layer: dict, origin_x: float, origin_y: float, rows: int, level_id: int) -> list:
    """Build zero or one Spawn entity instances from the layer's spawn field."""
    spawn = layer.get("spawn")
    if spawn is None:
        return []
    wx, wy = float(spawn[0]), float(spawn[1])
    px = reverse_surface(wx, wy, origin_x, origin_y, rows)
    return [make_entity(
        identifier="Spawn",
        def_uid=10,
        width=18, height=18,
        px=px,
        pivot_x=0.5, pivot_y=1.0,
        cols=layer["cols"], rows=rows,
        fields=[],
        level_id=level_id,
    )]


def build_enemies(layer: dict, origin_x: float, origin_y: float, rows: int, level_id: int) -> list:
    """Build Enemy entity instances."""
    instances = []
    for e in layer.get("enemies", []):
        wx, wy = float(e["x"]), float(e["y"])
        px = reverse_surface(wx, wy, origin_x, origin_y, rows)
        fields = [
            field_string("enemy_type",    50, e.get("enemy_type", "Dog")),
            field_float("patrol_range",   51, float(e.get("patrol_range", 72.0))),
            field_float("health",         52, float(e.get("health", 100.0))),
            field_float_nullable("speed_override", 53, e.get("speed_override")),
        ]
        instances.append(make_entity(
            identifier="Enemy",
            def_uid=11,
            width=18, height=18,
            px=px,
            pivot_x=0.5, pivot_y=1.0,
            cols=layer["cols"], rows=rows,
            fields=fields,
            level_id=level_id,
        ))
    return instances


def build_stars(layer: dict, origin_x: float, origin_y: float, rows: int, level_id: int) -> list:
    """Build Star entity instances. Position is [x, y, z]; z is ignored."""
    instances = []
    for s in layer.get("stars", []):
        wx, wy = float(s[0]), float(s[1])
        px = reverse_center(wx, wy, origin_x, origin_y, rows)
        instances.append(make_entity(
            identifier="Star",
            def_uid=12,
            width=18, height=18,
            px=px,
            pivot_x=0.5, pivot_y=0.5,
            cols=layer["cols"], rows=rows,
            fields=[],
            level_id=level_id,
        ))
    return instances


def build_health_foods(layer: dict, origin_x: float, origin_y: float, rows: int, level_id: int) -> list:
    """Build HealthFood entity instances. Position is [x, y, z]; z is ignored."""
    instances = []
    for h in layer.get("health_foods", []):
        wx, wy = float(h[0]), float(h[1])
        px = reverse_center(wx, wy, origin_x, origin_y, rows)
        instances.append(make_entity(
            identifier="HealthFood",
            def_uid=13,
            width=18, height=18,
            px=px,
            pivot_x=0.5, pivot_y=0.5,
            cols=layer["cols"], rows=rows,
            fields=[],
            level_id=level_id,
        ))
    return instances


def build_doors(layer: dict, origin_x: float, origin_y: float, rows: int, level_id: int) -> list:
    """Build Door entity instances."""
    instances = []
    for d in layer.get("doors", []):
        wx, wy = float(d["x"]), float(d["y"])
        px = reverse_surface(wx, wy, origin_x, origin_y, rows)
        fields = [
            field_int("target_layer", 60, int(d.get("target_layer", 1))),
        ]
        instances.append(make_entity(
            identifier="Door",
            def_uid=14,
            width=18, height=36,
            px=px,
            pivot_x=0.5, pivot_y=1.0,
            cols=layer["cols"], rows=rows,
            fields=fields,
            level_id=level_id,
        ))
    return instances


def build_gate(layer: dict, origin_x: float, origin_y: float, rows: int, level_id: int) -> list:
    """Build Gate entity instances.

    Gate position (world-space):
        gate_x = origin_x + gate_col * TILE_SIZE + TILE_SIZE / 2   (tile centre of gate_col)
        gate_y = origin_y + 3 * TILE_SIZE                           (ground_top for surface anchor)
                 = origin_y + rows-that-are-underground * TILE_SIZE
    In the compiled source the ground row is game_row=0 (bottom), so the
    surface anchor wy = origin_y + (0 + 1) * TILE_SIZE = origin_y + TILE_SIZE.
    However the Gate is 72px tall (4 tiles) and sits on the ground like the
    player — surface anchor means its bottom edge is at ground level.
    The game spawns it at:
        gate_y = origin_y + 1 * TILE_SIZE      (same formula as surface entities)
    Which equals origin_y + TILE_SIZE.
    But looking at the actual Rust code the gate is placed at the ground surface:
        wy = origin_y + (game_row + 1) * TILE_SIZE  with game_row = 0
           = origin_y + TILE_SIZE
    """
    gate_col = layer.get("gate_col")
    if gate_col is None:
        return []
    stars_required = layer.get("stars_required", 10)
    # World-space position (bottom-centre of gate)
    gate_wx = origin_x + gate_col * TILE_SIZE + TILE_SIZE / 2.0
    gate_wy = origin_y + TILE_SIZE  # game_row=0, surface anchor
    px = reverse_surface(gate_wx, gate_wy, origin_x, origin_y, rows)
    fields = [
        field_int("gate_col",       70, int(gate_col)),
        field_int("stars_required", 71, int(stars_required) if stars_required is not None else 10),
    ]
    return [make_entity(
        identifier="Gate",
        def_uid=15,
        width=36, height=72,
        px=px,
        pivot_x=0.5, pivot_y=1.0,
        cols=layer["cols"], rows=rows,
        fields=fields,
        level_id=level_id,
    )]


def build_exit(layer: dict, origin_x: float, origin_y: float, rows: int, level_id: int) -> list:
    """Build Exit entity instances.

    The Rust spawner places the Exit door 40 world-units to the right of the
    Gate centre.  gate_x = origin_x + gate_col * TILE_SIZE + TILE_SIZE/2.
    exit_x = gate_x + 40.
    exit_y = same surface anchor as gate.
    """
    exit_next = layer.get("exit_next_level")
    if exit_next is None:
        return []
    gate_col = layer.get("gate_col")
    if gate_col is None:
        return []
    gate_wx = origin_x + gate_col * TILE_SIZE + TILE_SIZE / 2.0
    exit_wx = gate_wx + 40.0
    exit_wy = origin_y + TILE_SIZE  # game_row=0, surface anchor
    px = reverse_surface(exit_wx, exit_wy, origin_x, origin_y, rows)
    fields = [
        field_string("exit_next_level", 80, exit_next),
    ]
    return [make_entity(
        identifier="Exit",
        def_uid=16,
        width=18, height=36,
        px=px,
        pivot_x=0.5, pivot_y=1.0,
        cols=layer["cols"], rows=rows,
        fields=fields,
        level_id=level_id,
    )]


def build_props(layer: dict, origin_x: float, origin_y: float, rows: int, level_id: int) -> list:
    """Build Prop entity instances.

    Prop positions in compiled JSON are world-space centre coordinates.
    z in the compiled JSON → z_depth field.
    """
    instances = []
    for p in layer.get("props", []):
        wx, wy = float(p["x"]), float(p["y"])
        px = reverse_center(wx, wy, origin_x, origin_y, rows)
        fields = [
            field_string("model_id",  81, p.get("model_id", "")),
            field_float("scale_x",   82, float(p.get("scale_x", 1.0))),
            field_float("scale_z",   83, float(p.get("scale_z", 1.0))),
            field_float("z_depth",   84, float(p.get("z", -15.0))),
            field_float("rotation_y", 85, float(p.get("rotation_y", 0.0))),
            field_float("scale_y",   100, float(p.get("scale_y", 1.0))),
            field_bool("foreground", 101, bool(p.get("foreground", False))),
        ]
        instances.append(make_entity(
            identifier="Prop",
            def_uid=17,
            width=18, height=18,
            px=px,
            pivot_x=0.5, pivot_y=0.5,
            cols=layer["cols"], rows=rows,
            fields=fields,
            level_id=level_id,
        ))
    return instances


def build_lights(layer: dict, origin_x: float, origin_y: float, rows: int, level_id: int) -> list:
    """Build Light entity instances.

    Light pivot is [0, 0] (top-left corner), so we use reverse_center with an
    adjustment — but actually the Rust converter uses px_to_world for lights,
    which is the centre-based formula.  We therefore reverse_center and the
    resulting px is interpreted as the pivot (top-left) position.

    In practice the game places the light at the world-space position from the
    compiled JSON, which is the light's origin point.  We keep symmetry with
    how the Rust code converts lights and use reverse_center so the round-trip
    is correct.
    """
    instances = []
    for lt in layer.get("lights", []):
        wx, wy = float(lt["x"]), float(lt["y"])
        px = reverse_center(wx, wy, origin_x, origin_y, rows)
        color_hex = rgb_floats_to_hex(lt.get("color", [1.0, 1.0, 1.0]))
        fields = [
            field_string_nullable("color",     103, color_hex),
            field_float("intensity",           104, float(lt.get("intensity", 100000.0))),
            field_float("z_depth",             105, float(lt.get("z", 3.0))),
            # radius and range are not in the compiled JSON — use field defaults
            field_float("radius",              106, 0.5),
            field_float("range",               107, 200.0),
        ]
        instances.append(make_entity(
            identifier="Light",
            def_uid=102,
            width=18, height=18,
            px=px,
            pivot_x=0.0, pivot_y=0.0,
            cols=layer["cols"], rows=rows,
            fields=fields,
            level_id=level_id,
        ))
    return instances


# ---------------------------------------------------------------------------
# Layer instance builders
# ---------------------------------------------------------------------------

def make_entities_layer(entities: list, cols: int, rows: int, level_id: int) -> dict:
    """Create a complete Entities layer instance."""
    return {
        "__identifier": "Entities",
        "__type": "Entities",
        "__cWid": cols,
        "__cHei": rows,
        "__gridSize": int(TILE_SIZE),
        "__opacity": 1,
        "__pxTotalOffsetX": 0,
        "__pxTotalOffsetY": 0,
        "__tilesetDefUid": None,
        "__tilesetRelPath": None,
        "iid": make_iid(),
        "levelId": level_id,
        "layerDefUid": 1,
        "pxOffsetX": 0,
        "pxOffsetY": 0,
        "visible": True,
        "optionalRules": [],
        "intGridCsv": [],
        "autoLayerTiles": [],
        "seed": 8345671,
        "overrideTilesetUid": None,
        "gridTiles": [],
        "entityInstances": entities,
    }


def make_tiles_layer(intgrid_csv: list, cols: int, rows: int, level_id: int) -> dict:
    """Create a complete Tiles (IntGrid) layer instance."""
    return {
        "__identifier": "Tiles",
        "__type": "IntGrid",
        "__cWid": cols,
        "__cHei": rows,
        "__gridSize": int(TILE_SIZE),
        "__opacity": 1,
        "__pxTotalOffsetX": 0,
        "__pxTotalOffsetY": 0,
        "__tilesetDefUid": None,
        "__tilesetRelPath": None,
        "iid": make_iid(),
        "levelId": level_id,
        "layerDefUid": 2,
        "pxOffsetX": 0,
        "pxOffsetY": 0,
        "visible": True,
        "optionalRules": [],
        "intGridCsv": intgrid_csv,
        "autoLayerTiles": [],
        "seed": 9182736,
        "overrideTilesetUid": None,
        "gridTiles": [],
        "entityInstances": [],
    }


# ---------------------------------------------------------------------------
# Full LDtk level builder
# ---------------------------------------------------------------------------

def create_ldtk_level(
    name: str,
    uid: int,
    compiled_layer: dict,
    world_x: int,
    world_y: int,
) -> dict:
    """Create a complete LDtk level dict from a compiled_levels layer.

    Args:
        name:          LDtk identifier for this level (e.g. "Forest_Cave")
        uid:           Unique integer uid for the level
        compiled_layer: One entry from compiled_levels.json layers array
        world_x/y:     Where to place this level in LDtk world layout (px)
    """
    cols       = compiled_layer["cols"]
    rows       = compiled_layer["rows"]
    origin_x   = float(compiled_layer["origin_x"])
    origin_y   = float(compiled_layer["origin_y"])
    px_wid     = cols * int(TILE_SIZE)
    px_hei     = rows * int(TILE_SIZE)

    # --- IntGrid ---
    raw_tiles  = compiled_layer.get("tiles", [])
    intgrid    = tiles_to_intgrid(raw_tiles, cols, rows)

    # --- Entities ---
    entities: list = []
    entities += build_spawn(compiled_layer, origin_x, origin_y, rows, uid)
    entities += build_enemies(compiled_layer, origin_x, origin_y, rows, uid)
    entities += build_stars(compiled_layer, origin_x, origin_y, rows, uid)
    entities += build_health_foods(compiled_layer, origin_x, origin_y, rows, uid)
    entities += build_doors(compiled_layer, origin_x, origin_y, rows, uid)
    entities += build_gate(compiled_layer, origin_x, origin_y, rows, uid)
    entities += build_exit(compiled_layer, origin_x, origin_y, rows, uid)
    entities += build_props(compiled_layer, origin_x, origin_y, rows, uid)
    entities += build_lights(compiled_layer, origin_x, origin_y, rows, uid)

    # --- Layer instances (Entities first, then Tiles — matches LDtk convention) ---
    layer_instances = [
        make_entities_layer(entities, cols, rows, uid),
        make_tiles_layer(intgrid, cols, rows, uid),
    ]

    # --- Level field instances (OriginX / OriginY) ---
    field_instances = [
        {
            "__identifier": "OriginX",
            "__type": "Float",
            "__value": origin_x,
            "__tile": None,
            "defUid": 90,
            "realEditorValues": [{"id": "V_Float", "params": [origin_x]}],
        },
        {
            "__identifier": "OriginY",
            "__type": "Float",
            "__value": origin_y,
            "__tile": None,
            "defUid": 91,
            "realEditorValues": [{"id": "V_Float", "params": [origin_y]}],
        },
    ]

    return {
        "identifier": name,
        "iid": make_iid(),
        "uid": uid,
        "worldX": world_x,
        "worldY": world_y,
        "worldDepth": 0,
        "pxWid": px_wid,
        "pxHei": px_hei,
        "__bgColor": "#1B1B2E",
        "bgColor": None,
        "useAutoIdentifier": False,
        "bgRelPath": None,
        "bgPos": None,
        "bgPivotX": 0.5,
        "bgPivotY": 0.5,
        "__smartColor": "#82828C",
        "__bgPos": None,
        "externalRelPath": None,
        "fieldInstances": field_instances,
        "layerInstances": layer_instances,
        "__neighbours": [],
    }


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

# LDtk level naming per compiled level-id and layer index
LAYER_NAMES = {
    "Forest": {
        0: "Forest",
        1: "Forest_Cave",
    },
    "Subdivision": {
        0: "Subdivision",
        1: "Subdivision_Sewer",
        2: "Subdivision_Rooftop",
    },
    "City": {
        0: "City",
        1: "City_Subway",
        2: "City_Rooftop",
    },
}

# World-layout positions for each LDtk level (purely visual in LDtk editor).
# Main layers placed in a row, sub-levels offset below.
WORLD_POSITIONS = {
    "Forest":               (0,     0),
    "Forest_Cave":          (2000,  0),
    "Subdivision":          (0,     1000),
    "Subdivision_Sewer":    (2000,  1000),
    "Subdivision_Rooftop":  (4000,  1000),
    "City":                 (0,     2000),
    "City_Subway":          (2000,  2000),
    "City_Rooftop":         (4000,  2000),
}


def main():
    # ------------------------------------------------------------------
    # Load source files
    # ------------------------------------------------------------------
    if not COMPILED_JSON.exists():
        print(f"ERROR: compiled_levels.json not found at {COMPILED_JSON}", file=sys.stderr)
        sys.exit(1)
    if not LDTK_FILE.exists():
        print(f"ERROR: jasperworld.ldtk not found at {LDTK_FILE}", file=sys.stderr)
        sys.exit(1)

    with open(COMPILED_JSON, "r", encoding="utf-8") as f:
        compiled = json.load(f)

    with open(LDTK_FILE, "r", encoding="utf-8") as f:
        ldtk = json.load(f)

    # ------------------------------------------------------------------
    # Build all 9 LDtk levels
    # ------------------------------------------------------------------
    next_uid   = ldtk.get("nextUid", 200)
    new_levels = []

    for compiled_level in compiled["levels"]:
        level_id = compiled_level["id"]
        if level_id not in LAYER_NAMES:
            print(f"WARNING: Unknown level id '{level_id}' — skipping", file=sys.stderr)
            continue

        for compiled_layer in compiled_level["layers"]:
            layer_idx = compiled_layer["id"]
            if layer_idx not in LAYER_NAMES[level_id]:
                print(
                    f"WARNING: No name mapping for {level_id} layer {layer_idx} — skipping",
                    file=sys.stderr,
                )
                continue

            ldtk_name    = LAYER_NAMES[level_id][layer_idx]
            world_x, world_y = WORLD_POSITIONS.get(ldtk_name, (0, 0))
            level_uid    = next_uid
            next_uid    += 1

            ldtk_level = create_ldtk_level(
                name=ldtk_name,
                uid=level_uid,
                compiled_layer=compiled_layer,
                world_x=world_x,
                world_y=world_y,
            )
            new_levels.append(ldtk_level)

            entity_count = len(
                ldtk_level["layerInstances"][0]["entityInstances"]
            )
            print(
                f"  {ldtk_name:30s}  uid={level_uid}"
                f"  cols={compiled_layer['cols']:3d}  rows={compiled_layer['rows']:3d}"
                f"  entities={entity_count}"
            )

    # ------------------------------------------------------------------
    # Patch the LDtk project and write
    # ------------------------------------------------------------------
    ldtk["levels"]  = new_levels
    ldtk["nextUid"] = next_uid

    with open(LDTK_FILE, "w", encoding="utf-8") as f:
        json.dump(ldtk, f, indent="\t")
        f.write("\n")

    print(f"\nWrote {len(new_levels)} levels to {LDTK_FILE}")


if __name__ == "__main__":
    main()
