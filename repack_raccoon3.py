#!/usr/bin/env python3
"""Repack newraccoon3.png (640x640, 7-col x 4-row, right-facing)
into raccoon.png (1024x512, 8-col x 4-row, 128x128 cells, left-facing, bottom-aligned)."""

from PIL import Image

src = Image.open("buildassets/newraccoon3.png").convert("RGBA")
src_w, src_h = src.size  # 640x640

# Source grid: 7 cols (rows 0-2), 3 cols (row 3), 4 rows
src_cols_per_row = [7, 7, 7, 3]
src_cell_w = src_w // 7   # ~91
src_cell_h = src_h // 4   # 160

# Target grid: 8 cols x 4 rows, 128x128 cells, 1024x512
dst_cell = 128
dst = Image.new("RGBA", (1024, 512), (0, 0, 0, 0))

for row in range(4):
    num_cols = src_cols_per_row[row]
    for col in range(num_cols):
        # Extract source cell
        x0 = col * src_cell_w
        y0 = row * src_cell_h
        cell = src.crop((x0, y0, x0 + src_cell_w, y0 + src_cell_h))

        # Flip horizontally (right-facing -> left-facing)
        cell = cell.transpose(Image.FLIP_LEFT_RIGHT)

        # Find bounding box of non-transparent content
        bbox = cell.getbbox()
        if bbox is None:
            continue

        sprite = cell.crop(bbox)
        sw, sh = sprite.size

        # Scale to fit within 128x128 while preserving aspect ratio
        scale = min(dst_cell / sw, dst_cell / sh)
        if scale < 1.0:
            new_w = int(sw * scale)
            new_h = int(sh * scale)
            sprite = sprite.resize((new_w, new_h), Image.LANCZOS)
            sw, sh = sprite.size

        # Bottom-align and center horizontally in 128x128 cell
        dx = col * dst_cell + (dst_cell - sw) // 2
        dy = row * dst_cell + (dst_cell - sh)  # bottom-align

        dst.paste(sprite, (dx, dy), sprite)

dst.save("assets/raccoon.png")
print("Saved assets/raccoon.png (1024x512, 8x4 grid, 128x128 cells, left-facing)")

# Print what's in each cell for verification
print("\nLayout:")
print("  Row 0, cols 0-6: Walk/run cycle (7 frames)")
print("  Row 1, cols 0-6: Walk/run variant (7 frames)")
print("  Row 2, cols 0-6: Standing/walking (7 frames)")
print("  Row 3, col 0: JUMP, col 1: IDLE, col 2: HURT")
