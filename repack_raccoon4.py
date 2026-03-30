#!/usr/bin/env python3
"""Repack newraccoon4.png into raccoon.png using flood-fill background removal."""

from PIL import Image
import numpy as np
from scipy import ndimage

src = Image.open("buildassets/newraccoon4.png").convert("RGBA")
arr = np.array(src)
h, w = arr.shape[:2]

# Detect background using flood fill from borders.
# Background pixels are low-saturation grey connected to the image border.
r, g, b = arr[:,:,0].astype(float), arr[:,:,1].astype(float), arr[:,:,2].astype(float)
max_rgb = np.maximum(np.maximum(r, g), b)
min_rgb = np.minimum(np.minimum(r, g), b)
saturation = max_rgb - min_rgb

# Pixels that could be background: low color variation, bright-ish grey
could_be_bg = (saturation < 20) & (max_rgb > 150)

# Label connected components and find those touching the border
labeled, num_features = ndimage.label(could_be_bg)
border_labels = set()
border_labels.update(labeled[0, :].flatten())
border_labels.update(labeled[-1, :].flatten())
border_labels.update(labeled[:, 0].flatten())
border_labels.update(labeled[:, -1].flatten())
border_labels.discard(0)

is_bg = np.isin(labeled, list(border_labels))
arr[is_bg, 3] = 0

removed = is_bg.sum()
print(f"Background removed: {removed}/{h*w} pixels ({100*removed/(h*w):.1f}%)")

src_clean = Image.fromarray(arr)

# Source grid: 5x5, cell = 128x128
src_cols, src_rows = 5, 5
cell_w, cell_h = w // src_cols, h // src_rows

# Target: 8x4, 128x128, 1024x512
dst_cell = 128
dst = Image.new("RGBA", (1024, 512), (0, 0, 0, 0))

# Mapping
mappings = []
for c in range(5): mappings.append((0, c, 0, c))
mappings.append((0, 0, 0, 5)); mappings.append((0, 1, 0, 6))
for c in range(5): mappings.append((1, c, 1, c))
mappings.append((1, 0, 1, 5)); mappings.append((1, 1, 1, 6))
for c in range(5): mappings.append((2, c, 2, c))
mappings.append((4, 0, 3, 0))  # JUMP
mappings.append((4, 1, 3, 2))  # IDLE
mappings.append((4, 2, 3, 3))  # HURT

for (sr, sc, dr, dc) in mappings:
    x0, y0 = sc * cell_w, sr * cell_h
    cell = src_clean.crop((x0, y0, x0 + cell_w, y0 + cell_h))
    bbox = cell.getbbox()
    if bbox is None:
        print(f"  WARNING: empty cell ({sr},{sc})")
        continue
    sprite = cell.crop(bbox)
    sw, sh = sprite.size

    scale = min(dst_cell / sw, dst_cell / sh)
    if scale < 1.0:
        sprite = sprite.resize((max(1, int(sw * scale)), max(1, int(sh * scale))), Image.LANCZOS)
        sw, sh = sprite.size

    dx = dc * dst_cell + (dst_cell - sw) // 2
    dy = dr * dst_cell + (dst_cell - sh)
    dst.paste(sprite, (dx, dy), sprite)

dst.save("assets/raccoon.png")
print("Saved assets/raccoon.png")

# Verify
result = np.array(dst)
for row in range(4):
    for col in range(8):
        cell = result[row*128:(row+1)*128, col*128:(col+1)*128]
        vis = (cell[:,:,3] > 0).sum()
        if vis > 0:
            print(f"  Frame {row*8+col} (r{row}c{col}): {vis} pixels")
