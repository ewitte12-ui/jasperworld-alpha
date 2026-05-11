#!/usr/bin/env python3
"""
update_ldtk_props.py

Adds Prop_Water entity definition and updates the Sanctuary level's Entities
layer with cherry blossom + water entity instances in jasperworld.ldtk.

Steps:
  1. Load levels/jasperworld.ldtk
  2. Add Prop_Water entity def (uid=526, fields 527-533)
  3. Set Prop_CherryBlossom (uid=508) allowOutOfBounds = true
  4. Append 5 entity instances to the Sanctuary level's Entities layer
  5. Update nextUid to 534
  6. Write back atomically
"""

import json
import uuid
import sys
import os

LDTK_PATH = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "levels", "jasperworld.ldtk"
)

print(f"Loading: {LDTK_PATH}")
with open(LDTK_PATH, "r", encoding="utf-8") as f:
    data = json.load(f)


# ---------------------------------------------------------------------------
# Helper: build a fieldDef entry
# ---------------------------------------------------------------------------
def make_field_def(identifier, uid, ftype, default_val, is_bool=False, is_string=False):
    if is_bool:
        default_override = {"id": "V_Bool", "params": [default_val]}
    elif is_string:
        default_override = {"id": "V_String", "params": [default_val]}
    else:
        default_override = {"id": "V_Float", "params": [default_val]}

    return {
        "identifier": identifier,
        "doc": None,
        "__type": ftype,
        "uid": uid,
        "type": "F_Bool" if is_bool else ("F_String" if is_string else "F_Float"),
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
        "autoChainRef": False,
        "allowOutOfLevelRef": False,
        "allowedRefs": "Any",
        "allowedRefsEntityUid": None,
        "allowedRefTags": [],
        "tilesetUid": None
    }


# ---------------------------------------------------------------------------
# Step 2: Add Prop_Water entity definition
# ---------------------------------------------------------------------------
prop_water_def = {
    "identifier": "Prop_Water",
    "uid": 526,
    "tags": [],
    "exportToToc": False,
    "allowOutOfBounds": True,
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
    "color": "#03A9F4",
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
    "fieldDefs": [
        make_field_def("model_id",   527, "String", "models/sanctuary/water_at_end_oflevel.png", is_string=True),
        make_field_def("scale_x",    528, "Float",  72.0),
        make_field_def("scale_y",    529, "Float",  54.0),
        make_field_def("scale_z",    530, "Float",  1.0),
        make_field_def("z_depth",    531, "Float",  0.2),
        make_field_def("rotation_y", 532, "Float",  0.0),
        make_field_def("foreground", 533, "Bool",   False, is_bool=True),
    ]
}

# Check if Prop_Water already exists (idempotency guard)
existing_uids = [e["uid"] for e in data["defs"]["entities"]]
if 526 not in existing_uids:
    data["defs"]["entities"].append(prop_water_def)
    print("Added Prop_Water entity definition (uid=526)")
else:
    print("Prop_Water (uid=526) already exists — skipping add")


# ---------------------------------------------------------------------------
# Step 3: Set Prop_CherryBlossom allowOutOfBounds = true
# ---------------------------------------------------------------------------
for ent_def in data["defs"]["entities"]:
    if ent_def["uid"] == 508:
        ent_def["allowOutOfBounds"] = True
        print(f"Set Prop_CherryBlossom (uid=508) allowOutOfBounds=true")
        break
else:
    print("WARNING: Prop_CherryBlossom (uid=508) not found in defs.entities!", file=sys.stderr)


# ---------------------------------------------------------------------------
# Helper: build a fieldInstance entry
# ---------------------------------------------------------------------------
def make_field_inst(identifier, ftype, value, def_uid, is_bool=False, is_string=False):
    if is_bool:
        v_id = "V_Bool"
    elif is_string:
        v_id = "V_String"
    else:
        v_id = "V_Float"
    return {
        "__identifier": identifier,
        "__type": ftype,
        "__value": value,
        "__tile": None,
        "defUid": def_uid,
        "realEditorValues": [{"id": v_id, "params": [value]}]
    }


# ---------------------------------------------------------------------------
# Helper: build a cherry blossom entity instance
# ---------------------------------------------------------------------------
def make_cherry_blossom(px_x, px_y, scale_x, scale_y, scale_z, z_depth, foreground, rotation_y=0.0):
    grid_x = px_x // 18
    grid_y = px_y // 18
    iid = str(uuid.uuid4())
    return {
        "__identifier": "Prop_CherryBlossom",
        "__grid": [grid_x, grid_y],
        "__pivot": [0.5, 1],
        "__tags": [],
        "__tile": None,
        "__smartColor": "#E91E63",
        "iid": iid,
        "width": 18,
        "height": 18,
        "defUid": 508,
        "px": [px_x, px_y],
        "fieldInstances": [
            make_field_inst("model_id",   "String", "models/sanctuary/tree_cherryblossom.glb", 509, is_string=True),
            make_field_inst("scale_x",    "Float",  scale_x,   510),
            make_field_inst("scale_z",    "Float",  scale_z,   511),
            make_field_inst("z_depth",    "Float",  z_depth,   512),
            make_field_inst("rotation_y", "Float",  rotation_y, 513),
            make_field_inst("scale_y",    "Float",  scale_y,   514),
            make_field_inst("foreground", "Bool",   foreground, 515, is_bool=True),
        ],
        "__worldX": px_x,
        "__worldY": 3000 + px_y
    }


# ---------------------------------------------------------------------------
# Helper: build a water entity instance
# ---------------------------------------------------------------------------
def make_water(px_x, px_y, scale_x, scale_y, scale_z, z_depth):
    grid_x = px_x // 18
    grid_y = px_y // 18
    iid = str(uuid.uuid4())
    return {
        "__identifier": "Prop_Water",
        "__grid": [grid_x, grid_y],
        "__pivot": [0.5, 1],
        "__tags": [],
        "__tile": None,
        "__smartColor": "#03A9F4",
        "iid": iid,
        "width": 18,
        "height": 18,
        "defUid": 526,
        "px": [px_x, px_y],
        "fieldInstances": [
            make_field_inst("model_id",   "String", "models/sanctuary/water_at_end_oflevel.png", 527, is_string=True),
            make_field_inst("scale_x",    "Float",  scale_x,  528),
            make_field_inst("scale_y",    "Float",  scale_y,  529),
            make_field_inst("scale_z",    "Float",  scale_z,  530),
            make_field_inst("z_depth",    "Float",  z_depth,  531),
            make_field_inst("rotation_y", "Float",  0.0,      532),
            make_field_inst("foreground", "Bool",   False,    533, is_bool=True),
        ],
        "__worldX": px_x,
        "__worldY": 3000 + px_y
    }


# ---------------------------------------------------------------------------
# Step 4: Append entity instances to Sanctuary level's Entities layer
# ---------------------------------------------------------------------------
# Coordinate conversion:
#   px_x = world_x - (-432) = world_x + 432
#   px_y = 396 - (world_y - (-200)) = 396 - world_y - 200 = 196 - world_y

def world_to_px(wx, wy):
    return (int(wx + 432), int(196 - wy))

new_instances = [
    # Cherry blossom at world (-500, -65)
    make_cherry_blossom(*world_to_px(-500, -65), scale_x=170.0, scale_y=170.0, scale_z=8.0, z_depth=10.0, foreground=True),
    # Cherry blossom at world (-225, -70)
    make_cherry_blossom(*world_to_px(-225, -70), scale_x=160.0, scale_y=160.0, scale_z=8.0, z_depth=10.0, foreground=True),
    # Cherry blossom at world (50, -60)
    make_cherry_blossom(*world_to_px(50, -60),   scale_x=180.0, scale_y=180.0, scale_z=8.0, z_depth=10.0, foreground=True),
    # Cherry blossom at world (325, -65)
    make_cherry_blossom(*world_to_px(325, -65),  scale_x=170.0, scale_y=170.0, scale_z=8.0, z_depth=10.0, foreground=True),
    # Water at world (387, -173)
    make_water(*world_to_px(387, -173), scale_x=72.0, scale_y=54.0, scale_z=1.0, z_depth=0.2),
]

# Print px coords for verification
for inst in new_instances:
    print(f"  {inst['__identifier']} px={inst['px']}")

# Find Sanctuary level (uid=525) and its Entities layer (layerDefUid=1)
sanctuary_found = False
for level in data["levels"]:
    if level["uid"] == 525:
        for layer_inst in level["layerInstances"]:
            if layer_inst["layerDefUid"] == 1:
                # Check for idempotency — only add if iids don't already exist
                existing_iids = {e["iid"] for e in layer_inst["entityInstances"]}
                added = 0
                for inst in new_instances:
                    # Check by identifier + px (don't re-add same position)
                    already_placed = any(
                        e["__identifier"] == inst["__identifier"] and e["px"] == inst["px"]
                        for e in layer_inst["entityInstances"]
                    )
                    if not already_placed:
                        layer_inst["entityInstances"].append(inst)
                        added += 1
                print(f"Added {added} entity instances to Sanctuary Entities layer")
                sanctuary_found = True
                break
        break

if not sanctuary_found:
    print("ERROR: Could not find Sanctuary level (uid=525) with Entities layer (layerDefUid=1)!", file=sys.stderr)
    sys.exit(1)


# ---------------------------------------------------------------------------
# Step 5: Update nextUid to 534
# ---------------------------------------------------------------------------
current_next = data.get("nextUid", 0)
if current_next < 534:
    data["nextUid"] = 534
    print(f"Updated nextUid: {current_next} -> 534")
else:
    print(f"nextUid already {current_next} >= 534 — no change needed")


# ---------------------------------------------------------------------------
# Write back
# ---------------------------------------------------------------------------
with open(LDTK_PATH, "w", encoding="utf-8") as f:
    json.dump(data, f, indent="\t", ensure_ascii=False)
    f.write("\n")

print(f"Successfully wrote {LDTK_PATH}")
