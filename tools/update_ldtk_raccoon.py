#!/usr/bin/env python3
"""
update_ldtk_raccoon.py

Adds the Prop_RaccoonFamily entity definition to defs.entities,
updates nextUid to 542, and places one instance in the Sanctuary level.
"""

import json
import shutil
import uuid
from pathlib import Path

LDTK_PATH = Path(__file__).parent.parent / "levels" / "jasperworld.ldtk"
BACKUP_PATH = LDTK_PATH.parent / "jasperworld.ldtk.bak3"

# ── 1. Backup ──────────────────────────────────────────────────────────────────
shutil.copy2(LDTK_PATH, BACKUP_PATH)
print(f"Backup written to {BACKUP_PATH}")

# ── 2. Load ────────────────────────────────────────────────────────────────────
with open(LDTK_PATH, "r", encoding="utf-8") as fh:
    data = json.load(fh)

# ── 3. Build entity definition ─────────────────────────────────────────────────
# Use Prop_Water (uid=526) as structural template; override all values.

def make_field_def(identifier, uid, ftype, py_type, default_value):
    """Build a fieldDef entry matching the LDtk format seen in Prop_Water."""
    if ftype == "F_String":
        ldtk_type = "String"
        default_override = {"id": "V_String", "params": [default_value]}
    elif ftype == "F_Float":
        ldtk_type = "Float"
        default_override = {"id": "V_Float", "params": [float(default_value)]}
    elif ftype == "F_Bool":
        ldtk_type = "Bool"
        default_override = {"id": "V_Bool", "params": [bool(default_value)]}
    else:
        raise ValueError(f"Unknown field type: {ftype}")

    return {
        "identifier": identifier,
        "doc": None,
        "__type": ldtk_type,
        "uid": uid,
        "type": ftype,
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
        "tilesetUid": None,
    }


raccoon_entity_def = {
    "identifier": "Prop_RaccoonFamily",
    "uid": 534,
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
    "color": "#FF9800",
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
        make_field_def("model_id",   535, "F_String", str,   "models/sanctuary/raccoon_family.png"),
        make_field_def("scale_x",    536, "F_Float",  float, 54.0),
        make_field_def("scale_z",    537, "F_Float",  float, 1.0),
        make_field_def("z_depth",    538, "F_Float",  float, 1.5),
        make_field_def("rotation_y", 539, "F_Float",  float, 0.0),
        make_field_def("scale_y",    540, "F_Float",  float, 54.0),
        make_field_def("foreground", 541, "F_Bool",   bool,  False),
    ],
}

# Guard: don't add duplicate
existing_uids = {e["uid"] for e in data["defs"]["entities"]}
existing_ids  = {e["identifier"] for e in data["defs"]["entities"]}
if 534 in existing_uids or "Prop_RaccoonFamily" in existing_ids:
    print("WARNING: Prop_RaccoonFamily already exists in defs.entities — skipping definition insert.")
else:
    data["defs"]["entities"].append(raccoon_entity_def)
    print("Added Prop_RaccoonFamily to defs.entities")

# ── 4. Update nextUid ──────────────────────────────────────────────────────────
old_next_uid = data.get("nextUid")
data["nextUid"] = 542
print(f"Updated nextUid: {old_next_uid} → 542")

# ── 5. Build entity instance ───────────────────────────────────────────────────
# fieldInstances order matches fieldDefs order (uid 535–541)
instance_iid = str(uuid.uuid4())

raccoon_instance = {
    "__identifier": "Prop_RaccoonFamily",
    "__grid": [42, 17],
    "__pivot": [0.5, 1],
    "__tags": [],
    "__tile": None,
    "__smartColor": "#FF9800",
    "iid": instance_iid,
    "width": 18,
    "height": 18,
    "defUid": 534,
    "px": [765, 315],
    "fieldInstances": [
        {
            "__identifier": "model_id",
            "__type": "String",
            "__value": "models/sanctuary/raccoon_family.png",
            "__tile": None,
            "defUid": 535,
            "realEditorValues": [
                {"id": "V_String", "params": ["models/sanctuary/raccoon_family.png"]}
            ],
        },
        {
            "__identifier": "scale_x",
            "__type": "Float",
            "__value": 54.0,
            "__tile": None,
            "defUid": 536,
            "realEditorValues": [{"id": "V_Float", "params": [54.0]}],
        },
        {
            "__identifier": "scale_z",
            "__type": "Float",
            "__value": 1.0,
            "__tile": None,
            "defUid": 537,
            "realEditorValues": [{"id": "V_Float", "params": [1.0]}],
        },
        {
            "__identifier": "z_depth",
            "__type": "Float",
            "__value": 1.5,
            "__tile": None,
            "defUid": 538,
            "realEditorValues": [{"id": "V_Float", "params": [1.5]}],
        },
        {
            "__identifier": "rotation_y",
            "__type": "Float",
            "__value": 0.0,
            "__tile": None,
            "defUid": 539,
            "realEditorValues": [{"id": "V_Float", "params": [0.0]}],
        },
        {
            "__identifier": "scale_y",
            "__type": "Float",
            "__value": 54.0,
            "__tile": None,
            "defUid": 540,
            "realEditorValues": [{"id": "V_Float", "params": [54.0]}],
        },
        {
            "__identifier": "foreground",
            "__type": "Bool",
            "__value": False,
            "__tile": None,
            "defUid": 541,
            "realEditorValues": [{"id": "V_Bool", "params": [False]}],
        },
    ],
    "__worldX": 765,
    "__worldY": 315,
}

# ── 6. Insert instance into Sanctuary / Entities layer ─────────────────────────
inserted = False
for level in data["levels"]:
    if level["identifier"] == "Sanctuary":
        for layer in level["layerInstances"]:
            if layer["__identifier"] == "Entities":
                # Guard: don't add duplicate iid or same position+defUid
                already = any(
                    inst["defUid"] == 534 and inst["px"] == [765, 315]
                    for inst in layer["entityInstances"]
                )
                if already:
                    print("WARNING: Prop_RaccoonFamily instance already exists at px=[765,315] — skipping instance insert.")
                else:
                    layer["entityInstances"].append(raccoon_instance)
                    inserted = True
                    print(f"Appended Prop_RaccoonFamily instance (iid={instance_iid}) to Sanctuary/Entities")
                break
        break

if not inserted and not already:
    print("ERROR: Could not find Sanctuary level or Entities layer!")

# ── 7. Write back ──────────────────────────────────────────────────────────────
with open(LDTK_PATH, "w", encoding="utf-8") as fh:
    json.dump(data, fh, indent="\t")
print(f"Wrote updated LDtk file to {LDTK_PATH}")

# ── 8. Validate ────────────────────────────────────────────────────────────────
with open(LDTK_PATH, "r", encoding="utf-8") as fh:
    check = json.load(fh)

# Verify entity def
found_def = next((e for e in check["defs"]["entities"] if e["identifier"] == "Prop_RaccoonFamily"), None)
assert found_def is not None, "Prop_RaccoonFamily def missing after write!"
assert found_def["uid"] == 534
assert len(found_def["fieldDefs"]) == 7
assert check["nextUid"] == 542

# Verify instance
found_inst = None
for level in check["levels"]:
    if level["identifier"] == "Sanctuary":
        for layer in level["layerInstances"]:
            if layer["__identifier"] == "Entities":
                found_inst = next(
                    (i for i in layer["entityInstances"] if i["defUid"] == 534),
                    None,
                )
                break
        break

assert found_inst is not None, "Prop_RaccoonFamily instance missing after write!"
assert found_inst["px"] == [765, 315]
assert len(found_inst["fieldInstances"]) == 7

print("\n=== Validation passed ===")
print(f"  Entity def uid=534, fieldDefs={len(found_def['fieldDefs'])}")
print(f"  nextUid={check['nextUid']}")
print(f"  Instance px={found_inst['px']}, iid={found_inst['iid']}")
print(f"  Total entity defs: {len(check['defs']['entities'])}")
