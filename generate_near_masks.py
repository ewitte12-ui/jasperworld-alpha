#!/usr/bin/env python3
"""Generate narrow organic silhouette masks for NEAR foreground elements.

Output: 128×256 RGBA PNGs (1:2 aspect matching 60×120 world-unit quads).
White RGB + organic alpha — used as base_color_texture with AlphaMode::Blend.
"""
import numpy as np
from PIL import Image, ImageFilter
import os

OUTPUT_DIR = "assets/backgrounds/subdivision/near"
WIDTH, HEIGHT = 128, 256  # 1:2 aspect ratio

def generate_mask(seed: int, output_path: str):
    rng = np.random.RandomState(seed)
    alpha = np.zeros((HEIGHT, WIDTH), dtype=np.float32)

    cx, cy = WIDTH / 2, HEIGHT / 2
    # Aspect-corrected distance so the blob is naturally shaped for 1:2 rect
    aspect = WIDTH / HEIGHT  # 0.5

    # Generate organic boundary as a function of angle
    n_harmonics = 6
    freqs = rng.uniform(0.8, 3.5, n_harmonics)
    amps = rng.uniform(0.04, 0.12, n_harmonics)
    phases = rng.uniform(0, 2 * np.pi, n_harmonics)

    for y in range(HEIGHT):
        for x in range(WIDTH):
            dx = (x - cx) / (WIDTH / 2)
            dy = (y - cy) / (HEIGHT / 2)
            # Elliptical distance (unit circle in both axes)
            dist = np.sqrt(dx**2 + dy**2)
            angle = np.arctan2(dy, dx)

            # Organic boundary with sine harmonics
            boundary = 0.72
            for i in range(n_harmonics):
                boundary += amps[i] * np.sin(freqs[i] * angle + phases[i])

            if dist < boundary:
                edge_dist = boundary - dist
                # Soft Gaussian-like falloff near edge
                falloff_width = 0.18
                alpha[y, x] = min(1.0, edge_dist / falloff_width)

    # Convert to uint8
    alpha_u8 = (alpha * 255).clip(0, 255).astype(np.uint8)

    # Gaussian blur for soft edges
    alpha_img = Image.fromarray(alpha_u8, mode='L')
    alpha_img = alpha_img.filter(ImageFilter.GaussianBlur(radius=4))

    # Create RGBA: white RGB + organic alpha
    rgba = Image.new('RGBA', (WIDTH, HEIGHT), (255, 255, 255, 0))
    rgba.putalpha(alpha_img)
    rgba.save(output_path)
    print(f"  Saved {output_path} ({WIDTH}x{HEIGHT})")

if __name__ == "__main__":
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    masks = [
        (42,  f"{OUTPUT_DIR}/near_silhouette_a.png"),
        (255, f"{OUTPUT_DIR}/near_silhouette_b.png"),
    ]

    for seed, path in masks:
        generate_mask(seed, path)
        # Verify
        img = Image.open(path)
        assert img.size == (WIDTH, HEIGHT), f"Bad size: {img.size}"
        assert img.mode == "RGBA", f"Bad mode: {img.mode}"

    print("Done — 2 masks generated at 128x256 (1:2 aspect)")
