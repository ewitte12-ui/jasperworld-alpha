#!/usr/bin/env python3
"""Repack newraccoon.png into assets/raccoon.png (128×128 cells).

Source bands (complete sprites only — no gap-closing needed):
  Band A: y=292-407  — 7 sitting raccoons (complete, with ears)
  Band B: y=463-611  — 7 running raccoons (complete, with ears)
  Band C: y=636-780  — 2 jump poses (complete, split at x=350)
  Band D: y=816-936  — 4 idle/sitting poses (complete)

Target: 1024×512, 8×4 grid, 128×128 cells.
  Row 0 (0-6):  Band A sitting (7 frames)
  Row 1 (8-14): Band B running (7 frames) ← walk animation
  Row 2: unused
  Row 3: (24-25) Band C jumps, (26-29) Band D idle
"""

from PIL import Image
import numpy as np

src = Image.open("buildassets/newraccoon.png").convert("RGBA")
arr = np.array(src)

dst_cell = 128
padding = 2
usable = dst_cell - 2 * padding
dst = Image.new("RGBA", (1024, 512), (0, 0, 0, 0))


def extract_and_place(region, cx0, cx1, dst_row, dst_col):
    """Extract a sprite from region columns cx0:cx1, scale to fit cell, place in dst."""
    sprite_region = region[:, cx0:cx1]
    sa = sprite_region[:, :, 3] > 10
    if not sa.any():
        return None

    # Crop to content bounds
    rows_with = np.any(sa, axis=1)
    cols_with = np.any(sa, axis=0)
    r0 = int(np.argmax(rows_with))
    r1 = len(rows_with) - int(np.argmax(rows_with[::-1]))
    c0 = int(np.argmax(cols_with))
    c1 = len(cols_with) - int(np.argmax(cols_with[::-1]))

    cropped = Image.fromarray(sprite_region[r0:r1, c0:c1])
    sw, sh = cropped.size

    # Scale to fit usable area
    scale = min(usable / sw, usable / sh)
    if scale < 1.0:
        cropped = cropped.resize(
            (max(1, int(sw * scale)), max(1, int(sh * scale))),
            Image.NEAREST,
        )
        sw, sh = cropped.size

    # Center horizontally, bottom-align
    dx = dst_col * dst_cell + padding + (usable - sw) // 2
    dy = dst_row * dst_cell + padding + (usable - sh)

    mask = cropped.split()[3].point(lambda a: 255 if a > 0 else 0)
    dst.paste(cropped, (dx, dy), mask)
    return (sw, sh)


# Band A: sitting cycle → row 0
region_a = arr[292:408, :, :]
alpha_a = region_a[:, :, 3] > 10
col_has_a = np.any(alpha_a, axis=0)
left_a = int(np.argmax(col_has_a))
right_a = len(col_has_a) - 1 - int(np.argmax(col_has_a[::-1]))
col_w_a = (right_a - left_a + 1) / 7
print(f"Band A (sitting): x={left_a}-{right_a}, col_w={col_w_a:.0f}")
for i in range(7):
    cx0 = left_a + int(i * col_w_a)
    cx1 = left_a + int((i + 1) * col_w_a)
    result = extract_and_place(region_a, cx0, cx1, 0, i)
    if result:
        print(f"  [{i}] {result[0]}x{result[1]} -> (0,{i})")

# Band B: running → row 1
region_b = arr[463:612, :, :]
alpha_b = region_b[:, :, 3] > 10
col_has_b = np.any(alpha_b, axis=0)
left_b = int(np.argmax(col_has_b))
right_b = len(col_has_b) - 1 - int(np.argmax(col_has_b[::-1]))
col_w_b = (right_b - left_b + 1) / 7
print(f"\nBand B (running): x={left_b}-{right_b}, col_w={col_w_b:.0f}")
for i in range(7):
    cx0 = left_b + int(i * col_w_b)
    cx1 = left_b + int((i + 1) * col_w_b)
    result = extract_and_place(region_b, cx0, cx1, 1, i)
    if result:
        print(f"  [{i}] {result[0]}x{result[1]} -> (1,{i})")

# Band C: jumps → row 3, cols 0-1 (split at x=350)
region_c = arr[636:781, :, :]
print(f"\nBand C (jump):")
for i, (cx0, cx1) in enumerate([(182, 350), (355, 560)]):
    result = extract_and_place(region_c, cx0, cx1, 3, i)
    if result:
        print(f"  [{i}] {result[0]}x{result[1]} -> (3,{i})")

# Band D: idle → row 3, cols 2-5
region_d = arr[816:937, :, :]
alpha_d = region_d[:, :, 3] > 10
col_has_d = np.any(alpha_d, axis=0)
left_d = int(np.argmax(col_has_d))
right_d = len(col_has_d) - 1 - int(np.argmax(col_has_d[::-1]))
col_w_d = (right_d - left_d + 1) / 4
print(f"\nBand D (idle): x={left_d}-{right_d}, col_w={col_w_d:.0f}")
for i in range(4):
    cx0 = left_d + int(i * col_w_d)
    cx1 = left_d + int((i + 1) * col_w_d)
    result = extract_and_place(region_d, cx0, cx1, 3, 2 + i)
    if result:
        print(f"  [{i}] {result[0]}x{result[1]} -> (3,{2+i})")

dst.save("assets/raccoon.png")
print(f"\nSaved assets/raccoon.png ({dst.size[0]}x{dst.size[1]})")

# Verify
result = np.array(dst)
print("\nVerification:")
for row in range(4):
    for col in range(8):
        cell = result[row * 128:(row + 1) * 128, col * 128:(col + 1) * 128]
        alpha = cell[:, :, 3] > 10
        vis = alpha.sum()
        if vis < 50:
            continue
        frame = row * 8 + col
        rows_with = np.any(alpha, axis=1)
        n_bands = 0
        in_band = False
        for y in range(128):
            if rows_with[y] and not in_band:
                in_band = True
                n_bands += 1
            elif not rows_with[y] and in_band:
                in_band = False
        status = "OK" if n_bands == 1 else f"{n_bands-1} GAP(S)"
        print(f"  Frame {frame:2d}: {vis:5d}px  {status}")
