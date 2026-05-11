#!/usr/bin/env python3
"""
update_ldtk.py — Modifies jasperworld.ldtk to:
  1. Add 4 new Sanctuary prop entity definitions to defs.entities
  2. Fix City Exit entity to point to Sanctuary
  3. Add the Sanctuary level
"""

import json
import shutil
import uuid
from pathlib import Path

LDTK_PATH = Path("/Users/ericwitte/Documents/claude_projects/jasperworld_alpha/levels/jasperworld.ldtk")
BACKUP_PATH = LDTK_PATH.with_suffix(".ldtk.bak2")

# ─── helpers ────────────────────────────────────────────────────────────────

def make_field(identifier, ftype, uid, default_value):
    """
    Build a fieldDef dict that exactly matches the existing boilerplate.
    ftype is one of: 'String', 'Float', 'Bool'
    """
    if ftype == "String":
        ldtk_type = "F_String"
        default_override = {"id": "V_String", "params": [default_value]}
        display_type = "String"
    elif ftype == "Float":
        ldtk_type = "F_Float"
        default_override = {"id": "V_Float", "params": [default_value]}
        display_type = "Float"
    elif ftype == "Bool":
        ldtk_type = "F_Bool"
        default_override = {"id": "V_Bool", "params": [default_value]}
        display_type = "Bool"
    else:
        raise ValueError(f"Unknown ftype: {ftype}")

    return {
        "identifier": identifier,
        "doc": None,
        "__type": display_type,
        "uid": uid,
        "type": ldtk_type,
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


def make_prop_entity(uid, identifier, color, field_uids, model_id,
                     scale_x, scale_y, scale_z, z_depth, rotation_y, foreground):
    """Build a full entity def dict for a prop."""
    return {
        "identifier": identifier,
        "uid": uid,
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
        "fieldDefs": [
            make_field("model_id",   "String", field_uids[0], model_id),
            make_field("scale_x",    "Float",  field_uids[1], scale_x),
            make_field("scale_z",    "Float",  field_uids[2], scale_z),
            make_field("z_depth",    "Float",  field_uids[3], z_depth),
            make_field("rotation_y", "Float",  field_uids[4], rotation_y),
            make_field("scale_y",    "Float",  field_uids[5], scale_y),
            make_field("foreground", "Bool",   field_uids[6], foreground),
        ],
    }


def make_field_instance(identifier, ftype, value, def_uid):
    """Build a fieldInstance dict for an entity instance."""
    if ftype == "String":
        ldtk_type = "String"
        real_val = {"id": "V_String", "params": [value]}
    elif ftype == "Float":
        ldtk_type = "Float"
        real_val = {"id": "V_Float", "params": [value]}
    elif ftype == "Bool":
        ldtk_type = "Bool"
        real_val = {"id": "V_Bool", "params": [value]}
    else:
        raise ValueError(f"Unknown ftype: {ftype}")

    return {
        "__identifier": identifier,
        "__type": ldtk_type,
        "__value": value,
        "__tile": None,
        "defUid": def_uid,
        "realEditorValues": [real_val],
    }


def make_prop_entity_instance(identifier, px, world_offset, def_uid, field_uids,
                               model_id, scale_x, scale_y, scale_z, z_depth,
                               rotation_y, foreground, smart_color="#E91E63"):
    """Build an entity instance dict for a prop."""
    grid_x = px[0] // 18
    grid_y = px[1] // 18
    world_x = world_offset[0] + px[0]
    world_y = world_offset[1] + px[1]
    return {
        "__identifier": identifier,
        "__grid": [grid_x, grid_y],
        "__pivot": [0.5, 1],
        "__tags": [],
        "__tile": None,
        "__smartColor": smart_color,
        "iid": str(uuid.uuid4()),
        "width": 18,
        "height": 18,
        "defUid": def_uid,
        "px": px,
        "fieldInstances": [
            make_field_instance("model_id",   "String", model_id,   field_uids[0]),
            make_field_instance("scale_x",    "Float",  scale_x,    field_uids[1]),
            make_field_instance("scale_z",    "Float",  scale_z,    field_uids[2]),
            make_field_instance("z_depth",    "Float",  z_depth,    field_uids[3]),
            make_field_instance("rotation_y", "Float",  rotation_y, field_uids[4]),
            make_field_instance("scale_y",    "Float",  scale_y,    field_uids[5]),
            make_field_instance("foreground", "Bool",   foreground, field_uids[6]),
        ],
        "__worldX": world_x,
        "__worldY": world_y,
    }


# ─── main ───────────────────────────────────────────────────────────────────

def main():
    # Backup
    shutil.copy2(LDTK_PATH, BACKUP_PATH)
    print(f"Backed up to {BACKUP_PATH}")

    with open(LDTK_PATH, "r", encoding="utf-8") as f:
        data = json.load(f)

    # ── Change 1: Add 4 new entity defs ─────────────────────────────────────
    COLOR = "#E91E63"

    new_entities = [
        make_prop_entity(
            uid=492,
            identifier="Prop_SanctuaryArch",
            color=COLOR,
            field_uids=list(range(493, 500)),
            model_id="models/sanctuary/ornate+chinese+arch+3d+model.glb",
            scale_x=60, scale_y=60, scale_z=60,
            z_depth=-5, rotation_y=-1.5707963, foreground=False,
        ),
        make_prop_entity(
            uid=500,
            identifier="Prop_SanctuaryLion",
            color=COLOR,
            field_uids=list(range(501, 508)),
            model_id="models/sanctuary/sanctuary_lionstatue.glb",
            scale_x=30, scale_y=30, scale_z=30,
            z_depth=2, rotation_y=0.0, foreground=False,
        ),
        make_prop_entity(
            uid=508,
            identifier="Prop_CherryBlossom",
            color=COLOR,
            field_uids=list(range(509, 516)),
            model_id="models/sanctuary/tree_cherryblossom.glb",
            scale_x=80, scale_y=80, scale_z=1,
            z_depth=10, rotation_y=0.0, foreground=True,
        ),
        make_prop_entity(
            uid=516,
            identifier="Prop_Temple",
            color=COLOR,
            field_uids=list(range(517, 524)),
            model_id="models/sanctuary/asian+temple+island+3d+model.glb",
            scale_x=50, scale_y=50, scale_z=50,
            z_depth=-5, rotation_y=-1.5707963, foreground=False,
        ),
    ]

    data["defs"]["entities"].extend(new_entities)
    data["nextUid"] = 524
    print(f"Added {len(new_entities)} entity defs. nextUid → 524")

    # ── Change 2: Fix City Exit → Sanctuary ─────────────────────────────────
    city = next(l for l in data["levels"] if l["uid"] == 114)
    fixed = False
    for li in city["layerInstances"]:
        if li["__type"] == "Entities":
            for ei in li["entityInstances"]:
                if ei["defUid"] == 16:  # Exit
                    for fi in ei["fieldInstances"]:
                        if fi["__identifier"] == "exit_next_level":
                            fi["__value"] = "Sanctuary"
                            fi["realEditorValues"] = [
                                {"id": "V_String", "params": ["Sanctuary"]}
                            ]
                            fixed = True
    if fixed:
        print("Fixed City Exit → Sanctuary")
    else:
        print("WARNING: City Exit not found!")

    # ── Change 3: Add Sanctuary level ───────────────────────────────────────
    SANCTUARY_UID = 525
    LEVEL_WIDTH  = 864   # 48 * 18
    LEVEL_HEIGHT = 396   # 22 * 18
    CW = 48
    CH = 22
    WORLD_X = 0
    WORLD_Y = 3000

    # Build intGridCsv (top-down in LDtk)
    # LDtk rows 0-18 (top): all 0
    # LDtk rows 19-21 (bottom = compiled rows 2-0): solid (1),
    #   EXCEPT cols 43-46 are 0, col 47 = 1
    int_grid = []
    for row in range(CH):
        if row < 19:
            int_grid.extend([0] * CW)
        else:
            row_vals = []
            for col in range(CW):
                if 43 <= col <= 46:
                    row_vals.append(0)
                else:
                    row_vals.append(1)
            int_grid.extend(row_vals)

    assert len(int_grid) == CW * CH, f"intGridCsv length mismatch: {len(int_grid)}"

    # World offset for __worldX/__worldY computation
    world_offset = (WORLD_X, WORLD_Y)

    # Arch field uids: 493-499
    ARCH_FIELD_UIDS  = list(range(493, 500))
    # Lion field uids: 501-507
    LION_FIELD_UIDS  = list(range(501, 508))

    ARCH_MODEL  = "models/sanctuary/ornate+chinese+arch+3d+model.glb"
    LION_MODEL  = "models/sanctuary/sanctuary_lionstatue.glb"

    entity_instances = []

    # Spawn at compiled [-396, -128], origin=(-432, -200)
    spawn_px = [36, 324]  # -396-(-432)=36, 396-(-128-(-200))=324
    entity_instances.append({
        "__identifier": "Spawn",
        "__grid": [spawn_px[0] // 18, spawn_px[1] // 18],
        "__pivot": [0.5, 1],
        "__tags": [],
        "__tile": None,
        "__smartColor": "#FFEB3B",
        "iid": str(uuid.uuid4()),
        "width": 18,
        "height": 18,
        "defUid": 10,
        "px": spawn_px,
        "fieldInstances": [],
        "__worldX": WORLD_X + spawn_px[0],
        "__worldY": WORLD_Y + spawn_px[1],
    })

    # Prop_SanctuaryArch at px=[72, 306]
    entity_instances.append(make_prop_entity_instance(
        identifier="Prop_SanctuaryArch",
        px=[72, 306],
        world_offset=world_offset,
        def_uid=492,
        field_uids=ARCH_FIELD_UIDS,
        model_id=ARCH_MODEL,
        scale_x=60, scale_y=60, scale_z=60,
        z_depth=-5, rotation_y=-1.5707963, foreground=False,
    ))

    # Lion at px=[52, 336], rotation_y=0
    entity_instances.append(make_prop_entity_instance(
        identifier="Prop_SanctuaryLion",
        px=[52, 336],
        world_offset=world_offset,
        def_uid=500,
        field_uids=LION_FIELD_UIDS,
        model_id=LION_MODEL,
        scale_x=30, scale_y=30, scale_z=30,
        z_depth=2, rotation_y=0.0, foreground=False,
    ))

    # Lion at px=[92, 336], rotation_y=pi (facing opposite)
    entity_instances.append(make_prop_entity_instance(
        identifier="Prop_SanctuaryLion",
        px=[92, 336],
        world_offset=world_offset,
        def_uid=500,
        field_uids=LION_FIELD_UIDS,
        model_id=LION_MODEL,
        scale_x=30, scale_y=30, scale_z=30,
        z_depth=2, rotation_y=3.14159, foreground=False,
    ))

    # Lion at px=[382, 336], rotation_y=0
    entity_instances.append(make_prop_entity_instance(
        identifier="Prop_SanctuaryLion",
        px=[382, 336],
        world_offset=world_offset,
        def_uid=500,
        field_uids=LION_FIELD_UIDS,
        model_id=LION_MODEL,
        scale_x=30, scale_y=30, scale_z=30,
        z_depth=2, rotation_y=0.0, foreground=False,
    ))

    # Lion at px=[422, 336], rotation_y=pi
    entity_instances.append(make_prop_entity_instance(
        identifier="Prop_SanctuaryLion",
        px=[422, 336],
        world_offset=world_offset,
        def_uid=500,
        field_uids=LION_FIELD_UIDS,
        model_id=LION_MODEL,
        scale_x=30, scale_y=30, scale_z=30,
        z_depth=2, rotation_y=3.14159, foreground=False,
    ))

    entities_layer = {
        "__identifier": "Entities",
        "__type": "Entities",
        "__cWid": CW,
        "__cHei": CH,
        "__gridSize": 18,
        "__opacity": 1,
        "__pxTotalOffsetX": 0,
        "__pxTotalOffsetY": 0,
        "__tilesetDefUid": None,
        "__tilesetRelPath": None,
        "iid": str(uuid.uuid4()),
        "levelId": SANCTUARY_UID,
        "layerDefUid": 1,
        "pxOffsetX": 0,
        "pxOffsetY": 0,
        "visible": True,
        "optionalRules": [],
        "intGridCsv": [],
        "autoLayerTiles": [],
        "seed": 1234,
        "overrideTilesetUid": None,
        "gridTiles": [],
        "entityInstances": entity_instances,
    }

    tiles_layer = {
        "__identifier": "Tiles",
        "__type": "IntGrid",
        "__cWid": CW,
        "__cHei": CH,
        "__gridSize": 18,
        "__opacity": 1,
        "__pxTotalOffsetX": 0,
        "__pxTotalOffsetY": 0,
        "__tilesetDefUid": None,
        "__tilesetRelPath": None,
        "iid": str(uuid.uuid4()),
        "levelId": SANCTUARY_UID,
        "layerDefUid": 2,
        "pxOffsetX": 0,
        "pxOffsetY": 0,
        "visible": True,
        "optionalRules": [],
        "intGridCsv": int_grid,
        "autoLayerTiles": [],
        "seed": 5678,
        "overrideTilesetUid": None,
        "gridTiles": [],
        "entityInstances": [],
    }

    # Level field instances: OriginX=-432, OriginY=-200 for Sanctuary
    # Origin: worldX and worldY correspond to level's (0,0) in compiled space
    # City has worldX=0, worldY=2000, originX=-864, originY=-200
    # Sanctuary: worldX=0, worldY=3000, half of 864 wide = 432
    sanctuary_field_instances = [
        {
            "__identifier": "OriginX",
            "__type": "Float",
            "__value": -432.0,
            "__tile": None,
            "defUid": 90,
            "realEditorValues": [{"id": "V_Float", "params": [-432.0]}],
        },
        {
            "__identifier": "OriginY",
            "__type": "Float",
            "__value": -200.0,
            "__tile": None,
            "defUid": 91,
            "realEditorValues": [{"id": "V_Float", "params": [-200.0]}],
        },
    ]

    sanctuary_level = {
        "identifier": "Sanctuary",
        "iid": str(uuid.uuid4()),
        "uid": SANCTUARY_UID,
        "worldX": WORLD_X,
        "worldY": WORLD_Y,
        "worldDepth": 0,
        "pxWid": LEVEL_WIDTH,
        "pxHei": LEVEL_HEIGHT,
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
        "fieldInstances": sanctuary_field_instances,
        "layerInstances": [entities_layer, tiles_layer],
        "__neighbours": [],
    }

    data["levels"].append(sanctuary_level)
    data["nextUid"] = 526  # 525 used for level uid, bump to 526
    print(f"Added Sanctuary level (uid={SANCTUARY_UID}). nextUid → 526")

    # ── Write ────────────────────────────────────────────────────────────────
    with open(LDTK_PATH, "w", encoding="utf-8") as f:
        json.dump(data, f, indent="\t")
    print(f"Wrote {LDTK_PATH}")

    # ── Validate ─────────────────────────────────────────────────────────────
    with open(LDTK_PATH, "r", encoding="utf-8") as f:
        check = json.load(f)

    # Verify entity defs
    entity_ids = {e["uid"]: e["identifier"] for e in check["defs"]["entities"]}
    for uid in [492, 500, 508, 516]:
        assert uid in entity_ids, f"Missing entity uid {uid}"
        print(f"  OK: entity uid={uid} identifier={entity_ids[uid]}")

    # Verify nextUid
    assert check["nextUid"] == 526, f"nextUid={check['nextUid']}"
    print(f"  OK: nextUid=526")

    # Verify City exit
    city2 = next(l for l in check["levels"] if l["uid"] == 114)
    for li in city2["layerInstances"]:
        if li["__type"] == "Entities":
            for ei in li["entityInstances"]:
                if ei["defUid"] == 16:
                    for fi in ei["fieldInstances"]:
                        if fi["__identifier"] == "exit_next_level":
                            assert fi["__value"] == "Sanctuary", f"City exit value={fi['__value']}"
                            print(f"  OK: City Exit → {fi['__value']}")

    # Verify Sanctuary level
    sanc = next((l for l in check["levels"] if l["identifier"] == "Sanctuary"), None)
    assert sanc is not None, "Sanctuary level not found"
    assert sanc["uid"] == SANCTUARY_UID
    assert len(sanc["layerInstances"]) == 2
    tiles_li = next(l for l in sanc["layerInstances"] if l["__type"] == "IntGrid")
    assert len(tiles_li["intGridCsv"]) == CW * CH, \
        f"intGridCsv length={len(tiles_li['intGridCsv'])}, expected {CW*CH}"
    ents_li = next(l for l in sanc["layerInstances"] if l["__type"] == "Entities")
    print(f"  OK: Sanctuary level uid={sanc['uid']}, entities={len(ents_li['entityInstances'])}, intGridCsv len={len(tiles_li['intGridCsv'])}")

    print("\nAll validations passed.")


if __name__ == "__main__":
    main()
