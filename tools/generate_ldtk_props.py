#!/usr/bin/env python3
"""
generate_ldtk_props.py

Adds 25 per-prop entity definitions to levels/jasperworld.ldtk.
Each definition mirrors the structure of the generic "Prop" entity (uid 17)
but pre-fills default field values for a specific model.

Idempotent: if Prop_* entities already exist they are replaced in-place.
The file's top-level `nextUid` field is updated to reflect allocated uids.
"""

import json
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------
REPO_ROOT = Path(__file__).resolve().parent.parent
LDTK_PATH = REPO_ROOT / "levels" / "jasperworld.ldtk"

# ---------------------------------------------------------------------------
# Prop table
# (identifier, model_id, scale_x, scale_y, scale_z, z_depth, rotation_y, foreground, color)
# ---------------------------------------------------------------------------
PROPS = [
    ("Prop_TreeOak",       "models/tree_oak.glb",                        85,    85,    1,    10,      0,       True,  "#4CAF50"),
    ("Prop_TreePine",      "models/tree_pine.glb",                       90,    90,    1,    10,      0,       True,  "#4CAF50"),
    ("Prop_TreeFat",       "models/tree_fat.glb",                        80,    80,    1,    10,      0,       True,  "#4CAF50"),
    ("Prop_TreeDefault",   "models/tree_default.glb",                    70,    70,    1,    10,      0,       True,  "#4CAF50"),
    ("Prop_TreeSubLg",     "models/suburban/tree-suburban-large.glb",   180,   180,    1,    10,      0,       True,  "#4CAF50"),
    ("Prop_TreeSubSm",     "models/suburban/tree-suburban-small.glb",   140,   140,    1,    10,      0,       True,  "#4CAF50"),
    ("Prop_PlantBush",     "models/plant_bush.glb",                      49,    49,   27,   -15,      0,       True,  "#8BC34A"),
    ("Prop_BushLarge",     "models/plant_bushLarge.glb",                 79,    79,   43,   -15,      0,       True,  "#8BC34A"),
    ("Prop_GrassLarge",    "models/grass_large.glb",                     38,    38,   12,   -15,      0,       True,  "#8BC34A"),
    ("Prop_YellowFlower",  "models/yellow_flower.glb",                    5,    20,   20,   -15, -1.571,       True,  "#FFEB3B"),
    ("Prop_FlowerRed",     "models/flower_redA.glb",                     66,    66,   17,   -15,      0,       True,  "#F44336"),
    ("Prop_LargeRock",     "models/large_rock.glb",                       9,    28,   28,   -15, -1.571,       True,  "#9E9E9E"),
    ("Prop_SmallRock",     "models/small_rock.glb",                       5,    13,   13,   -15, -1.571,       True,  "#9E9E9E"),
    ("Prop_MushroomRed",   "models/mushroom_red.glb",                    30,    30,   12,     3,      0,       False, "#F44336"),
    ("Prop_MushroomTan",   "models/mushroom_tan.glb",                    25,    25,   10,     3,      0,       False, "#FFE0B2"),
    ("Prop_Mushrooms",     "models/mushrooms.glb",                       28,    28,   11,     3,      0,       False, "#FFE0B2"),
    ("Prop_Taxi",          "models/city/taxi.glb",                       90,    90,   90,   -15,      0,       True,  "#2196F3"),
    ("Prop_ConstrCone",    "models/city/construction-cone.glb",        12.6,  12.6, 12.6,  -15,      0,       True,  "#FF9800"),
    ("Prop_LightCurved",   "models/city/light-curved.glb",               70,    70,   16,   -15,      0,       True,  "#2196F3"),
    ("Prop_CaveRock",      "models/cave/cliff_cave_rock.glb",            10,    10,    3,     3,      0,       False, "#795548"),
    ("Prop_CaveStone",     "models/cave/cliff_cave_stone.glb",           12,    12,    4,     3,      0,       False, "#795548"),
    ("Prop_SewerBrick",    "models/sewer/brick-wall.glb",                16,    16,    4,     3,      0,       False, "#607D8B"),
    ("Prop_SewerColumn",   "models/sewer/column-large.glb",              18,    18,    6,     3,      0,       False, "#607D8B"),
    ("Prop_SewerFence",    "models/sewer/iron-fence.glb",                20,    20,    5,     3,      0,       False, "#607D8B"),
    ("Prop_SewerStoneCol", "models/sewer/stone-wall-column.glb",         18,    18,    6,     3,      0,       False, "#607D8B"),
]

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def find_all_uids(obj):
    """Recursively collect every integer value stored under the key 'uid'."""
    uids = []
    if isinstance(obj, dict):
        for k, v in obj.items():
            if k == "uid" and isinstance(v, int):
                uids.append(v)
            else:
                uids.extend(find_all_uids(v))
    elif isinstance(obj, list):
        for item in obj:
            uids.extend(find_all_uids(item))
    return uids


def make_field_def(identifier, field_type, uid, default_override, *, auto_chain=False, allowed_refs="Any", allowed_refs_only_same=False):
    """Build a single fieldDef dict matching the LDtk schema used by 'Prop'."""
    if field_type == "F_Float":
        display_type = "Float"
    elif field_type == "F_String":
        display_type = "String"
    elif field_type == "F_Bool":
        display_type = "Bool"
    else:
        display_type = field_type

    return {
        "identifier": identifier,
        "doc": None,
        "__type": display_type,
        "uid": uid,
        "type": field_type,
        "isArray": False,
        "canBeNull": False,
        "arrayMinLength": None,
        "arrayMaxLength": None,
        "editorDisplayMode": "Hidden",
        "editorDisplayScale": 1,
        "editorDisplayPos": "Above",
        "editorLinkStyle": "StraightArrow",
        "editorDisplayColor": None,
        "editorAlwaysShow": False,
        "editorShowInWorld": True,
        "editorCutLongValues": True,
        "editorTextSuffix": None,
        "editorTextPrefix": None,
        "useForSmartColor": False,
        "exportToToc": False,
        "searchable": False,
        "min": None,
        "max": None,
        "regex": None,
        "acceptFileTypes": None,
        "defaultOverride": default_override,
        "textLanguageMode": None,
        "symmetricalRef": False,
        # scale_y and foreground in the original Prop use autoChainRef/allowedRefs overrides
        "autoChainRef": auto_chain,
        "allowOutOfLevelRef": auto_chain,
        "allowedRefs": "OnlySame" if allowed_refs_only_same else "Any",
        "allowedRefsEntityUid": None,
        "allowedRefTags": [],
        "tilesetUid": None,
    }


def make_entity_def(identifier, model_id, scale_x, scale_y, scale_z, z_depth, rotation_y, foreground, color, entity_uid, first_field_uid):
    """Build a complete entity definition dict for one prop type."""
    f = first_field_uid  # convenience alias

    field_defs = [
        make_field_def("model_id",   "F_String", f + 0,
                       {"id": "V_String", "params": [model_id]}),
        make_field_def("scale_x",    "F_Float",  f + 1,
                       {"id": "V_Float",  "params": [scale_x]}),
        make_field_def("scale_z",    "F_Float",  f + 2,
                       {"id": "V_Float",  "params": [scale_z]}),
        make_field_def("z_depth",    "F_Float",  f + 3,
                       {"id": "V_Float",  "params": [z_depth]}),
        make_field_def("rotation_y", "F_Float",  f + 4,
                       {"id": "V_Float",  "params": [rotation_y]}),
        # scale_y: mirrors original — autoChainRef=True, allowedRefs="OnlySame"
        make_field_def("scale_y",    "F_Float",  f + 5,
                       {"id": "V_Float",  "params": [scale_y]},
                       auto_chain=True, allowed_refs_only_same=True),
        # foreground: mirrors original — autoChainRef=True, allowedRefs="OnlySame"
        make_field_def("foreground", "F_Bool",   f + 6,
                       {"id": "V_Bool",   "params": [foreground]},
                       auto_chain=True, allowed_refs_only_same=True),
    ]

    return {
        "identifier": identifier,
        "uid": entity_uid,
        "tags": [],
        "exportToToc": False,
        "allowOutOfBounds": False,
        "doc": None,
        "width": 18,
        "height": 18,
        "resizableX": False,
        "resizableY": False,
        "minWidth": None,
        "maxWidth": None,
        "minHeight": None,
        "maxHeight": None,
        "keepAspectRatio": False,
        "tileOpacity": 1,
        "fillOpacity": 0.08,
        "lineOpacity": 1,
        "hollow": False,
        "color": color,
        "renderMode": "Rectangle",
        "showName": True,
        "tilesetId": None,
        "tileRenderMode": "FitInside",
        "tileRect": None,
        "uiTileRect": None,
        "nineSliceBorders": [],
        "maxCount": 0,
        "limitScope": "PerLevel",
        "limitBehavior": "MoveLastOne",
        "pivotX": 0.5,
        "pivotY": 1,
        "fieldDefs": field_defs,
    }


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    if not LDTK_PATH.exists():
        print(f"ERROR: LDtk file not found at {LDTK_PATH}", file=sys.stderr)
        sys.exit(1)

    print(f"Reading {LDTK_PATH} …")
    with LDTK_PATH.open("r", encoding="utf-8") as fh:
        data = json.load(fh)

    # ------------------------------------------------------------------
    # 1. Find highest uid currently in the file (both uid fields and nextUid)
    # ------------------------------------------------------------------
    all_uids = find_all_uids(data)
    # Also respect the file's own nextUid tracking field
    file_next_uid = data.get("nextUid", 0)
    max_uid = max(max(all_uids, default=0), file_next_uid - 1)

    print(f"  Max uid found in file: {max_uid}  (nextUid header: {file_next_uid})")

    # ------------------------------------------------------------------
    # 2. Identify existing Prop_* entities so we can replace them
    # ------------------------------------------------------------------
    entities = data["defs"]["entities"]
    existing_identifiers = {e["identifier"]: idx for idx, e in enumerate(entities)}
    existing_prop_names = {name for name in existing_identifiers if name.startswith("Prop_")}

    if existing_prop_names:
        print(f"  Found {len(existing_prop_names)} existing Prop_* entity defs — they will be replaced.")

    # ------------------------------------------------------------------
    # 3. Allocate uid block for new / replaced entities
    #    Each entity needs: 1 entity uid + 7 field uids = 8 uids
    # ------------------------------------------------------------------
    # When replacing an existing entity we re-use its entity uid but
    # allocate fresh field uids so we never collide.  The simplest safe
    # approach: always allocate a fresh block starting at max_uid + 1.
    # ------------------------------------------------------------------
    next_free = max_uid + 1

    # Build replacement/new entity defs
    new_entities: list[dict] = []
    replaced = []
    added = []

    for row in PROPS:
        identifier, model_id, scale_x, scale_y, scale_z, z_depth, rotation_y, foreground, color = row

        if identifier in existing_identifiers:
            # Re-use the existing entity uid so level instances keep working
            entity_uid = entities[existing_identifiers[identifier]]["uid"]
            replaced.append(identifier)
        else:
            entity_uid = next_free
            next_free += 1
            added.append(identifier)

        first_field_uid = next_free
        next_free += 7  # 7 fields per entity

        new_entities.append(
            make_entity_def(
                identifier, model_id,
                scale_x, scale_y, scale_z,
                z_depth, rotation_y, foreground, color,
                entity_uid, first_field_uid,
            )
        )

    # ------------------------------------------------------------------
    # 4. Splice new definitions into defs.entities right after "Prop" (uid 17)
    # ------------------------------------------------------------------
    # Remove all existing Prop_* entries first (they will be re-inserted)
    entities_cleaned = [e for e in entities if e["identifier"] not in existing_prop_names]

    # Find insertion point: after the "Prop" entity (uid 17)
    insert_after_idx = None
    for idx, e in enumerate(entities_cleaned):
        if e["identifier"] == "Prop" and e["uid"] == 17:
            insert_after_idx = idx
            break

    if insert_after_idx is None:
        # Fallback: append at the end of the entity list
        print("  WARNING: Could not find 'Prop' (uid 17) — appending new entities at end.")
        updated_entities = entities_cleaned + new_entities
    else:
        updated_entities = (
            entities_cleaned[: insert_after_idx + 1]
            + new_entities
            + entities_cleaned[insert_after_idx + 1 :]
        )

    data["defs"]["entities"] = updated_entities

    # ------------------------------------------------------------------
    # 5. Update the file's nextUid tracker
    # ------------------------------------------------------------------
    data["nextUid"] = next_free
    print(f"  Updated nextUid: {file_next_uid} → {next_free}")

    # ------------------------------------------------------------------
    # 6. Write back
    # ------------------------------------------------------------------
    with LDTK_PATH.open("w", encoding="utf-8") as fh:
        json.dump(data, fh, indent="\t", ensure_ascii=False)
        fh.write("\n")  # trailing newline to match LDtk convention

    # ------------------------------------------------------------------
    # Summary
    # ------------------------------------------------------------------
    print("\n=== Summary ===")
    if added:
        print(f"  Added   ({len(added)}): {', '.join(added)}")
    if replaced:
        print(f"  Replaced ({len(replaced)}): {', '.join(replaced)}")
    print(f"  Total Prop_* entity defs now in file: {len(new_entities)}")
    print(f"  UIDs allocated this run: {max_uid + 1} – {next_free - 1}")
    print(f"Done. Wrote {LDTK_PATH}")


if __name__ == "__main__":
    main()
